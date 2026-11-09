//! macOS 输入捕获：rdev（CGEventTap）。
//! - 需要"系统设置 → 隐私与安全性 → 辅助功能"里授权本程序
//! - 全局键鼠事件经 rdev 回调注册为 VK，再共用 Windows 侧的命名/过滤/显示逻辑
//!
//! 与 Windows 的低级钩子一样，回调只做最轻量的状态更新与入队，
//! 字符串处理、配置读取、界面显示与统计上报都在消费线程完成。
#![allow(dead_code)]
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, RecvTimeoutError, Sender};

use rdev::{Button, EventType, Key};

use crate::stats::Shared;
use super::{add_modifier, in_skip, is_ctrl_vk, vk_name};

/// 连续按键（含自动重复）计数，抬起时复位；与控制键分开统计
static REPEAT_RECORD: AtomicI32 = AtomicI32::new(0);
static CTRL_REPEAT_RECORD: AtomicI32 = AtomicI32::new(0);

const EVENT_QUEUE_SIZE: usize = 2048;
static EVENT_TX: OnceLock<Sender<Ev>> = OnceLock::new();

enum Ev {
    Key { vk: u32, extended: bool, down: bool, first: bool },
    Button { name: &'static str, down: bool },
    Wheel { dx: i64, dy: i64 },
}

/// rdev Key -> (VK, extended)。extended 仅用于区分主键盘 Enter 与小键盘 Enter
fn key_to_vk(key: Key) -> Option<(u32, bool)> {
    let pair = match key {
        Key::Alt => (0xA4, false),
        Key::AltGr => (0xA5, false),
        Key::Backspace => (0x08, false),
        Key::CapsLock => (0x14, false),
        Key::ControlLeft => (0xA2, false),
        Key::ControlRight => (0xA3, false),
        Key::Delete => (0x2E, false),
        Key::DownArrow => (0x28, false),
        Key::End => (0x23, false),
        Key::Escape => (0x1B, false),
        Key::F1 => (0x70, false),
        Key::F2 => (0x71, false),
        Key::F3 => (0x72, false),
        Key::F4 => (0x73, false),
        Key::F5 => (0x74, false),
        Key::F6 => (0x75, false),
        Key::F7 => (0x76, false),
        Key::F8 => (0x77, false),
        Key::F9 => (0x78, false),
        Key::F10 => (0x79, false),
        Key::F11 => (0x7A, false),
        Key::F12 => (0x7B, false),
        Key::Home => (0x24, false),
        Key::LeftArrow => (0x25, false),
        Key::MetaLeft => (0x5B, false),
        Key::MetaRight => (0x5C, false),
        Key::PageDown => (0x22, false),
        Key::PageUp => (0x21, false),
        Key::Return => (0x0D, false),
        Key::RightArrow => (0x27, false),
        Key::ShiftLeft => (0xA0, false),
        Key::ShiftRight => (0xA1, false),
        Key::Space => (0x20, false),
        Key::Tab => (0x09, false),
        Key::UpArrow => (0x26, false),
        Key::PrintScreen => (0x2C, false),
        Key::ScrollLock => (0x91, false),
        Key::Pause => (0x13, false),
        Key::NumLock => (0x90, false),
        Key::BackQuote => (0xC0, false),
        Key::Num1 => (0x31, false),
        Key::Num2 => (0x32, false),
        Key::Num3 => (0x33, false),
        Key::Num4 => (0x34, false),
        Key::Num5 => (0x35, false),
        Key::Num6 => (0x36, false),
        Key::Num7 => (0x37, false),
        Key::Num8 => (0x38, false),
        Key::Num9 => (0x39, false),
        Key::Num0 => (0x30, false),
        Key::Minus => (0xBD, false),
        Key::Equal => (0xBB, false),
        Key::KeyQ => (0x51, false),
        Key::KeyW => (0x57, false),
        Key::KeyE => (0x45, false),
        Key::KeyR => (0x52, false),
        Key::KeyT => (0x54, false),
        Key::KeyY => (0x59, false),
        Key::KeyU => (0x55, false),
        Key::KeyI => (0x49, false),
        Key::KeyO => (0x4F, false),
        Key::KeyP => (0x50, false),
        Key::LeftBracket => (0xDB, false),
        Key::RightBracket => (0xDD, false),
        Key::KeyA => (0x41, false),
        Key::KeyS => (0x53, false),
        Key::KeyD => (0x44, false),
        Key::KeyF => (0x46, false),
        Key::KeyG => (0x47, false),
        Key::KeyH => (0x48, false),
        Key::KeyJ => (0x4A, false),
        Key::KeyK => (0x4B, false),
        Key::KeyL => (0x4C, false),
        Key::SemiColon => (0xBA, false),
        Key::Quote => (0xDE, false),
        Key::BackSlash => (0xDC, false),
        Key::IntlBackslash => (0xE2, false),
        Key::KeyZ => (0x5A, false),
        Key::KeyX => (0x58, false),
        Key::KeyC => (0x43, false),
        Key::KeyV => (0x56, false),
        Key::KeyB => (0x42, false),
        Key::KeyN => (0x4E, false),
        Key::KeyM => (0x4D, false),
        Key::Comma => (0xBC, false),
        Key::Dot => (0xBE, false),
        Key::Slash => (0xBF, false),
        Key::Insert => (0x2D, false),
        Key::KpReturn => (0x0D, true),
        Key::KpMinus => (0x6D, false),
        Key::KpPlus => (0x6B, false),
        Key::KpMultiply => (0x6A, false),
        Key::KpDivide => (0x6F, false),
        Key::Kp0 => (0x60, false),
        Key::Kp1 => (0x61, false),
        Key::Kp2 => (0x62, false),
        Key::Kp3 => (0x63, false),
        Key::Kp4 => (0x64, false),
        Key::Kp5 => (0x65, false),
        Key::Kp6 => (0x66, false),
        Key::Kp7 => (0x67, false),
        Key::Kp8 => (0x68, false),
        Key::Kp9 => (0x69, false),
        Key::KpDelete => (0x6E, false),
        _ => return None,
    };
    Some(pair)
}

fn button_name(btn: Button) -> Option<&'static str> {
    match btn {
        Button::Left => Some("LButton"),
        Button::Right => Some("RButton"),
        Button::Middle => Some("MButton"),
        Button::Unknown(3) => Some("XButton1"),
        Button::Unknown(4) => Some("XButton2"),
        _ => None,
    }
}

/// rdev 回调：只更新按键状态并投递事件，保持 CGEventTap 快速返回
fn on_event(event: rdev::Event) {
    match event.event_type {
        EventType::KeyPress(key) => key_event(key, true),
        EventType::KeyRelease(key) => key_event(key, false),
        EventType::ButtonPress(btn) => button_event(btn, true),
        EventType::ButtonRelease(btn) => button_event(btn, false),
        EventType::Wheel { delta_x, delta_y } => {
            if let Some(tx) = EVENT_TX.get() {
                let _ = tx.try_send(Ev::Wheel { dx: delta_x, dy: delta_y });
            }
        }
        _ => {}
    }
}

fn key_event(key: Key, down: bool) {
    let Some((vk, extended)) = key_to_vk(key) else { return };
    // 先更新按下状态（add_modifier 依赖），再入队交给消费线程
    let was_down = crate::winutil::key_down(vk as i32);
    crate::winutil::set_key_down(vk, down);
    crate::winutil::lock_key_event(vk as i32, down);
    if let Some(tx) = EVENT_TX.get() {
        let _ = tx.try_send(Ev::Key {
            vk,
            extended,
            down,
            first: down && !was_down,
        });
    }
}

fn button_event(btn: Button, down: bool) {
    let Some(name) = button_name(btn) else { return };
    if let Some(tx) = EVENT_TX.get() {
        let _ = tx.try_send(Ev::Button { name, down });
    }
}

fn reset_key_repeat(vk: u32) {
    let slot = if is_ctrl_vk(vk) { &CTRL_REPEAT_RECORD } else { &REPEAT_RECORD };
    slot.store(0, Ordering::Relaxed);
}

/// 按键按下：重复计数过滤 + 命名 + 统一走 PushTxt（needShowKey/统计/进程过滤都在其中）
fn handle_key(shared: &Arc<Shared>, vk: u32, extended: bool) {
    let is_ctrl = is_ctrl_vk(vk);
    let rep_atomic = if is_ctrl { &CTRL_REPEAT_RECORD } else { &REPEAT_RECORD };
    let name = vk_name(vk, extended);
    let (max, skipped) = {
        let cfg = shared.cfg.read().unwrap();
        let max = if is_ctrl { cfg.max_ctrlpress_count } else { cfg.max_keypress_count };
        (max, in_skip(&cfg.skip_keys(), &name))
    };
    if skipped || rep_atomic.load(Ordering::Relaxed) >= max {
        return;
    }
    rep_atomic.fetch_add(1, Ordering::Relaxed);
    let full_key = format!("{}{}", add_modifier(), name);
    crate::stats::push_txt(shared, &full_key, false);
}

/// 鼠标按键 / 滚轮
fn handle_mouse(shared: &Arc<Shared>, name: &str) {
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

/// 热键字符 -> VK
fn char_to_vk(c: char) -> Option<u32> {
    let u = c.to_ascii_uppercase();
    match u {
        'A'..='Z' | '0'..='9' => Some(u as u32),
        '`' => Some(0xC0),
        '=' => Some(0xBB),
        ',' => Some(0xBC),
        '-' => Some(0xBD),
        '.' => Some(0xBE),
        '/' => Some(0xBF),
        '[' => Some(0xDB),
        '\\' => Some(0xDC),
        ']' => Some(0xDD),
        ';' => Some(0xBA),
        '\'' => Some(0xDE),
        _ => None,
    }
}

/// 检查 hotkey4Show 并切换 needShowKey（空串表示未启用）。
/// macOS 没有 RegisterHotKey，直接在事件流里比对修饰键状态。
fn check_hotkey(shared: &Arc<Shared>, vk: u32) -> bool {
    let hotkey = shared.cfg.read().unwrap().hotkey4_show.clone();
    if hotkey.trim().is_empty() {
        return false;
    }
    let mut need = [false; 4]; // ctrl, alt, shift, win(cmd)
    let mut main = None;
    for c in hotkey.trim().chars() {
        match c {
            '^' => need[0] = true,
            '!' => need[1] = true,
            '+' => need[2] = true,
            '#' => need[3] = true,
            _ => main = Some(c),
        }
    }
    let Some(ch) = main else { return false };
    if char_to_vk(ch) != Some(vk) {
        return false;
    }
    let down = |a: i32, b: i32| crate::winutil::key_down(a) || crate::winutil::key_down(b);
    if need[0] != down(0xA2, 0xA3)
        || need[1] != down(0xA4, 0xA5)
        || need[2] != down(0xA0, 0xA1)
        || need[3] != down(0x5B, 0x5C)
    {
        return false;
    }
    let mut cfg = shared.cfg.write().unwrap();
    cfg.need_show_key = !cfg.need_show_key;
    drop(cfg);
    shared.show_key_generation.fetch_add(1, Ordering::Relaxed);
    true
}

/// 消费线程：按键命名、过滤、显示与统计都在这里完成
fn consume_events(shared: Arc<Shared>, rx: Receiver<Ev>) {
    loop {
        if shared.exit.load(Ordering::Relaxed) {
            break;
        }
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ev::Key { vk, extended, down, first }) => {
                if down {
                    // 命中热键时消费掉该按键（与 Windows RegisterHotKey 行为一致）
                    if first && check_hotkey(&shared, vk) {
                        continue;
                    }
                    handle_key(&shared, vk, extended);
                } else {
                    reset_key_repeat(vk);
                }
            }
            Ok(Ev::Button { name, down }) => {
                if down {
                    handle_mouse(&shared, name);
                } else {
                    REPEAT_RECORD.store(0, Ordering::Relaxed);
                }
            }
            Ok(Ev::Wheel { dx, dy }) => {
                if dy > 0 {
                    handle_mouse(&shared, "WheelUp");
                } else if dy < 0 {
                    handle_mouse(&shared, "WheelDown");
                }
                if dx > 0 {
                    handle_mouse(&shared, "WheelRight");
                } else if dx < 0 {
                    handle_mouse(&shared, "WheelLeft");
                }
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(_) => break,
        }
    }
}

/// 检查"辅助功能"授权（AXIsProcessTrustedWithOptions，不触发系统弹窗）
#[cfg(target_os = "macos")]
fn is_accessibility_trusted() -> bool {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::string::CFString;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> u8;
    }

    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::false_value();
    let dict = CFDictionary::from_CFType_pairs(&[(key, value)]);
    unsafe { AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef()) != 0 }
}

/// 弹框提示授权，返回用户是否选择"去授权"（经 osascript，避免引入 AppKit 绑定依赖）
#[cfg(target_os = "macos")]
fn prompt_accessibility_dialog() -> bool {
    let script = concat!(
        r#"display dialog "#,
        r#""ShowKeyBoard 需要监控键盘鼠标动作。"#,
        r#"\n\n当前尚未获得"辅助功能"授权，"#,
        r#"点击"去授权"打开设置页，添加并勾选本程序后，"#,
        r#"键鼠捕获会自动开始。""#,
        r#" with title "ShowKeyBoard 需要辅助功能权限""#,
        r#" buttons {"稍后", "去授权"} default button "去授权" with icon caution"#
    );
    std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).contains("去授权"))
        .unwrap_or(false)
}

/// 跳转到 系统设置 → 隐私与安全性 → 辅助功能
#[cfg(target_os = "macos")]
fn open_accessibility_settings() {
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}

/// 启动输入捕获线程。rdev（CGEventTap）阻塞在自己的线程里，
/// 返回的线程只等待退出标志，保证主线程 join 不会卡死。
pub fn start(shared: Arc<Shared>) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("input-macos".into())
        .spawn(move || {
            let (tx, rx) = bounded(EVENT_QUEUE_SIZE);
            let _ = EVENT_TX.set(tx);

            let consumer_shared = shared.clone();
            std::thread::Builder::new()
                .name("input-consumer".into())
                .spawn(move || consume_events(consumer_shared, rx))
                .expect("spawn input consumer");

            let tap_shared = shared.clone();
            std::thread::Builder::new()
                .name("input-tap".into())
                .spawn(move || {
                    let mut hinted = false;
                    loop {
                        if is_accessibility_trusted() {
                            match rdev::listen(on_event) {
                                Ok(()) => break,
                                Err(e) => {
                                    eprintln!("input: rdev listen failed: {:?}", e);
                                    std::thread::sleep(Duration::from_secs(5));
                                }
                            }
                        } else {
                            // 未授权：浮窗常驻提示 + 弹框引导，跳转设置页；授权后自动开始捕获
                            if !hinted {
                                hinted = true;
                                let _ = tap_shared.ui_tx.send(format!(
                                    "{}键盘鼠标捕获失败：请在 系统设置 → 隐私与安全性 → 辅助功能 中允许本程序",
                                    crate::net::STATUS_STICKY
                                ));
                            }
                            if prompt_accessibility_dialog() {
                                open_accessibility_settings();
                            }
                            while !is_accessibility_trusted()
                                && !tap_shared.exit.load(Ordering::Relaxed)
                            {
                                std::thread::sleep(Duration::from_millis(500));
                            }
                            if tap_shared.exit.load(Ordering::Relaxed) {
                                break;
                            }
                        }
                    }
                })
                .expect("spawn rdev listen");

            while !shared.exit.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(200));
            }
        })
        .expect("spawn input thread")
}

/// macOS 每次按键都实时读取配置，无需像 Windows 那样缓存钩子参数。
/// 保留该函数以兼容 net.rs 配置热重载时的调用。
pub fn refresh_hook_params() {}
