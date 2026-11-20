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

// macOS 全局键鼠监听：精简版 rdev（源码内联在本目录，见 rdev.rs 头注释）
#[cfg(target_os = "macos")]
pub(crate) mod rdev;

/// VK -> AHK 按键名（macOS 侧先把 rdev Key 映射为 VK 再共用此函数）
/// 名称与 AHK GetKeyName 保持一致（KeyList.txt / skipRecord / 跨端统计都依赖它）
pub(crate) fn vk_name(vk: u32, extended: bool) -> String {
    let name = match vk {
        0x08 => "Backspace", 0x09 => "Tab", 0x0D => "Enter", 0x10 => "Shift",
        0x11 => "Control", 0x12 => "Alt", 0x13 => "Pause", 0x14 => "CapsLock",
        0x1B => "Escape", 0x20 => "Space", 0x21 => "PgUp", 0x22 => "PgDn",
        0x23 => "End", 0x24 => "Home", 0x25 => "Left", 0x26 => "Up",
        0x27 => "Right", 0x28 => "Down", 0x2C => "PrintScreen", 0x2D => "Insert",
        0x2E => "Delete", 0x2F => "Help", 0x5B => "LWin", 0x5C => "RWin", 0x5D => "AppsKey",
        0x5F => "Sleep",
        0x60 => "Numpad0", 0x61 => "Numpad1", 0x62 => "Numpad2", 0x63 => "Numpad3",
        0x64 => "Numpad4", 0x65 => "Numpad5", 0x66 => "Numpad6", 0x67 => "Numpad7",
        0x68 => "Numpad8", 0x69 => "Numpad9", 0x6A => "NumpadMult", 0x6B => "NumpadAdd",
        0x6D => "NumpadSub", 0x6E => "NumpadDot", 0x6F => "NumpadDiv",
        0x90 => "NumLock", 0x91 => "ScrollLock",
        0xA0 => "LShift", 0xA1 => "RShift", 0xA2 => "LControl", 0xA3 => "RControl",
        0xA4 => "LAlt", 0xA5 => "RAlt",
        0xA6 => "BrowserBack", 0xA7 => "BrowserForward", 0xA8 => "BrowserRefresh",
        0xA9 => "BrowserStop", 0xAA => "BrowserSearch", 0xAB => "BrowserFavorites",
        0xAC => "BrowserHome",
        0xAD => "VolumeMute", 0xAE => "VolumeDown", 0xAF => "VolumeUp",
        0xB0 => "MediaNext", 0xB1 => "MediaPrev", 0xB2 => "MediaStop",
        0xB3 => "MediaPlayPause", 0xB4 => "LaunchMail", 0xB5 => "LaunchMedia",
        0xB6 => "LaunchApp1", 0xB7 => "LaunchApp2",
        // macOS 媒体/设备控制键（NX_SYSDEFINED 专有，Windows 无对应 VK）
        0xE002 => "BrightnessUp", 0xE003 => "BrightnessDown",
        0xE006 => "Power",
        0xE00B => "ContrastUp", 0xE00C => "ContrastDown",
        0xE00D => "LaunchPanel", 0xE00E => "Eject", 0xE00F => "VideoMirror",
        0xE013 => "MediaFastForward", 0xE014 => "MediaRewind",
        0xE015 => "IlluminationUp", 0xE016 => "IlluminationDown",
        0xE017 => "IlluminationToggle",
        // macOS 功能键顶排的 Apple 专有键
        0xE100 => "Dictation", 0xE101 => "Spotlight", 0xE102 => "DoNotDisturb",
        0xE103 => "MissionControl",
        // OEM 键：AHK GetKeyName 返回的是未按 Shift 的基础字符（; 而不是 :）
        0xBA => ";", 0xBB => "=", 0xBC => ",", 0xBD => "-", 0xBE => ".", 0xBF => "/",
        0xC0 => "`", 0xDB => "[", 0xDC => "\\", 0xDD => "]", 0xDE => "'",
        0xE2 => "\\", // VK_OEM_102（国际键盘的额外 \ 键）
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

/// AddModifier：修饰键前缀（顺序与 AHK getKeyInput.ahk 的 ModifierMapping 一致）
pub(crate) fn add_modifier() -> String {
    const ORDER: [(i32, &str); 8] = [
        (0xA2, "<^"), // LControl
        (0xA3, ">^"), // RControl
        (0x5B, "<#"), // LWin / Cmd
        (0x5C, ">#"), // RWin / RCmd
        (0xA4, "<!"), // LAlt
        (0xA5, ">!"), // RAlt
        (0xA0, "<+"), // LShift
        (0xA1, ">+"), // RShift
    ];
    let mut s = String::new();
    for (vk, sym) in ORDER {
        if crate::winutil::key_down(vk) {
            s.push_str(sym);
        }
    }
    s
}

/// AHK GetKeyName 的名称归一化：skipRecord 里用户常写简写（Esc / Del / Ctrl），
/// AHK 内部会统一成规范名再比较，这里做等价处理，避免简写失效。
fn canonical_key(name: &str) -> &str {
    match name {
        "Esc" => "Escape",
        "Ctrl" => "Control",
        "Del" => "Delete",
        "Ins" => "Insert",
        "BS" => "Backspace",
        "Return" => "Enter",
        "Menu" => "AppsKey",
        other => other,
    }
}

pub(crate) fn in_skip(set: &[String], name: &str) -> bool {
    let target = canonical_key(name);
    set.iter().any(|k| canonical_key(k) == target)
}

pub(crate) fn is_ctrl_vk(vk: u32) -> bool {
    matches!(vk,
        0x10 | 0x11 | 0x12 | 0x14 |
        0x5B | 0x5C |
        0x90 | 0x91 |
        0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5
    )
}

/// 媒体键 / 浏览器键 VK 范围。
/// 笔记本 Fn 键常由厂商驱动通过 SendInput 注入，带 LLKHF_INJECTED 标志，
/// 需要单独放行，否则会被“忽略注入事件”的过滤规则丢弃。
pub(crate) fn is_media_vk(vk: u32) -> bool {
    (0xA6..=0xB7).contains(&vk)
}
