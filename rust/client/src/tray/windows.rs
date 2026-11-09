//! 托盘图标与菜单（对应 AHK CreateMenu/MenuHandler）
#![allow(dead_code)]
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

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

/// 旧版启动文件夹快捷方式路径（对应 AHK A_Startup "\" APPName ".Lnk"），仅用于兼容识别/清理
fn startup_link() -> Option<std::path::PathBuf> {
    std::env::var_os("APPDATA").map(|p| {
        std::path::PathBuf::from(p)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup")
            .join("ShowKeyBoard.lnk")
    })
}

fn current_exe_path() -> std::path::PathBuf {
    std::env::current_exe().unwrap_or_default()
}

/// 开机启动写入 HKCU\...\Run（相当于 AHK FileCreateShortcut，无需外部进程）
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_NAME: &str = "ShowKeyBoard";

fn to_wide0(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 打开 Run 键：write=true 时允许写入（不存在则创建），否则只读
fn run_key(write: bool) -> Option<windows_sys::Win32::System::Registry::HKEY> {
    use windows_sys::Win32::System::Registry::{
        RegCreateKeyExW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE,
        REG_OPTION_NON_VOLATILE,
    };
    let sub = to_wide0(RUN_KEY);
    let mut key: HKEY = std::ptr::null_mut();
    let rc = unsafe {
        if write {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                sub.as_ptr(),
                0,
                std::ptr::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                std::ptr::null(),
                &mut key,
                std::ptr::null_mut(),
            )
        } else {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                sub.as_ptr(),
                0,
                KEY_QUERY_VALUE,
                &mut key,
            )
        }
    };
    if rc == 0 { Some(key) } else { None }
}

/// 读取 Run 键中的开机启动命令行
fn startup_command() -> Option<String> {
    use windows_sys::Win32::System::Registry::{RegCloseKey, RegQueryValueExW, REG_SZ};
    let key = run_key(false)?;
    let name = to_wide0(RUN_NAME);
    let mut ty: u32 = 0;
    let mut buf = [0u16; 1024];
    let mut len = (buf.len() * 2) as u32;
    let rc = unsafe {
        RegQueryValueExW(
            key,
            name.as_ptr(),
            std::ptr::null(),
            &mut ty,
            buf.as_mut_ptr() as *mut u8,
            &mut len,
        )
    };
    unsafe { RegCloseKey(key) };
    if rc != 0 || ty != REG_SZ {
        return None;
    }
    let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..n]))
}

/// 创建/删除开机启动项（对应 AHK FileCreateShortcut / FileDelete）
fn startup_set(enable: bool) {
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegSetValueExW, REG_SZ,
    };
    // 清理旧版本写入“启动”文件夹的快捷方式，避免出现两份启动项
    if let Some(lnk) = startup_link() {
        let _ = std::fs::remove_file(lnk);
    }
    let Some(key) = run_key(true) else { return };
    let name = to_wide0(RUN_NAME);
    unsafe {
        if enable {
            let cmd = to_wide0(&format!("\"{}\"", current_exe_path().display()));
            RegSetValueExW(
                key,
                name.as_ptr(),
                0,
                REG_SZ,
                cmd.as_ptr() as *const u8,
                (cmd.len() * 2) as u32,
            );
        } else {
            RegDeleteValueW(key, name.as_ptr());
        }
        RegCloseKey(key);
    }
}

/// 开机启动是否已启用：Run 键中记录的命令行包含当前 exe；
/// 兼容识别旧版本写在“启动”文件夹的 ShowKeyBoard.lnk。
fn startup_checked() -> bool {
    let exe = current_exe_path().to_string_lossy().to_lowercase();
    if let Some(cmd) = startup_command() {
        if !exe.is_empty() && cmd.to_lowercase().contains(&exe) {
            return true;
        }
    }
    startup_link().map(|p| p.exists()).unwrap_or(false)
}

pub fn start(shared: Arc<Shared>, owner: isize) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("tray".into())
        .spawn(move || unsafe {
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
                    "Setting", "Statistics", "Show/Hide Key", "Auto Start",
                    "Reset Client", "Full Reset", "Exit",
                )
            } else {
                (
                    "参数设置", "数据统计", "按键显示", "开机启动",
                    "重启程序", "完全重启", "退出程序",
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
            let mut startup_on = startup_checked();
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
                crate::icon::ico_to_rgba(&bytes, 32)
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
                        ID_STARTUP => {
                            // 注册表读写是瞬间操作，直接在托盘线程执行
                            startup_set(!startup_on);
                            startup_on = startup_checked();
                            mi_startup.set_checked(startup_on);
                        }
                        ID_RESTART => {
                            // 重启程序：只重启客户端，后台服务不退出
                            shared.restart_mode.store(1, Ordering::Relaxed);
                            shared.exit.store(true, Ordering::Relaxed);
                            let _ = slint::quit_event_loop();
                        }
                        ID_RESTART_ALL => {
                            // 完全重启：先同步关闭后台服务（/exit 不生效则按 pid 强制结束并等待端口下线），
                            // 再退出客户端；主线程 join 本托盘线程后才拉起新实例，
                            // 新实例检测不到服务，会重新启动后端。
                            shared.restart_mode.store(2, Ordering::Relaxed);
                            crate::net::shutdown_server(&shared);
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
                            let mut cfg = shared.cfg.write().unwrap();
                            cfg.need_show_key = !cfg.need_show_key;
                            let new_state = cfg.need_show_key;
                            drop(cfg);
                            shared.show_key_generation.fetch_add(1, Ordering::Relaxed);
                            let _ = new_state;
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

/// Windows 托盘在独立线程运行自己的消息循环，无需外部轮询。
/// 保留该函数是为了与 macOS 实现（必须在 UI 主线程惰性创建）保持同一接口。
pub fn poll(_shared: &Arc<Shared>) {}

