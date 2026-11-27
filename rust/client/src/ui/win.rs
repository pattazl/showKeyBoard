//! Windows 原生浮窗：WS_EX_LAYERED + LWA_COLORKEY/LWA_ALPHA + GDI 绘制。
//!
//! 与 AHK 原版一致：窗口背景色作为颜色键透明，整体不透明度用 layered alpha 控制，
//! 文本/边框由 GDI 绘制。窗口按需创建、淡出后立即销毁，不做预设窗口池，
//! 因此不存在"复用旧渲染表面导致全透明"的问题。

use std::cell::RefCell;
use std::sync::atomic::{AtomicIsize, Ordering};

use windows_sys::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateRoundRectRgn, CreateSolidBrush, DeleteObject,
    DrawTextW, EndPaint, FillRect, GetStockObject, InvalidateRect, RoundRect, SelectObject,
    SetBkMode, SetTextColor, SetWindowRgn, UpdateWindow, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
    DEFAULT_PITCH, DT_NOPREFIX, DT_WORDBREAK, FF_DONTCARE, HGDIOBJ, NULL_BRUSH,
    OUT_DEFAULT_PRECIS, PAINTSTRUCT, PS_SOLID, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowLongPtrW, PostMessageW, RegisterClassW, SetLayeredWindowAttributes, SetTimer,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, GWL_EXSTYLE, GWLP_USERDATA,
    HWND_TOPMOST, LWA_ALPHA, LWA_COLORKEY, MA_NOACTIVATE, MSG, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE, WM_ERASEBKGND,
    WM_MOUSEACTIVATE, WM_NCDESTROY, WM_PAINT, WM_QUIT, WM_TIMER, WNDCLASSW, WS_EX_APPWINDOW,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

use super::Color;

const CLASS_NAME: &str = "SKBPopupNative";
const LOOP_CLASS: &str = "SKBUiLoop";

/// 不透明背景时使用的颜色键（正常绘制不会出现的颜色）
const CANARY_KEY: COLORREF = 0x00FF_00FF; // RGB(255, 0, 255)

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn rgb(c: &Color) -> COLORREF {
    (c.r as u32) | ((c.g as u32) << 8) | ((c.b as u32) << 16)
}

struct State {
    label: String,
    bg: Color,
    txt: Color,
    edge: Color,
    show_edge: bool,
    font_name: String,
    font_size: i32,
    font_weight: i32,
    radius: f32,
    scale: f32,
    fade: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            label: String::new(),
            bg: Color::default(),
            txt: Color::default(),
            edge: Color::default(),
            show_edge: true,
            font_name: "Tahoma".to_string(),
            font_size: 26,
            font_weight: 400,
            radius: 0.0,
            scale: 1.0,
            fade: 1.0,
        }
    }
}

/// 一个原生浮窗。Drop 时销毁 HWND。
pub struct PopupWindow {
    hwnd: HWND,
}

impl PopupWindow {
    pub fn new(_id: usize) -> Option<Self> {
        register_popup_class();
        let hinst = unsafe { GetModuleHandleW(std::ptr::null()) };
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
                wide(CLASS_NAME).as_ptr(),
                wide(CLASS_NAME).as_ptr(),
                WS_POPUP,
                0,
                0,
                1,
                1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinst,
                std::ptr::null(),
            )
        };
        if hwnd.is_null() {
            return None;
        }
        let state = Box::into_raw(Box::new(State::default()));
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
            SetLayeredWindowAttributes(hwnd, CANARY_KEY, 255, LWA_COLORKEY | LWA_ALPHA);
        }
        Some(Self { hwnd })
    }

    fn with_state(&self, f: impl FnOnce(&mut State)) {
        if self.hwnd.is_null() {
            return;
        }
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut State;
            if !ptr.is_null() {
                f(&mut *ptr);
            }
        }
    }

    fn with_state_ret<T>(&self, f: impl FnOnce(&State) -> T) -> Option<T> {
        if self.hwnd.is_null() {
            return None;
        }
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *const State;
            if ptr.is_null() {
                None
            } else {
                Some(f(&*ptr))
            }
        }
    }

    pub fn set_label(&self, s: &str) {
        self.with_state(|st| st.label = s.to_string());
        self.invalidate();
    }

    pub fn set_bg_color(&self, c: Color) {
        self.with_state(|st| st.bg = c);
        self.apply_layered();
        self.invalidate();
    }

    pub fn set_txt_color(&self, c: Color) {
        self.with_state(|st| st.txt = c);
        self.apply_layered();
        self.invalidate();
    }

    pub fn set_edge_color(&self, c: Color) {
        self.with_state(|st| st.edge = c);
        self.invalidate();
    }

    pub fn set_show_edge(&self, b: bool) {
        self.with_state(|st| st.show_edge = b);
        self.invalidate();
    }

    pub fn set_font_name(&self, s: &str) {
        self.with_state(|st| st.font_name = s.to_string());
        self.invalidate();
    }

    pub fn set_font_size(&self, px: i32) {
        self.with_state(|st| st.font_size = px);
        self.invalidate();
    }

    pub fn set_font_weight(&self, w: i32) {
        self.with_state(|st| st.font_weight = w);
        self.invalidate();
    }

    pub fn set_corner_radius(&self, r: f32) {
        self.with_state(|st| st.radius = r.max(0.0));
        let (w, h) = self.client_size();
        self.apply_region(w, h);
        self.invalidate();
    }

    pub fn set_scale(&self, scale: f32) {
        self.with_state(|st| st.scale = if scale > 0.0 { scale } else { 1.0 });
    }

    pub fn set_fade(&self, f: f32) {
        self.with_state(|st| st.fade = f.clamp(0.0, 1.0));
        self.apply_layered();
        self.invalidate();
    }

    pub fn set_position(&self, x: i32, y: i32) {
        unsafe {
            SetWindowPos(
                self.hwnd,
                std::ptr::null_mut(),
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
            );
        }
    }

    pub fn set_size(&self, w: u32, h: u32) {
        unsafe {
            SetWindowPos(
                self.hwnd,
                std::ptr::null_mut(),
                0,
                0,
                w.max(1) as i32,
                h.max(1) as i32,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
            );
        }
        self.apply_region(w.max(1) as i32, h.max(1) as i32);
        self.invalidate();
    }

    /// 一次性设置位置与尺寸（原生化后无"窗口未就绪"问题，直接生效）。
    /// `_top_anchored` 仅 macOS 使用（Y 表示菜单栏下方可视区偏移），Windows 忽略。
    pub fn set_frame(&self, x: i32, y: i32, w: u32, h: u32, _scale: f32, _top_anchored: bool) {
        unsafe {
            SetWindowPos(
                self.hwnd,
                std::ptr::null_mut(),
                x,
                y,
                w.max(1) as i32,
                h.max(1) as i32,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
            );
        }
        self.apply_region(w.max(1) as i32, h.max(1) as i32);
        self.invalidate();
    }

    pub fn request_redraw(&self) {
        self.invalidate();
        unsafe {
            UpdateWindow(self.hwnd);
        }
    }

    pub fn show(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            // 每次显示都重新置顶：休眠唤醒后 Z-order 可能被系统重置
            SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
            );
        }
        self.invalidate();
    }

    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    pub fn apply_styles(&self, click_through: bool) {
        if self.hwnd.is_null() {
            return;
        }
        unsafe {
            let cur = GetWindowLongPtrW(self.hwnd, GWL_EXSTYLE) as u32;
            let mut style = cur & !(WS_EX_APPWINDOW | WS_EX_TRANSPARENT);
            style |= WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_LAYERED;
            if click_through {
                style |= WS_EX_TRANSPARENT;
            }
            if style != cur {
                SetWindowLongPtrW(self.hwnd, GWL_EXSTYLE, style as isize);
            }
        }
    }

    fn client_size(&self) -> (i32, i32) {
        let mut rc: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        unsafe {
            GetClientRect(self.hwnd, &mut rc);
        }
        (rc.right, rc.bottom)
    }

    fn apply_region(&self, w: i32, h: i32) {
        let radius = self
            .with_state_ret(|st| (st.radius * st.scale).round() as i32)
            .unwrap_or(0);
        unsafe {
            if radius > 0 {
                let rgn = CreateRoundRectRgn(0, 0, w + 1, h + 1, radius * 2, radius * 2);
                SetWindowRgn(self.hwnd, rgn, 1);
            } else {
                SetWindowRgn(self.hwnd, std::ptr::null_mut(), 1);
            }
        }
    }

    /// 统一设置 layered 属性：颜色键 + 整体 alpha（guiOpacity × fade）
    fn apply_layered(&self) {
        let Some((key, alpha)) = self.with_state_ret(|st| {
            let key = if st.bg.a == 0 { rgb(&st.bg) } else { CANARY_KEY };
            let alpha = ((st.txt.a as f32) * st.fade).round().clamp(0.0, 255.0) as u8;
            (key, alpha)
        }) else {
            return;
        };
        unsafe {
            SetLayeredWindowAttributes(self.hwnd, key, alpha, LWA_COLORKEY | LWA_ALPHA);
        }
    }

    fn invalidate(&self) {
        if self.hwnd.is_null() {
            return;
        }
        unsafe {
            InvalidateRect(self.hwnd, std::ptr::null(), 0);
        }
    }
}

impl Drop for PopupWindow {
    fn drop(&mut self) {
        if !self.hwnd.is_null() {
            unsafe {
                DestroyWindow(self.hwnd);
            }
        }
    }
}

unsafe extern "system" fn popup_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            paint(hwnd);
            0
        }
        WM_ERASEBKGND => 1,
        WM_MOUSEACTIVATE => MA_NOACTIVATE as LRESULT,
        WM_NCDESTROY => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut State;
            if !ptr.is_null() {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                drop(Box::from_raw(ptr));
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn paint(hwnd: HWND) {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
    if ptr.is_null() {
        return;
    }
    let st = &*ptr;
    let mut ps: PAINTSTRUCT = std::mem::zeroed();
    let hdc = BeginPaint(hwnd, &mut ps);
    let mut rc: RECT = std::mem::zeroed();
    GetClientRect(hwnd, &mut rc);
    let w = rc.right.max(1);
    let h = rc.bottom.max(1);

    // 背景整面填色：透明时填的颜色正好是颜色键，会被系统抠掉
    let brush = CreateSolidBrush(rgb(&st.bg));
    FillRect(hdc, &rc, brush);
    DeleteObject(brush as HGDIOBJ);

    let radius = (st.radius * st.scale).round() as i32;
    // 边框（实色背景与透明背景都画）
    if st.show_edge {
        let p = CreatePen(PS_SOLID as i32, 1, rgb(&st.edge));
        let b = GetStockObject(NULL_BRUSH as i32);
        let op = SelectObject(hdc, p as HGDIOBJ);
        let ob = SelectObject(hdc, b);
        RoundRect(hdc, 0, 0, w, h, radius * 2, radius * 2);
        SelectObject(hdc, op);
        SelectObject(hdc, ob);
        DeleteObject(p as HGDIOBJ);
    }
    // 文本
    if !st.label.is_empty() {
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, rgb(&st.txt));
        let font_px = ((st.font_size as f32) * st.scale).round().max(1.0) as i32;
        let fam = wide(&st.font_name);
        let hfont = CreateFontW(
            -font_px,
            0,
            0,
            0,
            st.font_weight,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            5, // CLEARTYPE_QUALITY
            (DEFAULT_PITCH | FF_DONTCARE) as u32,
            fam.as_ptr(),
        );
        let old = SelectObject(hdc, hfont as HGDIOBJ);
        let pad = (4.0 * st.scale).round() as i32;
        let pad_y = (5.0 * st.scale).round() as i32;
        let mut tr = RECT {
            left: pad,
            top: pad_y,
            right: w - pad,
            bottom: h - pad_y,
        };
        let text = wide(&st.label);
        DrawTextW(hdc, text.as_ptr(), -1, &mut tr, DT_WORDBREAK | DT_NOPREFIX);
        SelectObject(hdc, old);
        DeleteObject(hfont as HGDIOBJ);
    }
    EndPaint(hwnd, &ps);
}

fn register_popup_class() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| unsafe {
        let hinst = GetModuleHandleW(std::ptr::null());
        // 类名必须活过 RegisterClassW：不能用临时 Vec 的指针（悬垂）
        let class_name = wide(CLASS_NAME);
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(popup_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        RegisterClassW(&wc);
    });
}

// ---------------- UI 主循环（Win32 消息泵 + 20ms 定时器） ----------------

static LOOP_HWND: AtomicIsize = AtomicIsize::new(0);

thread_local! {
    static TICK: RefCell<Option<Box<dyn FnMut()>>> = const { RefCell::new(None) };
}

unsafe extern "system" fn loop_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TIMER => {
            TICK.with(|t| {
                if let Some(f) = t.borrow_mut().as_mut() {
                    f();
                }
            });
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// 在当前线程跑 Windows 消息循环，每 20ms 调用一次 tick。
pub fn run_loop(tick: Box<dyn FnMut()>) {
    unsafe {
        let hinst = GetModuleHandleW(std::ptr::null());
        // 类名/窗口名必须活过注册与创建调用，不能用临时 Vec 的指针
        let class_name = wide(LOOP_CLASS);
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(loop_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        RegisterClassW(&wc);
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinst,
            std::ptr::null(),
        );
        LOOP_HWND.store(hwnd as isize, Ordering::Relaxed);
        TICK.with(|t| *t.borrow_mut() = Some(tick));
        SetTimer(hwnd, 1, 20, None);
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// 退出消息循环（可从任意线程调用）。
pub fn request_exit() {
    let h = LOOP_HWND.load(Ordering::Relaxed);
    if h != 0 {
        unsafe {
            PostMessageW(h as HWND, WM_QUIT, 0, 0);
        }
    }
}
