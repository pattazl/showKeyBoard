//! rdev 共享类型（提取自 rdev 0.5.3 的 rdev.rs，仅保留本用到的事件/错误类型，
//! 去掉 serde 序列化与 simulate/grab 相关内容）。

use std::time::SystemTime;
use std::{fmt, fmt::Display};

/// Errors that occur when trying to capture OS events.
/// Be careful on Mac, not setting accessibility does not cause an error
/// it justs ignores events.
#[derive(Debug)]
#[non_exhaustive]
pub enum ListenError {
    /// MacOS
    EventTapError,
    /// MacOS
    LoopSourceError,
    /// Linux
    MissingDisplayError,
    /// Linux
    KeyboardError,
    /// Linux
    RecordContextEnablingError,
    /// Linux
    RecordContextError,
    /// Linux
    XRecordExtensionError,
    /// Windows
    KeyHookError(u32),
    /// Windows
    MouseHookError(u32),
}

/// Key names based on physical location on the device
/// Merge Option(MacOS) and Alt(Windows, Linux) into Alt
/// Merge Windows (Windows), Meta(Linux), Command(MacOS) into Meta
/// Characters based on Qwerty layout, don't use this for characters as it WILL
/// depend on the layout. Use Event.name instead. Key modifiers gives those keys
/// a different value too.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    /// Alt key on Linux and Windows (option key on macOS)
    Alt,
    AltGr,
    Backspace,
    CapsLock,
    ControlLeft,
    ControlRight,
    Delete,
    DownArrow,
    End,
    Escape,
    F1,
    F10,
    F11,
    F12,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    Home,
    LeftArrow,
    /// also known as "windows", "super", and "command"
    MetaLeft,
    /// also known as "windows", "super", and "command"
    MetaRight,
    PageDown,
    PageUp,
    Return,
    RightArrow,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    UpArrow,
    PrintScreen,
    ScrollLock,
    Pause,
    NumLock,
    BackQuote,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Minus,
    Equal,
    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    LeftBracket,
    RightBracket,
    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    SemiColon,
    Quote,
    BackSlash,
    IntlBackslash,
    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    Comma,
    Dot,
    Slash,
    Insert,
    KpReturn,
    KpMinus,
    KpPlus,
    KpMultiply,
    KpDivide,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDelete,
    Function,
    Unknown(u32),
}

/// Standard mouse buttons
/// Some mice have more than 3 buttons. These are not defined, and different
/// OSs will give different `Button::Unknown` values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Button {
    Left,
    Right,
    Middle,
    Unknown(u8),
}

/// In order to manage different OSs, the current EventType choices are a mix and
/// match to account for all possible events.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EventType {
    /// The keys correspond to a standard qwerty layout, they don't correspond
    /// To the actual letter a user would use, that requires some layout logic to be added.
    KeyPress(Key),
    KeyRelease(Key),
    /// Mouse Button
    ButtonPress(Button),
    ButtonRelease(Button),
    /// Values in pixels. `EventType::MouseMove{x: 0, y: 0}` corresponds to the
    /// top left corner, with x increasing downward and y increasing rightward
    MouseMove {
        x: f64,
        y: f64,
    },
    /// `delta_y` represents vertical scroll and `delta_x` represents horizontal scroll.
    /// Positive values correspond to scrolling up or right and negative values
    /// correspond to scrolling down or left
    Wheel {
        delta_x: i64,
        delta_y: i64,
    },
}

/// When events arrive from the OS they get some additional information added from
/// EventType, which is the time when this event was received, and the name Option
/// which contains what characters should be emmitted from that event. This relies
/// on the OS layout and keyboard state machinery.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub time: SystemTime,
    pub name: Option<String>,
    pub event_type: EventType,
}

impl Display for ListenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
