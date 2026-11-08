//! 系统相关工具：显示器 / 前台应用 / 锁屏 / 窗口样式 / 字体测量 / 图标。
//! Windows 与 macOS 各自实现，对外接口保持一致。
#![allow(dead_code)]

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;
