//! 托盘菜单：Windows 用 Win32 消息循环，macOS 用 tray-icon（NSStatusItem）。
//!
//! Windows 在 main 中起独立线程并自建消息泵；macOS 的 NSStatusItem 必须在主线程
//! （Slint 事件循环已启动后）创建，因此由 UI 定时器调用 `poll()` 惰性创建并处理菜单事件。
#![allow(dead_code)]

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;
