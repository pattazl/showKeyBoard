//! 托盘图标与菜单（对应 AHK CreateMenu/MenuHandler）
#![allow(dead_code)]
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::stats::Shared;

/// 编译期嵌入托盘图标（对应 AHK 的 res\keyboard.ico）
const EMBEDDED_ICON: &[u8] = include_bytes!("../../res/keyboard.ico");

const ID_SETTINGS: &str = "skb.settings";
const ID_STAT: &str = "skb.stat";
const ID_SHOW: &str = "skb.show";
const ID_RELOAD: &str = "skb.reload";
const ID_EXIT: &str = "skb.exit";

/// 气泡提示图标的 uID（与 tray-icon 自身的图标互不冲突）
const BALLOON_UID: u32 = 0xB00C;
/// 气泡保留时长（秒），到期后移除临时图标
const BALLOON_SECS: u64 = 6;

fn fill_wstr<const N: usize>(dst: &mut [u16; N], s: &str) {
    let mut i = 0;
    for u in s.encode_utf16() {
        if i + 1 >= N {
            break;
        }
        dst[i] = u;
        i += 1;
    }
    dst[i] = 0;
}

/// 启动托盘气泡提示（needTraytip）：临时挂一个带 NIF_INFO 的托盘图标并显示气泡
unsafe fn balloon_show(hwnd: isize, title: &str, text: &str) -> bool {
    use windows_sys::Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_TIP, NIIF_INFO, NIM_ADD, NOTIFYICONDATAW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{LoadIconW, IDI_APPLICATION};

    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd as _;
    nid.uID = BALLOON_UID;
    nid.uFlags = NIF_INFO | NIF_ICON | NIF_TIP;
    nid.hIcon = LoadIconW(std::ptr::null_mut(), IDI_APPLICATION);
    fill_wstr(&mut nid.szTip, "ShowKeyBoard");
    fill_wstr(&mut nid.szInfoTitle, title);
    fill_wstr(&mut nid.szInfo, text);
    nid.dwInfoFlags = NIIF_INFO;
    Shell_NotifyIconW(NIM_ADD, &nid) != 0
}

/// 移除气泡用的临时托盘图标
unsafe fn balloon_remove(hwnd: isize) {
    use windows_sys::Win32::UI::Shell::{
        Shell_NotifyIconW, NIM_DELETE, NOTIFYICONDATAW,
    };
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd as _;
    nid.uID = BALLOON_UID;
    Shell_NotifyIconW(NIM_DELETE, &nid);
}

pub fn start(shared: Arc<Shared>, owner: isize) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("tray".into())
        .spawn(move || unsafe {
            let lang_en = shared.cfg.read().unwrap().default_lang == "en-US";
            let (t_set, t_stat, t_show, t_reload, t_exit, tip) = if lang_en {
                (
                    "Setting", "Statistics", "Show/Hide Keys", "Reload Config", "Exit",
                    format!("ShowKeyBoard v{}", crate::stats::env_full_version()),
                )
            } else {
                (
                    "参数设置", "数据统计", "按键显示", "重载配置", "退出程序",
                    format!("ShowKeyBoard v{}", crate::stats::env_full_version()),
                )
            };

            let menu = Menu::new();
            let mi_settings = MenuItem::with_id(ID_SETTINGS, t_set, true, None);
            let mi_stat = MenuItem::with_id(ID_STAT, t_stat, true, None);
            let need_show = shared.cfg.read().unwrap().need_show_key;
            let mi_show = CheckMenuItem::with_id(ID_SHOW, t_show, true, need_show, None);
            let mi_reload = MenuItem::with_id(ID_RELOAD, t_reload, true, None);
            let sep1 = PredefinedMenuItem::separator();
            let sep2 = PredefinedMenuItem::separator();
            let mi_exit = MenuItem::with_id(ID_EXIT, t_exit, true, None);
            let _ = menu.append(&mi_settings);
            let _ = menu.append(&mi_stat);
            let _ = menu.append(&mi_show);
            let _ = menu.append(&mi_reload);
            let _ = menu.append(&sep1);
            let _ = menu.append(&sep2);
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
                crate::winutil::ico_to_rgba(&bytes, 32)
                    .and_then(|(rgba, w, h)| tray_icon::Icon::from_rgba(rgba, w, h).ok())
            };

            let mut builder = TrayIconBuilder::new().with_tooltip(tip).with_menu(Box::new(menu));
            if let Some(ic) = icon {
                builder = builder.with_icon(ic);
            }
            // 注意：不 mem::forget——退出循环后 drop 会自动移除托盘图标，
            // 否则退出后会残留一个"幽灵"图标（变量存活到循环结束）
            #[allow(unused_variables)]
            let tray: TrayIcon = match builder.build() {
                Ok(t) => t,
                Err(_) => return,
            };

            // 消息泵 + 菜单事件
            let mut msg: windows_sys::Win32::UI::WindowsAndMessaging::MSG = std::mem::zeroed();
            let mut last_gen = shared.show_key_generation.load(Ordering::Relaxed);

            // needTraytip：启动时托盘气泡提示
            let mut balloon_at: Option<std::time::Instant> = None;
            if shared.cfg.read().unwrap().need_traytip {
                let (ver, hotkey) = {
                    let c = shared.cfg.read().unwrap();
                    (crate::stats::env_full_version(), c.hotkey4_show.clone())
                };
                let text = if hotkey.is_empty() {
                    format!("ShowKeyBoard v{} 已启动", ver)
                } else {
                    format!("ShowKeyBoard v{} 已启动，热键 {} 显示/隐藏", ver, hotkey)
                };
                if balloon_show(owner, "ShowKeyBoard", &text) {
                    balloon_at = Some(std::time::Instant::now());
                }
            }

            loop {
                // 气泡到期后移除临时托盘图标
                if let Some(t) = balloon_at {
                    if t.elapsed().as_secs() >= BALLOON_SECS {
                        balloon_remove(owner);
                        balloon_at = None;
                    }
                }
                // hotkey4Show 从钩子线程切换 needShowKey 时，同步勾选态
                let gen = shared.show_key_generation.load(Ordering::Relaxed);
                if gen != last_gen {
                    last_gen = gen;
                    let checked = shared.cfg.read().unwrap().need_show_key;
                    mi_show.set_checked(checked);
                }
                while windows_sys::Win32::UI::WindowsAndMessaging::PeekMessageW(
                    &mut msg, std::ptr::null_mut(), 0, 0,
                    windows_sys::Win32::UI::WindowsAndMessaging::PM_REMOVE,
                ) != 0
                {
                    windows_sys::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                    windows_sys::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
                }
                while let Ok(ev) = MenuEvent::receiver().try_recv() {
                    match ev.id().as_ref() {
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
                            let _ = new_state;
                        }
                        ID_RELOAD => {
                            shared.ws_reload.store(true, Ordering::Relaxed);
                        }
                        ID_EXIT => {
                            shared.exit.store(true, Ordering::Relaxed);
                            let _ = slint::quit_event_loop();
                        }
                        _ => {}
                    }
                }
                if shared.exit.load(Ordering::Relaxed) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        })
        .expect("spawn tray thread")
}


