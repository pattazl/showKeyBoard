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

use crate::i18n;
use crate::stats::Shared;

/// 编译期嵌入托盘图标（对应 AHK 的 res\keyboard.ico）
const EMBEDDED_ICON: &[u8] = include_bytes!("../../../res/keyboard.ico");

const ID_STARTUP: &str = "skb.startup";
const ID_RESTART: &str = "skb.restart";
const ID_SETTINGS: &str = "skb.settings";
const ID_STAT: &str = "skb.stat";
const ID_SHOW: &str = "skb.show";
const ID_VERSION: &str = "skb.version";
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
    let Some(path) = launch_agent_path() else { return false };
    if !path.exists() {
        return false;
    }
    // 主程序改名后，迁移仍指向旧文件名的 LaunchAgent。
    if std::fs::read_to_string(&path)
        .map(|s| s.contains("showkeyboard_client"))
        .unwrap_or(false)
    {
        startup_set(true);
    }
    true
}

// ---------------- 启动通知 ----------------

/// 启动气泡提示（needTraytip）：macOS 用本地通知展示与 Windows 气泡等价的内容。
/// 必须在主线程调用（UI 定时器内首次创建托盘时），NSUserNotificationCenter
/// 属于 Foundation，已随 cocoa 链接，无需额外框架或授权代码。
fn notify_started(shared: &Arc<Shared>) {
    if !shared.cfg.read().unwrap().need_traytip {
        return;
    }
    let (lang, hotkey) = {
        let c = shared.cfg.read().unwrap();
        (
            i18n::Lang::from_ini(&c.default_lang),
            c.hotkey4_show.clone(),
        )
    };
    let text = i18n::balloon_started(lang, &crate::stats::env_full_version(), &hotkey);
    post_notification("ShowKeyBoard", &text);
}

fn post_notification(title: &str, text: &str) {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let center: id =
            msg_send![class!(NSUserNotificationCenter), defaultUserNotificationCenter];
        if center == nil {
            return;
        }
        let note: id = msg_send![class!(NSUserNotification), alloc];
        if note == nil {
            return;
        }
        let note: id = msg_send![note, init];
        let title: id = NSString::alloc(nil).init_str(title);
        let body: id = NSString::alloc(nil).init_str(text);
        let _: () = msg_send![note, setTitle: title];
        let _: () = msg_send![note, setInformativeText: body];
        let _: () = msg_send![center, deliverNotification: note];
        let _: () = msg_send![note, release];
        let _: () = msg_send![title, release];
        let _: () = msg_send![body, release];
    }
}

// ---------------- 托盘 ----------------

fn create(shared: &Arc<Shared>) -> Option<TrayState> {
    let (lang, need_show, hotkey, only_web) = {
        let c = shared.cfg.read().unwrap();
        (
            i18n::Lang::from_ini(&c.default_lang),
            c.need_show_key,
            c.hotkey4_show.clone(),
            c.ui.show_key_only_web,
        )
    };
    let t = i18n::menu_texts(lang);
    // 按键显示 + 热键标记（对应 AHK showOnlyWebTxt / L_menu_4show）
    let mark = if only_web {
        format!("Web {}", hotkey.trim())
    } else {
        hotkey.trim().to_string()
    };
    let show_label = if mark.is_empty() {
        t.show_key.to_string()
    } else {
        format!("{} {}", t.show_key, mark)
    };
    let tip = format!("ShowKeyBoard v{}", crate::stats::env_full_version());

    let menu = Menu::new();
    let startup_on = startup_checked();
    let mi_startup = CheckMenuItem::with_id(ID_STARTUP, t.auto_start, true, startup_on, None);
    let mi_restart = MenuItem::with_id(ID_RESTART, t.restart, true, None);
    let mi_settings = MenuItem::with_id(ID_SETTINGS, t.settings, true, None);
    let mi_stat = MenuItem::with_id(ID_STAT, t.statistics, true, None);
    let mi_show = CheckMenuItem::with_id(ID_SHOW, show_label, true, need_show, None);
    let sep1 = PredefinedMenuItem::separator();
    let sep2 = PredefinedMenuItem::separator();
    let sep3 = PredefinedMenuItem::separator();
    let mi_exit = MenuItem::with_id(ID_EXIT, t.exit, true, None);
    let _ = menu.append(&mi_startup);
    let _ = menu.append(&sep1);
    let _ = menu.append(&mi_restart);
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
                Some(st) => {
                    *cell.borrow_mut() = Some(st);
                    // 托盘就绪后发启动通知（needTraytip）
                    notify_started(shared);
                }
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
                    // 重启程序：重启整个客户端进程（内嵌服务端随进程一起重启）
                    shared.restart.store(true, Ordering::Relaxed);
                    shared.exit.store(true, Ordering::Relaxed);
                    let _ = slint::quit_event_loop();
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
                    // 切换 needShowKey，并立即写回 ini（与 Web 设置界面双向同步）
                    let (new_state, ini_path) = {
                        let mut cfg = shared.cfg.write().unwrap();
                        cfg.need_show_key = !cfg.need_show_key;
                        (cfg.need_show_key, cfg.ini_path.clone())
                    };
                    let value = if new_state { "1" } else { "0" };
                    crate::config::write_ini_value(&ini_path, "common", "needShowKey", value);
                    crate::net::notify_server_config("common", "needShowKey", value);
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
