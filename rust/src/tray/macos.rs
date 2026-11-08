//! macOS 托盘（tray-icon 封装的 NSStatusItem）。
//!
//! NSStatusItem / NSMenu 必须在主线程创建，而 Slint 的事件循环已经占用了主线程，
//! 因此这里不做独立线程 + 消息循环，而是由 UI 定时器每帧调用 `poll()`：
//! 首次调用时惰性创建托盘，之后轮询 MenuEvent 分发菜单动作。
#![allow(dead_code)]
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::stats::Shared;

/// 编译期嵌入托盘图标（对应 AHK 的 res\keyboard.ico）
const EMBEDDED_ICON: &[u8] = include_bytes!("../../../res/keyboard.ico");

const ID_STARTUP: &str = "skb.startup";
const ID_RESTART: &str = "skb.restart";
const ID_RESTART_ALL: &str = "skb.restart_all";
const ID_SETTINGS: &str = "skb.settings";
const ID_STAT: &str = "skb.stat";
const ID_SHOW: &str = "skb.show";
const ID_EXIT: &str = "skb.exit";

const AGENT_LABEL: &str = "com.showkeyboard.client";

struct TrayState {
    _tray: TrayIcon,
    startup_on: bool,
    mi_startup: CheckMenuItem,
    mi_show: CheckMenuItem,
    last_gen: u64,
}

thread_local! {
    static STATE: RefCell<Option<TrayState>> = const { RefCell::new(None) };
}

// ---------------- 开机启动（LaunchAgents） ----------------

fn launch_agent_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| {
        PathBuf::from(h)
            .join("Library")
            .join("LaunchAgents")
            .join(format!("{}.plist", AGENT_LABEL))
    })
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn startup_set(enable: bool) {
    let Some(path) = launch_agent_path() else { return };
    if enable {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let exe = std::env::current_exe().unwrap_or_default();
        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#,
            label = AGENT_LABEL,
            exe = xml_escape(&exe.to_string_lossy()),
        );
        let _ = std::fs::write(path, plist);
    } else {
        let _ = std::fs::remove_file(path);
    }
}

fn startup_checked() -> bool {
    launch_agent_path().map(|p| p.exists()).unwrap_or(false)
}

// ---------------- 托盘 ----------------

fn create(shared: &Arc<Shared>) -> Option<TrayState> {
    let (lang_en, need_show, hotkey, only_web) = {
        let c = shared.cfg.read().unwrap();
        (
            c.default_lang == "en-US",
            c.need_show_key,
            c.hotkey4_show.clone(),
            c.ui.show_key_only_web,
        )
    };
    let (t_set, t_stat, t_show, t_startup, t_restart, t_restart_all, t_exit) = if lang_en {
        (
            "Setting",
            "Statistics",
            "Show/Hide Key",
            "Auto Start",
            "Reset Client",
            "Full Reset",
            "Exit",
        )
    } else {
        (
            "参数设置",
            "数据统计",
            "按键显示",
            "开机启动",
            "重启程序",
            "完全重启",
            "退出程序",
        )
    };
    // 按键显示 + 热键标记（对应 AHK showOnlyWebTxt / L_menu_4show）
    let mark = if only_web {
        format!("Web {}", hotkey.trim())
    } else {
        hotkey.trim().to_string()
    };
    let show_label = if mark.is_empty() {
        t_show.to_string()
    } else {
        format!("{} {}", t_show, mark)
    };
    let tip = format!("ShowKeyBoard v{}", crate::stats::env_full_version());

    let menu = Menu::new();
    let startup_on = startup_checked();
    let mi_startup = CheckMenuItem::with_id(ID_STARTUP, t_startup, true, startup_on, None);
    let mi_restart = MenuItem::with_id(ID_RESTART, t_restart, true, None);
    let mi_restart_all = MenuItem::with_id(ID_RESTART_ALL, t_restart_all, true, None);
    let mi_settings = MenuItem::with_id(ID_SETTINGS, t_set, true, None);
    let mi_stat = MenuItem::with_id(ID_STAT, t_stat, true, None);
    let mi_show = CheckMenuItem::with_id(ID_SHOW, show_label, true, need_show, None);
    let sep1 = PredefinedMenuItem::separator();
    let sep2 = PredefinedMenuItem::separator();
    let sep3 = PredefinedMenuItem::separator();
    let mi_exit = MenuItem::with_id(ID_EXIT, t_exit, true, None);
    let _ = menu.append(&mi_startup);
    let _ = menu.append(&sep1);
    let _ = menu.append(&mi_restart);
    let _ = menu.append(&mi_restart_all);
    let _ = menu.append(&sep2);
    let _ = menu.append(&mi_show);
    let _ = menu.append(&mi_settings);
    let _ = menu.append(&mi_stat);
    let _ = menu.append(&sep3);
    let _ = menu.append(&mi_exit);

    // 图标：优先同目录 ShowKeyBoard.ico / keyboard.ico，否则使用编译期嵌入的 res/keyboard.ico
    let icon: Option<tray_icon::Icon> = {
        let dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default();
        let mut bytes: Option<Vec<u8>> = None;
        for name in ["ShowKeyBoard.ico", "keyboard.ico"] {
            if let Ok(b) = std::fs::read(dir.join(name)) {
                bytes = Some(b);
                break;
            }
        }
        let bytes = bytes.unwrap_or_else(|| EMBEDDED_ICON.to_vec());
        crate::icon::ico_to_rgba(&bytes, 22)
            .and_then(|(rgba, w, h)| tray_icon::Icon::from_rgba(rgba, w, h).ok())
    };

    let mut builder = TrayIconBuilder::new().with_tooltip(tip).with_menu(Box::new(menu));
    if let Some(ic) = icon {
        builder = builder.with_icon(ic);
    }
    let tray = builder.build().ok()?;

    Some(TrayState {
        _tray: tray,
        startup_on,
        mi_startup,
        mi_show,
        last_gen: shared.show_key_generation.load(Ordering::Relaxed),
    })
}

/// UI 定时器调用：惰性创建托盘并处理菜单事件（必须在主线程执行）
pub fn poll(shared: &Arc<Shared>) {
    STATE.with(|cell| {
        if cell.borrow().is_none() {
            match create(shared) {
                Some(st) => *cell.borrow_mut() = Some(st),
                // 托盘创建失败（例如没有 GUI 会话）时下帧再试
                None => return,
            }
        }
        let mut guard = cell.borrow_mut();
        let Some(st) = guard.as_mut() else { return };

        // hotkey4Show 从输入线程切换 needShowKey 时，同步勾选态
        let gen = shared.show_key_generation.load(Ordering::Relaxed);
        if gen != st.last_gen {
            st.last_gen = gen;
            let checked = shared.cfg.read().unwrap().need_show_key;
            st.mi_show.set_checked(checked);
        }

        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            match ev.id().as_ref() {
                ID_STARTUP => {
                    startup_set(!st.startup_on);
                    st.startup_on = startup_checked();
                    st.mi_startup.set_checked(st.startup_on);
                }
                ID_RESTART => {
                    // 重启程序：只重启客户端，后台服务不退出
                    shared.restart_mode.store(1, Ordering::Relaxed);
                    shared.exit.store(true, Ordering::Relaxed);
                    let _ = slint::quit_event_loop();
                }
                ID_RESTART_ALL => {
                    // 完全重启：先通知后台服务退出（HTTP 请求放后台线程）
                    let sh = shared.clone();
                    std::thread::spawn(move || {
                        crate::net::send_exit(&sh);
                        // 留出后台服务退出/释放端口的时间（AHK 中为 Sleep(200)）
                        std::thread::sleep(std::time::Duration::from_millis(600));
                        sh.restart_mode.store(2, Ordering::Relaxed);
                        sh.exit.store(true, Ordering::Relaxed);
                        let _ = slint::quit_event_loop();
                    });
                }
                ID_SETTINGS => {
                    let url = format!("{}/Setting", shared.cfg.read().unwrap().server_url());
                    crate::winutil::open_url(&url);
                }
                ID_STAT => {
                    let url = format!("{}/Today", shared.cfg.read().unwrap().server_url());
                    crate::winutil::open_url(&url);
                }
                ID_SHOW => {
                    let mut cfg = shared.cfg.write().unwrap();
                    cfg.need_show_key = !cfg.need_show_key;
                    let new_state = cfg.need_show_key;
                    drop(cfg);
                    shared.show_key_generation.fetch_add(1, Ordering::Relaxed);
                    st.mi_show.set_checked(new_state);
                }
                ID_EXIT => {
                    shared.exit.store(true, Ordering::Relaxed);
                    let _ = slint::quit_event_loop();
                }
                _ => {}
            }
        }
    });
}
