//! macOS 系统工具实现：CoreGraphics / CoreFoundation / libproc。
//! 与 `windows.rs` 保持相同的对外接口，供 config / stats / ui / net 共用。
#![allow(dead_code)]
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicI8, Ordering};
use std::time::{Duration, Instant};

use core_graphics::display::CGDisplay;

pub type Rect = (i32, i32, i32, i32); // left, top, right, bottom

// ---------------- CoreFoundation / CoreGraphics FFI ----------------

type CFTypeRef = *const c_void;
type CFArrayRef = *const c_void;
type CFDictionaryRef = *const c_void;
type CFStringRef = *const c_void;

#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: CFTypeRef);
    fn CFArrayGetCount(array: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, idx: isize) -> CFTypeRef;
    fn CFDictionaryGetValue(dict: CFDictionaryRef, key: CFTypeRef) -> CFTypeRef;
    fn CFNumberGetValue(num: CFTypeRef, the_type: i32, value_ptr: *mut c_void) -> u8;
    fn CFStringCreateWithCString(alloc: CFTypeRef, c_str: *const i8, encoding: u32) -> CFStringRef;
    fn CFBooleanGetValue(b: CFTypeRef) -> u8;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative: u32) -> CFArrayRef;
    fn CGEventCreate(source: CFTypeRef) -> CFTypeRef;
    fn CGEventGetLocation(event: CFTypeRef) -> CGPoint;
    fn CGSessionCopyCurrentDictionary() -> CFDictionaryRef;
    fn CGEventSourceFlagsState(state: i32) -> u64;
    static kCGWindowOwnerPID: CFStringRef;
}

extern "C" {
    fn proc_pidpath(pid: i32, buffer: *mut c_void, buffersize: u32) -> i32;
}

const WINDOW_LIST_ON_SCREEN_ONLY: u32 = 1 << 0;
const WINDOW_LIST_EXCLUDE_DESKTOP: u32 = 1 << 4;
const CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
const CF_NUMBER_SINT64: i32 = 4;
/// kCGEventSourceStateCombinedSessionState
const EVENT_SOURCE_COMBINED: i32 = 0;
const FLAG_ALPHA_SHIFT: u64 = 0x0001_0000; // CapsLock
const FLAG_NUMERIC_PAD: u64 = 0x0020_0000; // NumLock（外接键盘）

// ---------------- 显示器 / 光标 ----------------

/// 显示器缩放系数（物理像素 / points）：Retina 屏为 2.0，普通屏 1.0。
/// 注意 CGDisplay::pixels_wide() 在部分系统版本上返回的仍是 points 尺寸，
/// 必须用 CGDisplayModeGetPixelWidth（真实帧缓冲像素）计算。
fn display_scale(d: &CGDisplay) -> f64 {
    let b = d.bounds();
    if b.size.width > 0.0 {
        if let Some(mode) = d.display_mode() {
            let pw = mode.pixel_width() as f64;
            if pw > 0.0 {
                let s = pw / b.size.width;
                if s > 0.0 {
                    return s;
                }
            }
        }
    }
    1.0
}

/// 枚举全部显示器（返回物理像素矩形，与 Windows GetMonitorInfo 一致；
/// CoreGraphics bounds 为 points，需乘以各屏 Retina 缩放系数）
pub fn monitors() -> Vec<Rect> {
    let mut list = Vec::new();
    if let Ok(ids) = CGDisplay::active_displays() {
        for id in ids {
            let d = CGDisplay::new(id);
            let b = d.bounds();
            if b.size.width <= 0.0 || b.size.height <= 0.0 {
                continue;
            }
            let s = display_scale(&d);
            list.push((
                (b.origin.x * s) as i32,
                (b.origin.y * s) as i32,
                ((b.origin.x + b.size.width) * s) as i32,
                ((b.origin.y + b.size.height) * s) as i32,
            ));
        }
    }
    if list.is_empty() {
        let d = CGDisplay::main();
        let b = d.bounds();
        let s = display_scale(&d);
        list.push((
            (b.origin.x * s) as i32,
            (b.origin.y * s) as i32,
            ((b.origin.x + b.size.width) * s) as i32,
            ((b.origin.y + b.size.height) * s) as i32,
        ));
    }
    list
}

/// points 坐标所在显示器的缩放系数（找不到时取主屏）
fn scale_at_point_pts(x: f64, y: f64) -> f64 {
    if let Ok(ids) = CGDisplay::active_displays() {
        for id in ids {
            let d = CGDisplay::new(id);
            let b = d.bounds();
            if x >= b.origin.x
                && x < b.origin.x + b.size.width
                && y >= b.origin.y
                && y < b.origin.y + b.size.height
            {
                return display_scale(&d);
            }
        }
    }
    display_scale(&CGDisplay::main())
}

pub fn virtual_screen() -> (i32, i32, i32, i32) {
    let m = monitors();
    (
        m.iter().map(|r| r.0).min().unwrap_or(0),
        m.iter().map(|r| r.1).min().unwrap_or(0),
        m.iter().map(|r| r.2).max().unwrap_or(1920),
        m.iter().map(|r| r.3).max().unwrap_or(1080),
    )
}

pub fn cursor_pos() -> Option<(i32, i32)> {
    unsafe {
        let ev = CGEventCreate(std::ptr::null());
        if ev.is_null() {
            return None;
        }
        let p = CGEventGetLocation(ev);
        CFRelease(ev);
        // 返回物理像素坐标（与 monitors() 一致，供鼠标距离统计与 active_monitor 使用）
        let s = scale_at_point_pts(p.x, p.y);
        Some(((p.x * s) as i32, (p.y * s) as i32))
    }
}

/// macOS 没有"窗口句柄"概念；保留接口以兼容共用代码
pub fn foreground_hwnd() -> isize {
    0
}

/// 前台窗口所属进程 ID（0 表示无前台窗口）
pub fn foreground_pid() -> u32 {
    unsafe {
        let arr = CGWindowListCopyWindowInfo(
            WINDOW_LIST_ON_SCREEN_ONLY | WINDOW_LIST_EXCLUDE_DESKTOP,
            0,
        );
        if arr.is_null() {
            return 0;
        }
        let mut pid = 0u32;
        if CFArrayGetCount(arr) > 0 {
            let dict = CFArrayGetValueAtIndex(arr, 0);
            if !dict.is_null() {
                let val = CFDictionaryGetValue(dict, kCGWindowOwnerPID);
                if !val.is_null() {
                    let mut n: i64 = 0;
                    if CFNumberGetValue(val, CF_NUMBER_SINT64, &mut n as *mut i64 as *mut c_void) != 0
                    {
                        pid = n as u32;
                    }
                }
            }
        }
        CFRelease(arr);
        pid
    }
}

/// 前台窗口所在屏幕（用光标位置判断，桌面 / 密码框等场景与 Windows 一致地退化为光标）
pub fn active_monitor(monitors: &[Rect]) -> usize {
    if let Some((x, y)) = cursor_pos() {
        for (i, (l, t, r, b)) in monitors.iter().enumerate() {
            if x >= *l && x < *r && y >= *t && y < *b {
                return i;
            }
        }
    }
    0
}

/// 进程可执行文件完整路径
pub fn process_path_by_pid(pid: u32) -> String {
    let mut buf = [0u8; 4096];
    let n = unsafe { proc_pidpath(pid as i32, buf.as_mut_ptr() as *mut c_void, buf.len() as u32) };
    if n <= 0 {
        return String::new();
    }
    String::from_utf8_lossy(&buf[..n as usize]).to_string()
}

/// 前台窗口进程可执行文件完整路径（对应 AHK GetProcPath）
pub fn foreground_proc_path() -> String {
    let pid = foreground_pid();
    if pid == 0 {
        return "NotFoundActive".into();
    }
    let path = process_path_by_pid(pid);
    if path.is_empty() {
        format!("PID:{}", pid)
    } else {
        path
    }
}

/// macOS 没有 ES_PASSWORD 样式，密码框检测不适用（hideInWinPwd 在 macOS 下不生效）
pub fn focus_is_password() -> bool {
    false
}

/// 系统是否锁定（CGSSessionScreenIsLocked）
pub fn system_locked() -> bool {
    unsafe {
        let dict = CGSessionCopyCurrentDictionary();
        if dict.is_null() {
            return false;
        }
        let key = CFStringCreateWithCString(
            std::ptr::null(),
            c"CGSSessionScreenIsLocked".as_ptr(),
            CF_STRING_ENCODING_UTF8,
        );
        let mut locked = false;
        if !key.is_null() {
            let val = CFDictionaryGetValue(dict, key);
            if !val.is_null() {
                locked = CFBooleanGetValue(val) != 0;
            }
            CFRelease(key);
        }
        CFRelease(dict);
        locked
    }
}

// ---------------- 时间 ----------------

pub fn now_string(minutes_only: bool) -> String {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        if minutes_only {
            format!(
                "{:04}{:02}{:02}{:02}{:02}",
                tm.tm_year + 1900,
                tm.tm_mon + 1,
                tm.tm_mday,
                tm.tm_hour,
                tm.tm_min
            )
        } else {
            format!(
                "{:04}{:02}{:02}{:02}{:02}{:02}",
                tm.tm_year + 1900,
                tm.tm_mon + 1,
                tm.tm_mday,
                tm.tm_hour,
                tm.tm_min,
                tm.tm_sec
            )
        }
    }
}

/// 距离下一个本地 0:00:05 的秒数（对应 AHK ShutDownLeft：DateDiff(明天0点, 现在) + 5 秒余量）
/// 注意：与 AHK 一致，无论当前几点都等到“下一个 0:00:05”，
/// 这样重启后新实例不会立刻再次触发（否则 0:00:05 附近会陷入重启循环）。
pub fn secs_until_next_restart() -> u64 {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        let passed = (tm.tm_hour as u64) * 3600 + (tm.tm_min as u64) * 60 + tm.tm_sec as u64;
        86400 - passed + 5
    }
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ---------------- 字体 / 文本测量 ----------------

/// macOS 无法在运行期注册字体文件；返回 false 让调用方按字体族名处理
pub fn add_font_resource(_path: &std::path::Path) -> bool {
    false
}

/// 系统是否有该字体族：交给 Slint/CoreText 自行回退，这里只排除 Windows 专有字体
pub fn font_family_exists(name: &str) -> bool {
    !name.is_empty()
        && !name.eq_ignore_ascii_case("Segoe UI Symbol")
        && !name.eq_ignore_ascii_case("Tahoma")
}

/// 文本是否包含需要 SMP 字体回退的符号（🖰 🖱 ⎇ ⇧ 等）
pub fn needs_symbol_font(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c as u32,
            0x2190..=0x21FF
            | 0x2200..=0x23FF
            | 0x2400..=0x243F
            | 0x25A0..=0x25FF
            | 0x2700..=0x27BF
            | 0x2B00..=0x2BFF
            | 0x1F000..=0x1FAFF
        )
    })
}

pub fn strip_variation_selectors(text: &str) -> String {
    text.chars()
        .filter(|c| !matches!(*c, '\u{FE0E}' | '\u{FE0F}'))
        .collect()
}

/// 物理像素坐标所在显示器的 DPI 缩放（Retina=2.0；找不到所在屏返回 0 让调用方回退）
pub fn scale_at(x: i32, y: i32) -> f32 {
    if let Ok(ids) = CGDisplay::active_displays() {
        for id in ids {
            let d = CGDisplay::new(id);
            let b = d.bounds();
            let s = display_scale(&d);
            let (l, t, r, bt) = (
                b.origin.x * s,
                b.origin.y * s,
                (b.origin.x + b.size.width) * s,
                (b.origin.y + b.size.height) * s,
            );
            if (x as f64) >= l && (x as f64) < r && (y as f64) >= t && (y as f64) < bt {
                return s as f32;
            }
        }
    }
    0.0
}

/// macOS 不走 GDI 测量，返回 (0,0,0) 让调用方使用估算高度
pub fn measure_text(
    _text: &str,
    _family: &str,
    _font_px: i32,
    _bold: bool,
    _max_width: i32,
) -> (i32, i32, i32) {
    (0, 0, 0)
}

// ---------------- 按键状态 ----------------

/// 由输入线程维护的按键物理按下状态（VK 索引，对应 Windows 的 GetAsyncKeyState）
static KEY_DOWN: [AtomicBool; 256] = [const { AtomicBool::new(false) }; 256];

/// 输入线程在按键事件到达时更新（Windows 由系统维护，无需此函数）
pub fn set_key_down(vk: u32, down: bool) {
    if let Some(slot) = KEY_DOWN.get(vk as usize) {
        slot.store(down, Ordering::Relaxed);
    }
}

pub fn key_down(vk: i32) -> bool {
    if !(0..256).contains(&vk) {
        return false;
    }
    KEY_DOWN[vk as usize].load(Ordering::Relaxed)
}

static CAPS_TOGGLE: AtomicI8 = AtomicI8::new(-1);
static NUM_TOGGLE: AtomicI8 = AtomicI8::new(-1);
static SCROLL_TOGGLE: AtomicI8 = AtomicI8::new(-1);
static CAPS_DOWN: AtomicBool = AtomicBool::new(false);
static NUM_DOWN: AtomicBool = AtomicBool::new(false);
static SCROLL_DOWN: AtomicBool = AtomicBool::new(false);

fn toggle_slots(vk: i32) -> Option<(&'static AtomicI8, &'static AtomicBool)> {
    match vk {
        0x14 => Some((&CAPS_TOGGLE, &CAPS_DOWN)),
        0x90 => Some((&NUM_TOGGLE, &NUM_DOWN)),
        0x91 => Some((&SCROLL_TOGGLE, &SCROLL_DOWN)),
        _ => None,
    }
}

fn system_toggle(vk: i32) -> bool {
    let flags = unsafe { CGEventSourceFlagsState(EVENT_SOURCE_COMBINED) };
    match vk {
        0x14 => (flags & FLAG_ALPHA_SHIFT) != 0,
        0x90 => (flags & FLAG_NUMERIC_PAD) != 0,
        _ => false, // macOS 无 ScrollLock
    }
}

pub fn key_toggled(vk: i32) -> bool {
    let Some((slot, _down)) = toggle_slots(vk) else {
        return false;
    };
    let v = slot.load(Ordering::Relaxed);
    if v >= 0 {
        return v != 0;
    }
    let on = system_toggle(vk);
    slot.store(on as i8, Ordering::Relaxed);
    on
}

pub fn sync_lock_states() {
    for vk in [0x14i32, 0x90, 0x91] {
        if let Some((slot, _down)) = toggle_slots(vk) {
            slot.store(system_toggle(vk) as i8, Ordering::Relaxed);
        }
    }
}

/// 键盘事件到达时调用：在 up→down 跳变时翻转状态（忽略自动重复）
pub fn lock_key_event(vk: i32, is_down: bool) {
    let Some((slot, down)) = toggle_slots(vk) else {
        return;
    };
    if is_down {
        if !down.swap(true, Ordering::Relaxed) {
            let on = slot.load(Ordering::Relaxed) > 0;
            slot.store(!on as i8, Ordering::Relaxed);
        }
    } else {
        down.store(false, Ordering::Relaxed);
    }
}

// ---------------- 窗口样式 ----------------

/// macOS 浮窗样式：置顶 + 可选鼠标穿透。
/// 不抢焦点由 winit/NSWindow 的非激活浮层行为保证（不设置 owner/任务栏概念）。
pub fn apply_overlay_styles(win: &slint::Window, _title: &str, click_through: bool) -> bool {
    use slint::winit_030::winit::window::WindowLevel;
    use slint::winit_030::WinitWindowAccessor;
    win.with_winit_window(|w| {
        let _ = w.set_cursor_hittest(!click_through);
        w.set_window_level(WindowLevel::AlwaysOnTop);
    })
    .is_some()
}

pub fn open_url(url: &str) {
    let _ = std::process::Command::new("open").arg(url).spawn();
}

/// 带缓存的前台进程路径获取（避免每次按键都查询系统）
pub struct ProcPathCache {
    path: String,
    at: Instant,
    ttl: Duration,
}

impl ProcPathCache {
    pub fn new() -> Self {
        Self {
            path: String::new(),
            at: Instant::now() - Duration::from_secs(10),
            ttl: Duration::from_millis(200),
        }
    }
    pub fn get(&mut self) -> String {
        if self.at.elapsed() > self.ttl {
            self.path = foreground_proc_path();
            self.at = Instant::now();
        }
        self.path.clone()
    }
}
