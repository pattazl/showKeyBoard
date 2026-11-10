//! 精简版 rdev（macOS CGEventTap 全局键鼠监听）。
//!
//! 来源：rdev 0.5.3（MIT License, github.com/Narsil/rdev），按下述方式并入本项目：
//! - 仅保留 macOS `listen` 所需部分（原 crate 的 simulate/grab/display/keyboard
//!   等未用模块全部去掉，避免多余的 cocoa/其它依赖面）
//! - 补丁：`convert()` 不再做 TIS 输入源字符转换（新版 macOS 对 HIToolbox API
//!   加了主队列断言，在 CGEventTap 回调线程调用会 dispatch_assert_queue_fail
//!   直接 SIGTRAP 崩溃；本项目不使用 `Event::name`）
//! - 兼容 core-graphics 0.24（cg 0.24 仍是 `EventField` 常量，直接可用）
//!
//! Windows 侧不使用本模块（低级钩子在 `input/windows.rs`）。
#![allow(dead_code)]

mod keycodes;
mod macos;
mod types;

pub use macos::last_scroll_kind;
pub use macos::listen;
pub use types::{Button, Event, EventType, Key};
