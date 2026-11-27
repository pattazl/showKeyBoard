//! ShowKeyBoard Rust 客户端入口
//! 功能对齐 showKeyBoard.ahk：
//! - 全局键鼠捕获（合并 hook/getKeyInput，无 WM_COPYDATA）
//! - 手柄（XInput / joyGetPosEx）
//! - 分钟统计 / 按键统计 / 历史记录
//! - 内嵌 HTTP/WS 服务端（同进程，原 showkeyboard_server 迁入）：进程内调用替代外发 HTTP/WS
//! - Slint 浮窗 UI（渐隐、自定义字体背景、位置方向、多屏）
//! - 托盘菜单、hotkey4Show 热键、单实例、kb.pid、lastRecord.json、自动版本号
//!
//! 跨平台：Windows（Win32 钩子 / XInput / Win32 托盘）与 macOS（CGEventTap / gilrs / NSStatusItem）。
// 与 AHK 编译版一致：GUI 子系统，无控制台窗口（否则任务栏会出现控制台按钮）
#![cfg_attr(windows, windows_subsystem = "windows")]
#![allow(dead_code)]

mod config;
mod crashlog;
mod i18n;
mod icon;
mod input;
mod joy;
mod net;
mod server;
mod stats;
mod tray;
mod ui;
mod winutil;

use std::sync::atomic::Ordering;
use std::sync::Arc;

fn main() {
    // 由“重启程序”拉起的子进程带 --restart：端口沿用原值，不做顺延
    let restarted = std::env::args().any(|a| a == "--restart");
    // 工作目录切换到 exe 所在目录（与 AHK A_ScriptDir / A_WorkingDir 一致）
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    if let Some(dir) = &exe_dir {
        let _ = std::env::set_current_dir(dir);
    }

    // 崩溃现场记录：异常码/故障地址/寄存器/栈返回地址写入数据目录 crash.log
    crashlog::install();

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

    // 一个永不显示的 owner 窗口：托盘气泡提示（needTraytip）需要 hwnd 作为回调接收方
    #[cfg(windows)]
    let owner_hwnd = winutil::owner_hwnd() as isize;
    #[cfg(not(windows))]
    let _owner_hwnd: isize = 0;

    let cfg = config::Config::load();
    // macOS：GUI 应用无控制台输出，与 Windows 子系统对齐
    #[cfg(target_os = "macos")]
    winutil::silence_stdio();
    let (tx, rx) = crossbeam_channel::unbounded::<String>();
    // 实时按键文本管道（UI -> WebSocket 线程，有界：业务高峰时丢弃而不是拖慢 UI）
    let (ws_tx, ws_rx) = crossbeam_channel::bounded::<String>(256);
    let shared = stats::Shared::new(cfg, tx, ws_tx);

    // 键鼠钩子线程（合并 getKeyInput 功能）
    let hook_thread = input::start(shared.clone());

    // 轮询线程：鼠标距离 + 手柄 + 分钟刷新
    let poll_thread = joy::start(shared.clone());

    // 启动内嵌 HTTP/WS 服务端（同进程）：先建好服务端→客户端的配置通道，
    // 服务端在 set_para 变更配置后通过它推送给客户端。
    let (cfg_tx, cfg_rx) = crossbeam_channel::unbounded::<serde_json::Value>();
    match server::start(cfg_tx, !restarted) {
        Ok(port) => {
            // 端口冲突重试时，把实际绑定端口同步回客户端配置，
            // 保证托盘打开 Setting/Today 等 URL 指向正确端口。
            let mut c = shared.cfg.write().unwrap();
            if c.server_port != port {
                c.server_port = port;
            }
        }
        Err(e) => {
            eprintln!("embedded server start failed: {}", e);
            // 端口连续顺延仍无法绑定：GUI 程序无控制台，用系统模态框告知用户
            // （与 AHK 版一致：仅提示，客户端本体继续运行）
            let lang = i18n::Lang::from_ini(&shared.cfg.read().unwrap().default_lang);
            winutil::show_error_dialog("ShowKeyBoard", &i18n::server_start_failed(lang, &e));
        }
    }

    // 网络线程：进程内上报 /data 与 /sendPCInfo（原 HTTP 调用）
    let net_thread = net::start(shared.clone());
    // Bridge 线程：实时按键文本广播给 ahkKeyShow + 应用服务端推送的配置
    let bridge_thread = net::start_bridge(shared.clone(), ws_rx, cfg_rx);

    // 跨夜自动重启（对应 AHK 的 SetTimer(Reload, ShutDownLeft())）：
    // 每天本地 0:00:05 触发一次“重启程序”，与托盘 ID_RESTART 行为一致（重启整个客户端进程）。
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
                            shared.restart.store(true, Ordering::Relaxed);
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

    // 兜底看门狗：任何线程卡在系统调用上时，10 秒后强制退出，避免进程残留
    // （正常情况下各线程都能在毫秒级退出；重启前还要等待服务端释放端口，故留足余量）
    let _ = std::thread::Builder::new()
        .name("exit-watchdog".into())
        .spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(10));
            std::process::exit(0);
        });

    // 退出清理（AHK ExitFunc）
    exit_cleanup(&shared);

    shared.exit.store(true, Ordering::Relaxed);
    let _ = hook_thread.join();
    let _ = poll_thread.join();
    let _ = net_thread.join();
    let _ = bridge_thread.join();
    #[cfg(windows)]
    let _ = tray_thread.join();

    // 重启/退出前先优雅关闭内嵌 HTTP/WS 服务并等待监听端口释放。
    // 否则新进程启动时旧监听仍占用端口，绑定失败会顺延到下一个端口（端口漂移）。
    server::shutdown();

    // 重启（托盘菜单 / 跨夜）：释放单实例互斥后拉起新进程
    if shared.restart.load(Ordering::Relaxed) {
        config::release_single_instance();
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe).arg("--restart").spawn();
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

    // 3) 退出前让内嵌服务端把内存 preData 落库（服务端与客户端同进程，退出即一起结束）
    server::save_last_data();
}
