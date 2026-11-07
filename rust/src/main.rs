//! ShowKeyBoard Rust 客户端入口
//! 功能对齐 showKeyBoard.ahk：
//! - 全局键鼠捕获（合并 hook/getKeyInput，无 WM_COPYDATA）
//! - 手柄（XInput / joyGetPosEx）
//! - 分钟统计 / 按键统计 / 历史记录
//! - HTTP + WebSocket 与后台通讯（/data /version /sendPCInfo /exit，IniMonitor 热重载）
//! - Slint 浮窗 UI（渐隐、自定义字体背景、位置方向、多屏）
//! - 托盘菜单、hotkey4Show 热键、单实例、kb.pid、lastRecord.json、自动版本号
#![cfg(windows)]
// 与 AHK 编译版一致：GUI 子系统，无控制台窗口（否则任务栏会出现控制台按钮）
#![windows_subsystem = "windows"]
#![allow(dead_code)]

mod config;
mod input;
mod joy;
mod net;
mod stats;
mod tray;
mod ui;
mod winutil;

use std::sync::atomic::Ordering;
use std::sync::Arc;

fn main() {
    // 工作目录切换到 exe 所在目录（与 AHK A_ScriptDir / A_WorkingDir 一致）
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    if let Some(dir) = &exe_dir {
        let _ = std::env::set_current_dir(dir);
    }

    // 单实例（#SingleInstance Ignore）
    if config::SingleInstance::acquire("ShowKeyBoardRustSingleInstance").is_none() {
        // 已有实例运行，直接退出
        return;
    }
    config::write_pid_file(
        &std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    );

    // 在创建任何窗口之前指定 winit 窗口属性：popup 方式（带 owner + 不显示在任务栏），
    // 这样启动时预热的多个窗口不会在任务栏冒出多个按钮。
    // 该 owner 窗口同时用于托盘气泡提示（needTraytip）。
    let owner_hwnd = winutil::owner_hwnd() as isize;
    {
        // 保留 SLINT_BACKEND=winit-software 的调试入口
        let name = match std::env::var("SLINT_BACKEND") {
            Ok(v) if v.contains("winit") => v,
            _ => "winit".to_string(),
        };
        let _ = slint::BackendSelector::new()
            .backend_name(name)
            .with_winit_window_attributes_hook(move |attributes| {
                use slint::winit_030::winit::platform::windows::WindowAttributesExtWindows;
                attributes.with_skip_taskbar(true).with_owner_window(owner_hwnd)
            })
            .select();
    }

    let cfg = config::Config::load();
    let (tx, rx) = crossbeam_channel::unbounded::<String>();
    let shared = stats::Shared::new(cfg, tx);

    // 键鼠钩子线程（合并 getKeyInput 功能）
    let hook_thread = input::start(shared.clone());

    // 轮询线程：鼠标距离 + 手柄 + 分钟刷新
    let poll_thread = joy::start(shared.clone());

    // 网络线程
    let net_thread = net::start(shared.clone());
    let ws_thread = net::start_ws(shared.clone());

    // 托盘
    let tray_thread = tray::start(shared.clone(), owner_hwnd);

    // UI 主循环（阻塞）
    ui::run(shared.clone(), rx);

    // 兜底看门狗：任何线程卡在系统调用上时，5 秒后强制退出，避免进程残留
    // （正常情况下各线程都能在毫秒级退出，看门狗不会触发）
    let _ = std::thread::Builder::new()
        .name("exit-watchdog".into())
        .spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(5));
            std::process::exit(0);
        });

    // 退出清理（AHK ExitFunc）
    exit_cleanup(&shared);

    shared.exit.store(true, Ordering::Relaxed);
    let _ = hook_thread.join();
    let _ = poll_thread.join();
    let _ = net_thread.join();
    let _ = ws_thread.join();
    let _ = tray_thread.join();
}

/// 对应 AHK ExitFunc：保存 needShowKey、写出 lastRecord.json、通知服务端退出
fn exit_cleanup(shared: &Arc<stats::Shared>) {
    let cfg = shared.cfg.read().unwrap().clone();

    // 1) needShowKey 运行时开关写回 ini
    let ini = config::Ini::load(&cfg.ini_path);
    let file_val = ini.get("common", "needShowKey").map(|s| s.to_string());
    let current = if cfg.need_show_key { "1" } else { "0" };
    if file_val.as_deref() != Some(current) {
        write_ini_value(&cfg.ini_path, "common", "needShowKey", current);
    }

    // 2) 若存在 kbserver.pid，写出 lastRecord.json（后端重启后优先载入）
    let pid_path = cfg.http_path.join("kbserver.pid");
    if pid_path.exists() {
        // 最后一条分钟数据转差值并标注 LastFlag（与 AHK 一致）
        {
            let mut stats = shared.stats.lock().unwrap();
            let g_mouse = stats.global_mouse_count;
            let g_key = stats.global_key_count;
            let g_dist = stats.mouse_distance;
            if let Some(last) = stats.minute_recs.last_mut() {
                last.mouse_count = g_mouse - last.mouse_count;
                last.key_count = g_key - last.key_count;
                last.distance = g_dist - last.distance;
                last.last_flag = Some(1);
            }
        }
        let body = net::build_data_body(shared).to_string();
        let last_path = cfg.http_path.join("lastRecord.json");
        let _ = std::fs::write(last_path, body);
    }

    // 3) 通知后台退出
    net::send_exit(shared);
}

/// 追加/更新 INI 中的键值（简单实现：重写文件）
fn write_ini_value(path: &std::path::Path, section: &str, key: &str, value: &str) {
    let mut content = match std::fs::read(path) {
        Ok(bytes) => {
            if let Ok(s) = String::from_utf8(bytes.clone()) {
                s
            } else {
                let (cow, _, _) = encoding_rs::GBK.decode(&bytes);
                cow.into_owned()
            }
        }
        Err(_) => String::new(),
    };

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut in_section = false;
    let mut replaced = false;
    for line in lines.iter_mut() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            in_section = t[1..t.len() - 1].trim().eq_ignore_ascii_case(section);
            continue;
        }
        if in_section && t.starts_with(key) && t[key.len()..].trim_start().starts_with('=') {
            *line = format!("{}={}", key, value);
            replaced = true;
        }
    }
    if !replaced {
        if !lines.iter().any(|l| {
            let t = l.trim();
            t.starts_with('[') && t.ends_with(']') && t[1..t.len() - 1].trim().eq_ignore_ascii_case(section)
        }) {
            lines.push(format!("[{}]", section));
        }
        lines.push(format!("{}={}", key, value));
    }
    content = lines.join("\r\n");
    // INI 以 ANSI(GBK) 保存（与 Node iniAnsiWrite 一致）
    let (enc, _, _) = encoding_rs::GBK.encode(&content);
    let _ = std::fs::write(path, enc);
}
