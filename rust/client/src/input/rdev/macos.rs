//! CGEventTap 全局事件监听（macOS）。
//! 合并自 rdev 0.5.3 的 macos/listen.rs + macos/common.rs（MIT, © Nicolas Patry）。
//! 本项目补丁：`convert()` 不再做 TIS 输入源字符转换（新版 macOS 主队列断言
//! 在 tap 回调线程触发即 SIGTRAP；下游不使用 `Event::name`），也去掉了随之
//! 依赖的 Keyboard/KEYBOARD_STATE（原 crate 的 lazy_static 状态）。
#![allow(non_upper_case_globals, improper_ctypes, improper_ctypes_definitions)]

use super::keycodes::key_from_code;
use super::types::{Button, Event, EventType, ListenError};
use cocoa::base::{id, nil};
use cocoa::foundation::NSAutoreleasePool;
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, EventField,
};
use std::convert::TryInto;
use std::os::raw::c_void;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::SystemTime;

/// 滚动事件来源（convert 里同步记录，on_event 同线程紧随其后读取，值准确）：
/// 0 = 真鼠标滚轮（一格一条消息）；1 = 触控板主动滚动；2 = 触控板惯性滚动。
/// 依据 CGEventTypes.h：kCGScrollWheelEventScrollPhase = 99、
/// kCGScrollWheelEventMomentumPhase = 123（core-graphics 0.24 未封装常量，传原始值）。
static SCROLL_KIND: AtomicI32 = AtomicI32::new(0);

pub fn last_scroll_kind() -> i32 {
    SCROLL_KIND.load(Ordering::Relaxed)
}

pub type CFMachPortRef = *const c_void;
pub type CFIndex = u64;
pub type CFAllocatorRef = id;
pub type CFRunLoopSourceRef = id;
pub type CFRunLoopRef = id;
pub type CFRunLoopMode = id;
pub type CGEventTapProxy = id;
pub type CGEventRef = CGEvent;

// https://developer.apple.com/documentation/coregraphics/cgeventtapplacement?language=objc
pub type CGEventTapPlacement = u32;
#[allow(non_upper_case_globals)]
pub const kCGHeadInsertEventTap: u32 = 0;

// https://developer.apple.com/documentation/coregraphics/cgeventtapoptions?language=objc
#[allow(non_upper_case_globals)]
#[repr(u32)]
pub enum CGEventTapOption {
    ListenOnly = 1,
}

pub static mut LAST_FLAGS: CGEventFlags = CGEventFlags::CGEventFlagNull;

// https://developer.apple.com/documentation/coregraphics/cgeventmask?language=objc
pub type CGEventMask = u64;
#[allow(non_upper_case_globals)]
pub const kCGEventMaskForAllEvents: u64 = (1 << CGEventType::LeftMouseDown as u64)
    + (1 << CGEventType::LeftMouseUp as u64)
    + (1 << CGEventType::RightMouseDown as u64)
    + (1 << CGEventType::RightMouseUp as u64)
    + (1 << CGEventType::MouseMoved as u64)
    + (1 << CGEventType::LeftMouseDragged as u64)
    + (1 << CGEventType::RightMouseDragged as u64)
    + (1 << CGEventType::KeyDown as u64)
    + (1 << CGEventType::KeyUp as u64)
    + (1 << CGEventType::FlagsChanged as u64)
    + (1 << CGEventType::ScrollWheel as u64);

#[link(name = "Cocoa", kind = "framework")]
extern "C" {
    pub fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOption,
        eventsOfInterest: CGEventMask,
        callback: QCallback,
        user_info: id,
    ) -> CFMachPortRef;
    pub fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        tap: CFMachPortRef,
        order: CFIndex,
    ) -> CFRunLoopSourceRef;
    pub fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
    pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    pub fn CFRunLoopRun();

    pub static kCFRunLoopCommonModes: CFRunLoopMode;
}
pub type QCallback = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event)>> = None;

/// 回调线程里访问 static mut 统一走 addr_of_mut!（edition 2021 避免
/// static_mut_refs 引用产生的告警路径）
unsafe fn take_callback() -> Option<&'static mut dyn FnMut(Event)> {
    let slot = std::ptr::addr_of_mut!(GLOBAL_CALLBACK);
    (*slot).as_mut().map(|b| b.as_mut())
}

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    if let Some(event) = convert(_type, &cg_event) {
        if let Some(callback) = take_callback() {
            callback(event);
        }
    }
    cg_event
}

/// CGEvent -> rdev Event（含补丁：不再调用 TIS 转换 `name`）
unsafe fn convert(_type: CGEventType, cg_event: &CGEvent) -> Option<Event> {
    let option_type = match _type {
        CGEventType::LeftMouseDown => Some(EventType::ButtonPress(Button::Left)),
        CGEventType::LeftMouseUp => Some(EventType::ButtonRelease(Button::Left)),
        CGEventType::RightMouseDown => Some(EventType::ButtonPress(Button::Right)),
        CGEventType::RightMouseUp => Some(EventType::ButtonRelease(Button::Right)),
        CGEventType::MouseMoved => {
            let point = cg_event.location();
            Some(EventType::MouseMove {
                x: point.x,
                y: point.y,
            })
        }
        CGEventType::KeyDown => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            Some(EventType::KeyPress(key_from_code(code.try_into().ok()?)))
        }
        CGEventType::KeyUp => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            Some(EventType::KeyRelease(key_from_code(code.try_into().ok()?)))
        }
        CGEventType::FlagsChanged => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            let code = code.try_into().ok()?;
            let flags = cg_event.get_flags();
            if flags < LAST_FLAGS {
                LAST_FLAGS = flags;
                Some(EventType::KeyRelease(key_from_code(code)))
            } else {
                LAST_FLAGS = flags;
                Some(EventType::KeyPress(key_from_code(code)))
            }
        }
        CGEventType::ScrollWheel => {
            let delta_y =
                cg_event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1);
            let delta_x =
                cg_event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2);
            let scroll_phase = cg_event.get_integer_value_field(99);
            let momentum_phase = cg_event.get_integer_value_field(123);
            SCROLL_KIND.store(
                if momentum_phase > 0 {
                    2
                } else if scroll_phase > 0 {
                    1
                } else {
                    0
                },
                Ordering::Relaxed,
            );
            Some(EventType::Wheel { delta_x, delta_y })
        }
        _ => None,
    };
    if let Some(event_type) = option_type {
        // Patch: 原实现每个 KeyPress 都调用 TISCopyCurrentKeyboardInputSource /
        // TISGetInputSourceProperty（HIToolbox）。新版 macOS 上这些 API 断言必须在
        // 主队列调用，在 CGEventTap 回调线程里执行会 dispatch_assert_queue_fail
        // 直接 SIGTRAP 崩溃。本项目不使用 Event::name，故跳过按键字符转换。
        let name: Option<String> = None;
        return Some(Event {
            event_type,
            time: SystemTime::now(),
            name,
        });
    }
    None
}

pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    unsafe {
        let slot = std::ptr::addr_of_mut!(GLOBAL_CALLBACK);
        *slot = Some(Box::new(callback));
        let _pool = NSAutoreleasePool::new(nil);
        let tap = CGEventTapCreate(
            CGEventTapLocation::HID, // HID, Session, AnnotatedSession,
            kCGHeadInsertEventTap,
            CGEventTapOption::ListenOnly,
            kCGEventMaskForAllEvents,
            raw_callback,
            nil,
        );
        if tap.is_null() {
            return Err(ListenError::EventTapError);
        }
        let run_loop_source = CFMachPortCreateRunLoopSource(nil, tap, 0);
        if run_loop_source.is_null() {
            return Err(ListenError::LoopSourceError);
        }

        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, run_loop_source, kCFRunLoopCommonModes);

        CGEventTapEnable(tap, true);
        CFRunLoopRun();
    }
    Ok(())
}
