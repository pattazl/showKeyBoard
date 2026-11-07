//! 与后台服务通讯：HTTP(/version /data /sendPCInfo /exit) + WebSocket(ahkClient/IniMonitor)
//! 对应 AHK events.ahk。
#![allow(dead_code)]
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::stats::Shared;

const UA: &str = "ShowKeyBoardRust";

#[link(name = "kernel32")]
unsafe extern "system" {
    fn OutputDebugStringW(lp_output_string: *const u16);
}

/// 输出调试信息到调试器（DebugViewNt 等工具可查看），统一以 "AHK-" 开头
fn debug_out(msg: &str) {
    let text = format!("AHK-{}\0", msg);
    let wide: Vec<u16> = text.encode_utf16().collect();
    unsafe { OutputDebugStringW(wide.as_ptr()) }
}

fn post_json(url: &str, body: &str, timeout: u64) -> Result<String, String> {
    if !body.is_empty() {
        debug_out(&format!("POST {} {}", url, body));
    }
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_millis(timeout))
        .user_agent(UA)
        .build();
    let resp = agent
        .post(url)
        .set("Content-Type", "application/json")
        .send_string(body)
        .map_err(|e| e.to_string())?;
    resp.into_string().map_err(|e| e.to_string())
}

/// SendPCInfo：屏幕信息上报，同时更新鼠标距离校验边界
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
    let body = json!({"screen": arr, "flag": flag, "majorVersion": major}).to_string();
    let url = { shared.cfg.read().unwrap().server_url() };
    let url = format!("{}/sendPCInfo", url);
    let _ = post_json(&url, &body, 8000);
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

/// 启动 Node 后台服务（对应 AHK startServer 的简化版）
fn start_node_server(shared: &Arc<Shared>) {
    use std::os::windows::process::CommandExt;
    let (http_path, exec, debug) = {
        let cfg = shared.cfg.read().unwrap();
        (cfg.http_path.clone(), cfg.server_exec_name.clone(), cfg.show_http_debug)
    };
    let script = http_path.join("server.js");
    if !script.exists() {
        return;
    }
    let exec = exec.trim_end_matches(".exe").to_string();
    let mut cmd = std::process::Command::new(exec);
    cmd.arg(&script)
        .current_dir(&http_path)
        .creation_flags(if debug { 0 } else { 0x08000000 });
    let _ = cmd.spawn();
}

/// /version 连接检查（对应 CheckServer / ServerCore）
fn check_server(shared: &Arc<Shared>) -> bool {
    let url = { format!("{}/version", shared.cfg.read().unwrap().server_url()) };
    match post_json(&url, "", 3000) {
        Ok(text) if text.contains("showKeyBoardServer Version:") => {
            shared.server_state.store(1, Ordering::Relaxed);
            true
        }
        _ => {
            shared.server_state.store(0, Ordering::Relaxed);
            false
        }
    }
}

fn is_server_ok(shared: &Arc<Shared>) -> bool {
    shared.server_state.load(Ordering::Relaxed) == 1
}

/// 网络线程
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

            let mut check_count = 0;
            let mut last_send = Instant::now() - Duration::from_secs(10);
            let mut fail_count = 0;
            let mut started_server = false;

            loop {
                if shared.exit.load(Ordering::Relaxed) {
                    break;
                }

                // IniMonitor 热重载
                if shared.ws_reload.swap(false, Ordering::Relaxed) {
                    reload_config(&shared);
                }

                // PCInfo 请求
                let pc_flag = shared.pcinfo_dirty.swap(0, Ordering::Relaxed);
                if pc_flag != 0 && is_server_ok(&shared) {
                    send_pc_info(&shared, pc_flag);
                }

                if !is_server_ok(&shared) {
                    // 未连接：重试 /version，超过次数尝试启动 node 服务
                    if check_count < shared.cfg.read().unwrap().check_server_max {
                        check_count += 1;
                        if check_server(&shared) {
                            send_pc_info(&shared, 0);
                            check_count = 0;
                            continue;
                        }
                    } else if !started_server {
                        started_server = true;
                        start_node_server(&shared);
                        std::thread::sleep(Duration::from_secs(1));
                        if check_server(&shared) {
                            send_pc_info(&shared, 0);
                            check_count = 0;
                            continue;
                        }
                    }
                    std::thread::sleep(Duration::from_secs(3));
                    continue;
                }

                // 已连接：按 1 秒限频发送 /data
                let dirty = shared.stats.lock().unwrap().dirty;
                if dirty && last_send.elapsed().as_millis() >= 1000 {
                    let body = build_data_body(&shared).to_string();
                    let url = format!("{}/data", shared.cfg.read().unwrap().server_url());
                    shared.stats.lock().unwrap().dirty = false;
                    last_send = Instant::now();
                    if post_json(&url, &body, 8000).is_ok() {
                        fail_count = 0;
                    } else {
                        fail_count += 1;
                        // maxCountOfConnectFail：连续失败超过该次数才判定掉线
                        let max_fail = shared.cfg.read().unwrap().max_count_of_connect_fail;
                        if fail_count >= max_fail {
                            shared.server_state.store(-1, Ordering::Relaxed);
                            check_count = 0;
                            started_server = false;
                        }
                    }
                }

                std::thread::sleep(Duration::from_millis(200));
            }
        })
        .expect("spawn net thread")
}

/// 配置热重载（IniMonitor(1)）：重新读取 ini，并同步钩子参数
fn reload_config(shared: &Arc<Shared>) {
    let mut cfg = shared.cfg.write().unwrap();
    let new_cfg = crate::config::Config::load();
    // needShowKey 的运行时开关不丢失
    let need_show = cfg.need_show_key;
    let mut new_cfg = new_cfg;
    new_cfg.need_show_key = need_show;
    new_cfg.generation = cfg.generation + 1;
    *cfg = new_cfg;
    drop(cfg);
    crate::input::refresh_hook_params();
}

/// WebSocket 线程：ahkClient 身份，接收 IniMonitor
pub fn start_ws(shared: Arc<Shared>) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("ws".into())
        .spawn(move || loop {
            if shared.exit.load(Ordering::Relaxed) {
                break;
            }
            if shared.server_state.load(Ordering::Relaxed) != 1 {
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
            let url = { shared.cfg.read().unwrap().server_ws_url() };
            match tungstenite::connect(&url) {
                Ok((mut ws, _)) => {
                    use tungstenite::stream::MaybeTlsStream;
                    use tungstenite::Message;
                    let _ = ws.send(Message::Text("ahkClient".into()));
                    debug_out("WS send ahkClient");
                    if let MaybeTlsStream::Plain(tcp) = ws.get_mut() {
                        tcp.set_read_timeout(Some(Duration::from_secs(5))).ok();
                    }
                    loop {
                        if shared.exit.load(Ordering::Relaxed) {
                            break;
                        }
                        match ws.read() {
                            Ok(Message::Text(text)) => {
                                if text.trim() == "IniMonitor" {
                                    shared.ws_reload.store(true, Ordering::Relaxed);
                                }
                            }
                            Ok(Message::Ping(_)) => {}
                            Ok(_) => {}
                            Err(tungstenite::Error::Io(ref e))
                                if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut =>
                            {
                                continue;
                            }
                            Err(_) => break,
                        }
                    }
                }
                Err(_) => {
                    std::thread::sleep(Duration::from_secs(3));
                }
            }
            std::thread::sleep(Duration::from_millis(300));
        })
        .expect("spawn ws thread")
}

/// 退出时发送 /exit（ExitServer）
pub fn send_exit(shared: &Arc<Shared>) {
    if is_server_ok(shared) {
        let url = format!("{}/exit", shared.cfg.read().unwrap().server_url());
        let _ = post_json(&url, "", 3000);
    }
}
