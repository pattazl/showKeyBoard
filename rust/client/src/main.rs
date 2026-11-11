//! ShowKeyBoard Rust 客户端入口
//! 功能对齐 showKeyBoard.ahk：
//! - 全局键鼠捕获（合并 hook/getKeyInput，无 WM_COPYDATA）
//! - 手柄（XInput / joyGetPosEx）
//! - 分钟统计 / 按键统计 / 历史记录
//! - HTTP + WebSocket 与后台通讯（/data /version /sendPCInfo /exit，IniMonitor 热重载）
//! - Slint 浮窗 UI（渐隐、自定义字体背景、位置方向、多屏）
//! - 托盘菜单、hotkey4Show 热键、单实例、kb.pid、lastRecord.json、自动版本号
//!
//! 跨平台：Windows（Win32 钩子 / XInput / Win32 托盘）与 macOS（CGEventTap / gilrs / NSStatusItem）。
// 与 AHK 编译版一致：GUI 子系统，无控制台窗口（否则任务栏会出现控制台按钮）
#![cfg_attr(windows, windows_subsystem = "windows")]
#![allow(dead_code)]

mod config;
mod icon;
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

    // macOS：切为 Accessory（菜单栏应用），Dock/任务栏不显示程序图标，
    // 与 Windows 侧 windows_subsystem="windows"（GUI 子系统，无任务栏按钮）对齐；
    // NSStatusItem 托盘与全局 CGEventTap 不受影响
    #[cfg(target_os = "macos")]
    winutil::set_dock_hidden();

    // 单实例（#SingleInstance Ignore）
    if config::SingleInstance::acquire("ShowKeyBoardRustSingleInstance").is_none() {
        // 已有实例运行，直接退出
        return;
    }
    config::write_pid_file(&config::data_dir());

    // 在创建任何窗口之前指定 winit 窗口属性：popup 方式（带 owner + 不显示在任务栏），
    // 这样启动时预热的多个窗口不会在任务栏冒出多个按钮。
    // 该 owner 窗口同时用于托盘气泡提示（needTraytip）。
    // macOS 没有 owner/taskbar 概念，窗口层级与鼠标穿透由 winutil 在运行时设置。
    #[cfg(windows)]
    let owner_hwnd = winutil::owner_hwnd() as isize;
    #[cfg(not(windows))]
    let _owner_hwnd: isize = 0;
    {
        // 保留 SLINT_BACKEND=winit-software 的调试入口
        let name = match std::env::var("SLINT_BACKEND") {
            Ok(v) if v.contains("winit") => v,
            _ => "winit".to_string(),
        };
        let selector = slint::BackendSelector::new().backend_name(name);
        #[cfg(windows)]
        let selector = selector.with_winit_window_attributes_hook(move |attributes| {
            use slint::winit_030::winit::platform::windows::WindowAttributesExtWindows;
            // with_active(false)：首次 show() 走 ShowWindow(SW_SHOWNOACTIVATE)，
            // 所有浮窗（含控制键状态窗）即使置顶也绝不抢占前台焦点。
            attributes
                .with_skip_taskbar(true)
                .with_owner_window(owner_hwnd)
                .with_active(false)
        });
        let _ = selector.select();
    }

    let cfg = config::Config::load();
    // macOS：默认无控制台输出（GUI 应用，与 Windows 子系统对齐）；
    // showHttpDebug 开启时把输出写入数据目录 debug.log 并打开 Terminal 实时查看
    #[cfg(target_os = "macos")]
    if cfg.show_http_debug {
        winutil::show_debug_console(&cfg.data_dir.join("debug.log"));
    } else {
        winutil::silence_stdio();
    }
    let (tx, rx) = crossbeam_channel::unbounded::<String>();
    // 实时按键文本管道（UI -> WebSocket 线程，有界：业务高峰时丢弃而不是拖慢 UI）
    let (ws_tx, ws_rx) = crossbeam_channel::bounded::<String>(256);
    let shared = stats::Shared::new(cfg, tx, ws_tx);

    // 键鼠钩子线程（合并 getKeyInput 功能）
    let hook_thread = input::start(shared.clone());

    // 轮询线程：鼠标距离 + 手柄 + 分钟刷新
    let poll_thread = joy::start(shared.clone());

    // 网络线程
    let net_thread = net::start(shared.clone());
    let ws_thread = net::start_ws(shared.clone(), ws_rx);

    // 跨夜自动重启（对应 AHK 的 SetTimer(Reload, ShutDownLeft())）：
    // 每天本地 0:00:05 触发一次“重启程序”，与托盘 ID_RESTART 行为一致（只重启客户端，后台服务不退出）。
    // 这里只置标志位，由 UI 定时器在主线程退出事件循环，避免跨线程操作事件循环。
    // 线程为游离线程（不 join）：正常退出时它可能还在 sleep，进程结束时随主线程一起结束。
    let _midnight_thread = {
        let shared = shared.clone();
        std::thread::Builder::new()
            .name("midnight-restart".into())
            .spawn(move || {
                let start_date = winutil::now_string(true)[..8].to_string();
                loop {
                    if shared.exit.load(Ordering::Relaxed) {
                        return;
                    }
                    let now = winutil::now_string(false);
                    let secs = if &now[..8] != start_date.as_str() {
                        // 日期已变即已跨夜；等到 0:00:05 之后才重启（对应 AHK 的 +5 秒余量）
                        if &now[8..] >= "000005" {
                            shared.restart_mode.store(1, Ordering::Relaxed);
                            shared.exit.store(true, Ordering::Relaxed);
                            return;
                        }
                        1
                    } else {
                        // 分段睡眠（每段最多 60 秒），最后一段直接算到 0:00:05：
                        // 既保证准点，也能在系统休眠/校时/夏令时后按本地时间重新判定
                        winutil::secs_until_next_restart().min(60)
                    };
                    std::thread::sleep(std::time::Duration::from_secs(secs));
                }
            })
            .ok()
    };

    // 托盘：Windows 用独立线程消息循环；macOS 由 ui 主线程惰性创建（NSStatusItem 要求主线程）
    #[cfg(windows)]
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
    #[cfg(windows)]
    let _ = tray_thread.join();

    // 重启（托盘菜单）：释放单实例互斥后拉起新进程
    if shared.restart_mode.load(Ordering::Relaxed) != 0 {
        config::release_single_instance();
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe).spawn();
        }
    }
}

/// 对应 AHK ExitFunc：保存 needShowKey、写出 lastRecord.json、通知服务端退出
fn exit_cleanup(shared: &Arc<stats::Shared>) {
    let cfg = shared.cfg.read().unwrap().clone();

    // 1) needShowKey 运行时开关写回 ini
    let ini = config::Ini::load(&cfg.ini_path);
    let file_val = ini.get("common", "needShowKey").map(|s| s.to_string());
    let current = if cfg.need_show_key { "1" } else { "0" };
    if file_val.as_deref() != Some(current) {
        config::write_ini_value(&cfg.ini_path, "common", "needShowKey", current);
    }

    // 2) 若存在 kbserver.pid，写出 lastRecord.json（后端重启后优先载入）
    let pid_path = cfg.data_dir.join("kbserver.pid");
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
        let last_path = cfg.data_dir.join("lastRecord.json");
        let _ = std::fs::write(last_path, body);
    }

    // 3) 通知后台退出：重启程序时保留后台服务；完全重启已由托盘菜单先行通知
    if shared.restart_mode.load(Ordering::Relaxed) == 0 {
        net::send_exit(shared);
    }
}
