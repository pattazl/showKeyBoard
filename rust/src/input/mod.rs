//! 输入捕获（跨平台）：
//! - Windows：WH_KEYBOARD_LL / WH_MOUSE_LL 低级钩子
//! - macOS：CGEventTap（rdev 封装，需在"系统设置 → 隐私与安全性 → 辅助功能"授权）
//!
//! 各平台的采集实现只负责产生按键名（AHK 命名），过滤、显示与统计统一走 stats::push_txt，
//! 因此两边行为保持一致。
#![allow(dead_code)]

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

/// VK -> AHK 按键名（macOS 侧先把 rdev Key 映射为 VK 再共用此函数）
pub(crate) fn vk_name(vk: u32, extended: bool) -> String {
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
pub(crate) fn add_modifier() -> String {
    const ORDER: [(i32, &str); 8] = [
        (0xA4, "<!"), // LAlt
        (0xA2, "<^"), // LControl
        (0xA0, "<+"), // LShift
        (0x5B, "<#"), // LWin / Cmd
        (0xA5, ">!"), // RAlt
        (0xA3, ">^"), // RControl
        (0xA1, ">+"), // RShift
        (0x5C, ">#"), // RWin / RCmd
    ];
    let mut s = String::new();
    for (vk, sym) in ORDER {
        if crate::winutil::key_down(vk) {
            s.push_str(sym);
        }
    }
    s
}

pub(crate) fn in_skip(set: &[String], name: &str) -> bool {
    set.iter().any(|k| k == name)
}

pub(crate) fn is_ctrl_vk(vk: u32) -> bool {
    matches!(vk,
        0x10 | 0x11 | 0x12 | 0x14 |
        0x5B | 0x5C |
        0x90 | 0x91 |
        0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5
    )
}
