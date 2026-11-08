//! 手柄输入：Windows 用 XInput / winmm，macOS 用 gilrs（IOKit HID）。
//! 两者都输出 AHK 风格按键名（1Joy1 … 1JoyPOV8），统一走 stats::push_txt。
#![allow(dead_code)]

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;
