//! 与内嵌服务端的进程内通讯（原 server 已迁入客户端进程）。
//!
//! 合并后服务端与客户端同进程，不再有“连接后台服务”的概念：
//! 数据上报/配置同步均直接调用 `crate::server` 暴露的同步 API。
//! 对应 AHK events.ahk。
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::stats::Shared;

/// 客户端本地修改配置（写 ini 之后）直接进程内通知服务端，
/// 服务端据此合并落盘并向 Web 广播，保证两端配置一致。
/// （合并前：经 WS 发送 ConfigUpdate；现改为直接调用。）
pub fn notify_server_config(section: &str, key: &str, value: &str) {
    let mut kv = serde_json::Map::new();
    kv.insert(key.to_string(), Value::String(value.to_string()));
    let mut sections = serde_json::Map::new();
    sections.insert(section.to_string(), Value::Object(kv));
    let msg = json!({
        "type": "ConfigUpdate",
        "config": Value::Object(sections),
    });
    crate::server::apply_client_config(msg);
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn OutputDebugStringW(lp_output_string: *const u16);
}

/// 输出调试信息到调试器（DebugViewNt 等工具可查看），统一以 "AHK-" 开头
#[cfg(windows)]
fn debug_out(_msg: &str) {
    //let text = format!("AHK-{}\0", msg);
    //let wide: Vec<u16> = text.encode_utf16().collect();
    //unsafe { OutputDebugStringW(wide.as_ptr()) }
}

/// macOS 没有 DebugView：输出到 stderr（GUI 应用已在启动时通过
/// silence_stdio 丢弃标准输出，因此不会再显示）
#[cfg(target_os = "macos")]
fn debug_out(msg: &str) {
    eprintln!("AHK-{}", msg);
}

/// SendPCInfo：屏幕信息上报，同时更新鼠标距离校验边界。
/// 合并前走 HTTP POST /sendPCInfo，现改为进程内调用。
fn send_pc_info(shared: &Arc<Shared>, flag: i32) {
    let monitors = shared.monitors.read().unwrap().clone();
    if monitors.is_empty() {
        return;
    }
    let mut arr = Vec::new();
    let (mut min_l, mut min_t, mut max_r, mut max_b) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for (l, t, r, b) in monitors {
        min_l = min_l.min(l);
        min_t = min_t.min(t);
        max_r = max_r.max(r);
        max_b = max_b.max(b);
        arr.push(json!({"Left": l, "Top": t, "Right": r, "Bottom": b}));
    }
    *shared.screen_bounds.lock().unwrap() = (min_l, min_t, max_r, max_b);
    let major = shared.major_version.clone();
    let body = json!({"screen": arr, "flag": flag, "majorVersion": major});
    crate::server::send_pc_info(body);
}

/// 上报数据中的单个字母统一转小写（字母前后不能紧邻其他字母/数字，
/// 避免把 F5、LControl、App-Key-xxx 这类名称里的字母也改掉）
fn lower_single_letters(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    for (i, &c) in chars.iter().enumerate() {
        let prev_alnum = i > 0 && chars[i - 1].is_ascii_alphanumeric();
        let next_alnum = chars.get(i + 1).is_some_and(|n| n.is_ascii_alphanumeric());
        if c.is_ascii_uppercase() && !prev_alnum && !next_alnum {
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// 序列化 AllKeyRecord（与 AHK JSON.stringify(AllKeyRecord) 字段一致）
pub fn build_data_body(shared: &Arc<Shared>) -> Value {
    let stats = shared.stats.lock().unwrap();
    let mut obj = serde_json::Map::new();
    // 按键计数（BTreeMap 按名称排序）；单字母按键统一小写上报
    for (k, v) in &stats.keys {
        obj.insert(lower_single_letters(k), json!(v));
    }
    obj.insert("tick".into(), json!(stats.tick));
    obj.insert("updateTime".into(), json!(stats.update_time));
    let cfg = shared.cfg.read().unwrap();
    if cfg.record_minute {
        obj.insert("MinuteRecords".into(), serde_json::to_value(&stats.minute_recs).unwrap_or(json!([])));
    }
    if cfg.record_history_max > 0 {
        let history: Vec<String> = stats.record_history.iter().map(|s| lower_single_letters(s)).collect();
        obj.insert("recordHistory".into(), json!(history));
    }
    Value::Object(obj)
}

/// 网络线程：进程内上报 /data 与 /sendPCInfo（原 HTTP 调用替换为直接调用）。
/// 内嵌服务端由 main.rs 在 UI 循环前同步启动，此处只负责周期性数据上报。
pub fn start(shared: Arc<Shared>) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("net".into())
        .spawn(move || {
            // 初始屏幕边界
            {
                let mons = crate::winutil::monitors();
                let (mut min_l, mut min_t, mut max_r, mut max_b) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
                for (l, t, r, b) in &mons {
                    min_l = min_l.min(*l);
                    min_t = min_t.min(*t);
                    max_r = max_r.max(*r);
                    max_b = max_b.max(*b);
                }
                *shared.screen_bounds.lock().unwrap() = (min_l, min_t, max_r, max_b);
                *shared.monitors.write().unwrap() = mons;
            }

            let mut last_send = Instant::now() - Duration::from_secs(10);
            let mut sent_init_pcinfo = false;

            loop {
                if shared.exit.load(Ordering::Relaxed) {
                    break;
                }

                // 服务端就绪后首次上报 PC 信息（flag=0 触发 autoBackup，对应原连上后 send_pc_info(0)）
                if !sent_init_pcinfo {
                    sent_init_pcinfo = true;
                    send_pc_info(&shared, 0);
                }

                // PCInfo 请求（屏幕配置变化等）
                let pc_flag = shared.pcinfo_dirty.swap(0, Ordering::Relaxed);
                if pc_flag != 0 {
                    send_pc_info(&shared, pc_flag);
                }

                // 按 1 秒限频上报 /data（进程内调用）
                let dirty = shared.stats.lock().unwrap().dirty;
                if dirty && last_send.elapsed().as_millis() >= 1000 {
                    let body = build_data_body(&shared);
                    shared.stats.lock().unwrap().dirty = false;
                    last_send = Instant::now();
                    crate::server::upload_data(body);
                }

                std::thread::sleep(Duration::from_millis(200));
            }
        })
        .expect("spawn net thread")
}

/// 应用一份新配置：替换内存配置并让钩子参数立即生效。
/// 配置文件由发起变更的一端负责落盘，这里只更新内存。
fn apply_config(shared: &Arc<Shared>, mut new_cfg: crate::config::Config) {
    let mut cfg = shared.cfg.write().unwrap();
    let show_changed = cfg.need_show_key != new_cfg.need_show_key;
    // serverPort 变更后原监听端口已失效，需要重启进程按新端口重新绑定
    let port_changed = cfg.server_port != new_cfg.server_port;
    new_cfg.generation = cfg.generation + 1;
    *cfg = new_cfg;
    drop(cfg);
    if show_changed {
        // 通知托盘同步「按键显示」勾选状态（Web 端改动 needShowKey 后菜单不再残留旧状态）
        shared.show_key_generation.fetch_add(1, Ordering::Relaxed);
    }
    crate::input::refresh_hook_params();
    if port_changed {
        // 稍等片刻再退出：确保 setPara 的响应已写回浏览器，页面不会误报保存失败
        let shared = shared.clone();
        let _ = std::thread::Builder::new()
            .name("skb-port-restart".into())
            .spawn(move || {
                std::thread::sleep(Duration::from_millis(500));
                shared.restart.store(true, Ordering::Relaxed);
                shared.exit.store(true, Ordering::Relaxed);
            });
    }
}

/// Bridge 线程（替代原 WS 客户端线程）：
/// - 把 UI 产生的实时按键文本进程内广播给 ahkKeyShow 显示端（原 WS 发送文本帧）；
/// - 接收服务端进程内推送的配置，应用到客户端内存配置。
pub fn start_bridge(
    shared: Arc<Shared>,
    ws_rx: crossbeam_channel::Receiver<String>,
    cfg_rx: crossbeam_channel::Receiver<serde_json::Value>,
) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("skb-bridge".into())
        .spawn(move || {
            // 丢弃服务端就绪前积压的按键文本（AHK 未连接时 sendText 直接失败，不补发）
            while ws_rx.try_recv().is_ok() {}
            loop {
                if shared.exit.load(Ordering::Relaxed) {
                    break;
                }

                // 实时按键文本：阻塞等待最多 50ms，保证显示端近实时
                match ws_rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(text) => {
                        if !text.is_empty() {
                            debug_out(&format!("bridge send {}", text));
                            crate::server::broadcast_key_show(text);
                        }
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                    Err(_) => break,
                }

                // 服务端 → 客户端 配置推送
                while let Ok(map) = cfg_rx.try_recv() {
                    if let Some(new_cfg) = crate::config::Config::from_ws(&map) {
                        apply_config(&shared, new_cfg);
                    }
                }
            }
        })
        .expect("spawn bridge thread")
}
