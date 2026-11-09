//! 输入捕获：合并 hook/getKeyInput.ahk 到主程序。
//! - WH_KEYBOARD_LL 低级键盘钩子（不再使用 WM_COPYDATA 队列，直接进程内通信，效率更高）
//! - WH_MOUSE_LL 低级鼠标钩子（左右中键 / X1 X2 / 滚轮）
//! - hotkey4Show 注册热键切换显示
//!
//! 钩子回调只做最轻量的工作（原子状态 + 入管道），不做字符串/配置/UI 处理，
//! 由独立的消费线程完成显示与统计上报，避免拖慢系统输入（低级钩子超时会被系统摘除）。
#![allow(dead_code)]
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WAIT_TIMEOUT, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, VkKeyScanW, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, MsgWaitForMultipleObjectsEx, PeekMessageW, PM_REMOVE,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT,
    MSG, MWMO_INPUTAVAILABLE, QS_ALLINPUT, WM_QUIT,
};

use crate::stats::Shared;
use super::{add_modifier, in_skip, is_ctrl_vk, vk_name};

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
    max_keypress: std::sync::atomic::AtomicI32,
    max_ctrlpress: std::sync::atomic::AtomicI32,
    repeat_record: std::sync::atomic::AtomicI32,
    ctrl_repeat_record: std::sync::atomic::AtomicI32,
    h_kb: std::sync::atomic::AtomicIsize,
    h_mouse: std::sync::atomic::AtomicIsize,
    hotkey_dirty: std::sync::atomic::AtomicBool,
}

static CTX: std::sync::OnceLock<HookCtx> = std::sync::OnceLock::new();

/// 钩子 -> 消费线程的事件（只含基础字段，回调中不做任何字符串/配置处理）
enum HookEvent {
    Key { vk: u32, extended: bool, injected: bool, down: bool },
    Mouse { msg: usize, data: u16, injected: bool },
}

/// 钩子事件管道（有界，满了直接丢弃，绝不在钩子回调里阻塞）
const EVENT_QUEUE_SIZE: usize = 2048;
static EVENT_TX: std::sync::OnceLock<Sender<HookEvent>> = std::sync::OnceLock::new();

unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        let info = unsafe { &*(l_param as *const KBDLLHOOKSTRUCT) };
        let msg = w_param as usize;
        let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        // 切换键（CapsLock/NumLock/ScrollLock）状态跟踪：供 ctrlState 持续显示。
        // 放在 injected 过滤之前，软件方式切换的状态也能同步（仅原子操作）。
        crate::winutil::lock_key_event(info.vkCode as i32, is_down);
        // 按下/抬起都交给消费线程：计数与复位必须保持同一顺序，钩子回调立即返回
        if let Some(tx) = EVENT_TX.get() {
            let _ = tx.try_send(HookEvent::Key {
                vk: info.vkCode,
                extended: (info.flags & 1) != 0,
                injected: (info.flags & LLKHF_INJECTED) != 0,
                down: is_down,
            });
        }
    }
    unsafe { CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param) }
}

unsafe extern "system" fn mouse_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        let info = unsafe { &*(l_param as *const MSLLHOOKSTRUCT) };
        let msg = w_param as usize;
        // 鼠标钩子只收到按键/滚轮消息；按下与抬起都交给消费线程处理（复位计数）
        if let Some(tx) = EVENT_TX.get() {
            let _ = tx.try_send(HookEvent::Mouse {
                msg,
                data: (info.mouseData >> 16) as u16,
                injected: (info.flags & LLMHF_INJECTED) != 0,
            });
        }
    }
    unsafe { CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param) }
}

/// 消费线程：按键命名、参数过滤、界面显示与统计上报都在这里完成
fn consume_events(shared: Arc<Shared>, rx: Receiver<HookEvent>) {
    loop {
        if shared.exit.load(Ordering::Relaxed) {
            break;
        }
        let ev = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(ev) => ev,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
            Err(_) => break,
        };
        match ev {
            HookEvent::Key { vk, extended, injected, down } => {
                if down {
                    handle_key(&shared, vk, extended, injected);
                } else {
                    reset_key_repeat(vk);
                }
            }
            HookEvent::Mouse { msg, data, injected } => {
                if matches!(msg, WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP) {
                    if let Some(ctx) = CTX.get() {
                        ctx.repeat_record.store(0, Ordering::Relaxed);
                    }
                } else {
                    handle_mouse(&shared, msg, data, injected);
                }
            }
        }
    }
}

/// 按键抬起：复位该类别（普通键/控制键）的重复计数
fn reset_key_repeat(vk: u32) {
    if let Some(ctx) = CTX.get() {
        if is_ctrl_vk(vk) {
            ctx.ctrl_repeat_record.store(0, Ordering::Relaxed);
        } else {
            ctx.repeat_record.store(0, Ordering::Relaxed);
        }
    }
}

fn handle_key(shared: &Arc<Shared>, vk: u32, extended: bool, injected: bool) {
    if injected {
        return;
    }
    let ctx = CTX.get().unwrap();
    let is_ctrl = is_ctrl_vk(vk);
    let (max, rep_atomic) = if is_ctrl {
        (ctx.max_ctrlpress.load(Ordering::Relaxed), &ctx.ctrl_repeat_record)
    } else {
        (ctx.max_keypress.load(Ordering::Relaxed), &ctx.repeat_record)
    };
    let rep = rep_atomic.load(Ordering::Relaxed);
    if rep >= max {
        return;
    }
    let name = vk_name(vk, extended);
    let skipped = {
        let cfg = shared.cfg.read().unwrap();
        in_skip(&cfg.skip_keys(), &name)
    };
    if skipped {
        return;
    }
    rep_atomic.store(rep + 1, Ordering::Relaxed);
    let full_key = format!("{}{}", add_modifier(), name);
    // 统一走 PushTxt：needShowKey/monitorProc/activeWindowProc/hideInWinPwd
    // 过滤与记录统计都在其中（原来直接发 UI 通道导致这些参数全部失效）
    crate::stats::push_txt(shared, &full_key, false);
}

fn handle_mouse(shared: &Arc<Shared>, msg: usize, data: u16, injected: bool) {
    if injected {
        return;
    }
    let name = match msg {
        WM_LBUTTONDOWN => "LButton",
        WM_RBUTTONDOWN => "RButton",
        WM_MBUTTONDOWN => "MButton",
        WM_XBUTTONDOWN => {
            if data == 0x0002 {
                "XButton2"
            } else {
                "XButton1"
            }
        }
        WM_MOUSEWHEEL => {
            if (data as i16) > 0 {
                "WheelUp"
            } else {
                "WheelDown"
            }
        }
        WM_MOUSEHWHEEL => {
            if (data as i16) > 0 {
                "WheelRight"
            } else {
                "WheelLeft"
            }
        }
        _ => return,
    };
    let (skipped, show_mouse) = {
        let cfg = shared.cfg.read().unwrap();
        (in_skip(&cfg.skip_keys(), name), cfg.show_mouse_event != 0)
    };
    if skipped || !show_mouse {
        return;
    }
    let full_key = format!("{}{}", add_modifier(), name);
    crate::stats::push_txt(shared, &full_key, true);
}

/// 解析 hotkey4Show（如 ^!+s）并注册；空串则注销已有热键
fn register_show_hotkey(hotkey: &str) -> bool {
    // 先注销旧热键（重载配置时热键可能变化）
    unsafe { UnregisterHotKey(std::ptr::null_mut(), HOTKEY_ID); }
    if hotkey.trim().is_empty() {
        return false;
    }
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
pub fn start(shared: Arc<Shared>) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("input-hooks".into())
        .spawn(move || unsafe {
            let cfg = shared.cfg.read().unwrap();
            let max_kp = cfg.max_keypress_count;
            let max_cp = cfg.max_ctrlpress_count;
            drop(cfg);
            // 注意：切换键（CapsLock/NumLock/ScrollLock）的开启状态由钩子事件翻转，
            // 并由 UI 线程每秒从系统同步一次（UI 线程有活跃的消息循环，读到的 toggle 位才最新）。
            let consumer_shared = shared.clone();
            let ctx = HookCtx {
                shared: Box::leak(Box::new(shared)),
                max_keypress: std::sync::atomic::AtomicI32::new(max_kp),
                max_ctrlpress: std::sync::atomic::AtomicI32::new(max_cp),
                repeat_record: std::sync::atomic::AtomicI32::new(0),
                ctrl_repeat_record: std::sync::atomic::AtomicI32::new(0),
                h_kb: std::sync::atomic::AtomicIsize::new(0),
                h_mouse: std::sync::atomic::AtomicIsize::new(0),
                hotkey_dirty: std::sync::atomic::AtomicBool::new(false),
            };
            let _ = CTX.set(ctx);

            // 事件管道 + 消费线程：钩子回调只入队，显示/统计/上报在消费线程完成
            let (tx, rx) = bounded(EVENT_QUEUE_SIZE);
            let _ = EVENT_TX.set(tx);
            std::thread::Builder::new()
                .name("input-consumer".into())
                .spawn(move || consume_events(consumer_shared, rx))
                .expect("spawn input consumer");

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
                let ctx = CTX.get().unwrap();
                if ctx.shared.exit.load(Ordering::Relaxed) {
                    break;
                }
                // 配置热重载后在本线程重新注册热键（RegisterHotKey 与消息循环须同线程）
                if ctx.hotkey_dirty.swap(false, Ordering::Relaxed) {
                    let hotkey = ctx.shared.cfg.read().unwrap().hotkey4_show.clone();
                    register_show_hotkey(&hotkey);
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

/// 钩子参数热更新（maxKeypressCount）+ hotkey4Show 重注册标记
pub fn refresh_hook_params() {
    if let Some(ctx) = CTX.get() {
        let cfg = ctx.shared.cfg.read().unwrap();
        ctx.max_keypress.store(cfg.max_keypress_count, Ordering::Relaxed);
        ctx.max_ctrlpress.store(cfg.max_ctrlpress_count, Ordering::Relaxed);
        drop(cfg);
        ctx.hotkey_dirty.store(true, Ordering::Relaxed);
    }
}
