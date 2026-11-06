//! 输入捕获：合并 hook/getKeyInput.ahk 到主程序。
//! - WH_KEYBOARD_LL 低级键盘钩子（不再使用 WM_COPYDATA 队列，直接进程内通信，效率更高）
//! - WH_MOUSE_LL 低级鼠标钩子（左右中键 / X1 X2 / 滚轮）
//! - hotkey4Show 注册热键切换显示
#![allow(dead_code)]
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crossbeam_channel::Sender;
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WAIT_TIMEOUT, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, VkKeyScanW, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, MsgWaitForMultipleObjectsEx, PeekMessageW, PM_REMOVE,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT,
    MSG, MWMO_INPUTAVAILABLE, QS_ALLINPUT, WM_QUIT,
};

use crate::stats::Shared;

const WH_KEYBOARD_LL: i32 = 13;
const WH_MOUSE_LL: i32 = 14;
const WM_KEYDOWN: usize = 0x0100;
const WM_KEYUP: usize = 0x0101;
const WM_SYSKEYDOWN: usize = 0x0104;
const WM_SYSKEYUP: usize = 0x0105;
const WM_LBUTTONDOWN: usize = 0x0201;
const WM_LBUTTONUP: usize = 0x0202;
const WM_RBUTTONDOWN: usize = 0x0204;
const WM_RBUTTONUP: usize = 0x0205;
const WM_MBUTTONDOWN: usize = 0x0207;
const WM_MBUTTONUP: usize = 0x0208;
const WM_MOUSEWHEEL: usize = 0x020A;
const WM_XBUTTONDOWN: usize = 0x020B;
const WM_XBUTTONUP: usize = 0x020C;
const WM_MOUSEHWHEEL: usize = 0x020E;
const LLKHF_INJECTED: u32 = 0x10;
const LLMHF_INJECTED: u32 = 0x1;
const WM_HOTKEY: usize = 0x0312;
const HOTKEY_ID: i32 = 0xB00B;

/// 钩子上下文（回调只能拿到全局，因此使用静态）
struct HookCtx {
    shared: &'static Arc<Shared>,
    tx: Sender<String>,
    max_keypress: std::sync::atomic::AtomicI32,
    repeat_record: std::sync::atomic::AtomicI32,
    h_kb: std::sync::atomic::AtomicIsize,
    h_mouse: std::sync::atomic::AtomicIsize,
}

static CTX: std::sync::OnceLock<HookCtx> = std::sync::OnceLock::new();

/// VK -> AHK 按键名
pub fn vk_name(vk: u32, extended: bool) -> String {
    let name = match vk {
        0x08 => "Backspace", 0x09 => "Tab", 0x0D => "Enter", 0x10 => "Shift",
        0x11 => "Control", 0x12 => "Alt", 0x13 => "Pause", 0x14 => "CapsLock",
        0x1B => "Esc", 0x20 => "Space", 0x21 => "PgUp", 0x22 => "PgDn",
        0x23 => "End", 0x24 => "Home", 0x25 => "Left", 0x26 => "Up",
        0x27 => "Right", 0x28 => "Down", 0x2C => "PrintScreen", 0x2D => "Insert",
        0x2E => "Delete", 0x5B => "LWin", 0x5C => "RWin",
        0x60 => "Numpad0", 0x61 => "Numpad1", 0x62 => "Numpad2", 0x63 => "Numpad3",
        0x64 => "Numpad4", 0x65 => "Numpad5", 0x66 => "Numpad6", 0x67 => "Numpad7",
        0x68 => "Numpad8", 0x69 => "Numpad9", 0x6A => "NumpadMult", 0x6B => "NumpadAdd",
        0x6D => "NumpadSub", 0x6E => "NumpadDot", 0x6F => "NumpadDiv",
        0x90 => "NumLock", 0x91 => "ScrollLock",
        0xA0 => "LShift", 0xA1 => "RShift", 0xA2 => "LControl", 0xA3 => "RControl",
        0xA4 => "LAlt", 0xA5 => "RAlt",
        0xBA => ":", 0xBB => "=", 0xBC => ",", 0xBD => "-", 0xBE => ".", 0xBF => "/",
        0xC0 => "`", 0xDB => "[", 0xDC => "\\", 0xDD => "]", 0xDE => "'",
        v if (0x30..=0x39).contains(&v) => {
            return char::from_u32(v).unwrap().to_string();
        }
        v if (0x41..=0x5A).contains(&v) => {
            return char::from_u32(v).unwrap().to_string();
        }
        v if (0x70..=0x87).contains(&v) => {
            return format!("F{}", v - 0x70 + 1);
        }
        _ => return format!("vk{:X}", vk),
    };
    // 修饰键与 Enter 的左右/扩展区分（与 getKeyInput 的 sc 处理等价）
    if extended {
        match vk {
            0x0D => return "NumpadEnter".into(),
            0x10 => return "RShift".into(),
            0x11 => return "RControl".into(),
            0x12 => return "RAlt".into(),
            _ => {}
        }
    } else {
        match vk {
            0x10 => return "LShift".into(),
            0x11 => return "LControl".into(),
            0x12 => return "LAlt".into(),
            _ => {}
        }
    }
    name.to_string()
}

/// AddModifier：修饰键前缀（AHK 顺序：按名称排序）
fn add_modifier() -> String {
    const ORDER: [(i32, &str); 8] = [
        (0xA4, "<!"), // LAlt
        (0xA2, "<^"), // LControl
        (0xA0, "<+"), // LShift
        (0x5B, "<#"), // LWin
        (0xA5, ">!"), // RAlt
        (0xA3, ">^"), // RControl
        (0xA1, ">+"), // RShift
        (0x5C, ">#"), // RWin
    ];
    let mut s = String::new();
    for (vk, sym) in ORDER {
        if crate::winutil::key_down(vk) {
            s.push_str(sym);
        }
    }
    s
}

fn in_skip(set: &[String], name: &str) -> bool {
    set.iter().any(|k| k == name)
}

unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let ctx = CTX.get().unwrap();
    if n_code >= 0 {
        let info = unsafe { &*(l_param as *const KBDLLHOOKSTRUCT) };
        let msg = w_param as usize;
        let injected = (info.flags & LLKHF_INJECTED) != 0;
        if msg == WM_KEYUP || msg == WM_SYSKEYUP {
            ctx.repeat_record.store(0, Ordering::Relaxed);
        }
        let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        if is_down && !injected {
            let max = ctx.max_keypress.load(Ordering::Relaxed);
            let rep = ctx.repeat_record.load(Ordering::Relaxed);
            if rep < max {
                let name = vk_name(info.vkCode, (info.flags & 1) != 0);
                let cfg = ctx.shared.cfg.read().unwrap();
                let skipped = in_skip(&cfg.skip_keys(), &name);
                drop(cfg);
                if !skipped {
                    ctx.repeat_record.store(rep + 1, Ordering::Relaxed);
                    let full_key = format!("{}{}", add_modifier(), name);
                    let _ = ctx.tx.try_send(full_key); // 不阻塞钩子
                }
            }
        }
    }
    unsafe { CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param) }
}

unsafe extern "system" fn mouse_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let ctx = CTX.get().unwrap();
    if n_code >= 0 {
        let info = unsafe { &*(l_param as *const MSLLHOOKSTRUCT) };
        let injected = (info.flags & LLMHF_INJECTED) != 0;
        let msg = w_param as usize;
        let name: Option<String> = match msg {
            WM_LBUTTONDOWN => Some("LButton".into()),
            WM_RBUTTONDOWN => Some("RButton".into()),
            WM_MBUTTONDOWN => Some("MButton".into()),
            WM_XBUTTONDOWN => {
                let hi = (info.mouseData >> 16) as u16;
                Some(if hi == 0x0002 { "XButton2".into() } else { "XButton1".into() })
            }
            WM_MOUSEWHEEL => {
                let delta = (info.mouseData >> 16) as i16;
                Some(if delta > 0 { "WheelUp".into() } else { "WheelDown".into() })
            }
            WM_MOUSEHWHEEL => {
                let delta = (info.mouseData >> 16) as i16;
                Some(if delta > 0 { "WheelRight".into() } else { "WheelLeft".into() })
            }
            _ => None,
        };
        // 抬起事件用于复位重复计数
        if matches!(msg, WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP) {
            ctx.repeat_record.store(0, Ordering::Relaxed);
        }
        if let Some(name) = name {
            if !injected {
                let cfg = ctx.shared.cfg.read().unwrap();
                let skipped = in_skip(&cfg.skip_keys(), &name);
                let show_mouse = cfg.show_mouse_event != 0;
                drop(cfg);
                if !skipped && show_mouse {
                    let full_key = format!("{}{}", add_modifier(), name);
                    let _ = ctx.tx.try_send(full_key);
                }
            }
        }
    }
    unsafe { CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param) }
}

/// 解析 hotkey4Show（如 ^!+s）并注册
fn register_show_hotkey(hotkey: &str) -> bool {
    let mut mods = 0u32;
    let mut rest = hotkey.chars();
    loop {
        match rest.next() {
            Some('^') => mods |= MOD_CONTROL,
            Some('!') => mods |= MOD_ALT,
            Some('+') => mods |= MOD_SHIFT,
            Some('#') => mods |= MOD_WIN,
            Some(c) => {
                let mut buf = [0u16; 2];
                c.encode_utf16(&mut buf);
                let vk = unsafe { VkKeyScanW(buf[0]) } as u16;
                let vk = if vk == 0xFFFF {
                    (c.to_ascii_uppercase() as u32) as i32
                } else {
                    (vk & 0xFF) as i32
                };
                unsafe {
                    return RegisterHotKey(std::ptr::null_mut(), HOTKEY_ID, mods, vk as u32) != 0;
                }
            }
            None => return false,
        }
    }
}

/// 启动钩子线程（自带消息循环）
pub fn start(shared: Arc<Shared>, tx: Sender<String>) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("input-hooks".into())
        .spawn(move || unsafe {
            let max = shared.cfg.read().unwrap().max_keypress_count;
            let ctx = HookCtx {
                shared: Box::leak(Box::new(shared)),
                tx,
                max_keypress: std::sync::atomic::AtomicI32::new(max),
                repeat_record: std::sync::atomic::AtomicI32::new(0),
                h_kb: std::sync::atomic::AtomicIsize::new(0),
                h_mouse: std::sync::atomic::AtomicIsize::new(0),
            };
            let _ = CTX.set(ctx);

            let h_kb = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), std::ptr::null_mut(), 0);
            let h_mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), std::ptr::null_mut(), 0);
            CTX.get().unwrap().h_kb.store(h_kb as isize, Ordering::Relaxed);
            CTX.get().unwrap().h_mouse.store(h_mouse as isize, Ordering::Relaxed);

            // hotkey4Show 热键（失败不致命）
            let hotkey = CTX.get().unwrap().shared.cfg.read().unwrap().hotkey4_show.clone();
            let _ = register_show_hotkey(&hotkey);

            // 消息循环（钩子与热键依赖）。
            // 注意：不能用 GetMessageW 阻塞——空闲时它可能长时间不返回，导致
            // 退出标志检查不到、主线程 join 卡死、进程无法退出（托盘退出无效的根因）。
            let mut msg: MSG = std::mem::zeroed();
            loop {
                if CTX.get().unwrap().shared.exit.load(Ordering::Relaxed) {
                    break;
                }
                // 最多等 100ms，有消息立即返回（低级钩子回调依赖本线程取消息）
                let wait = MsgWaitForMultipleObjectsEx(
                    0,
                    std::ptr::null(),
                    100,
                    QS_ALLINPUT,
                    MWMO_INPUTAVAILABLE,
                );
                if wait == WAIT_TIMEOUT {
                    continue;
                }
                while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    if msg.message == WM_QUIT as u32 {
                        break;
                    }
                    if msg.message == WM_HOTKEY as u32 && msg.wParam == HOTKEY_ID as usize {
                        // 切换 needShowKey
                        let ctx = CTX.get().unwrap();
                        let mut cfg = ctx.shared.cfg.write().unwrap();
                        cfg.need_show_key = !cfg.need_show_key;
                        drop(cfg);
                        ctx.shared.show_key_generation.fetch_add(1, Ordering::Relaxed);
                    }
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }

            if h_kb != std::ptr::null_mut() {
                UnhookWindowsHookEx(h_kb);
            }
            if h_mouse != std::ptr::null_mut() {
                UnhookWindowsHookEx(h_mouse);
            }
        })
        .expect("spawn input thread")
}

/// 钩子参数热更新（maxKeypressCount）
pub fn refresh_hook_params() {
    if let Some(ctx) = CTX.get() {
        let max = ctx.shared.cfg.read().unwrap().max_keypress_count;
        ctx.max_keypress.store(max, Ordering::Relaxed);
    }
}
