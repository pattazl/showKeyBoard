//! Win32 工具函数：显示器、前台进程、密码框检测、锁屏检测、窗口样式等。
#![allow(dead_code)]
use std::mem;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, HWND, RECT, SYSTEMTIME};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MONITORINFO, MONITORINFOEXW, HDC, HMONITOR, TEXTMETRICW,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::SystemInformation::GetLocalTime;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::{
    AttachThreadInput, OpenProcess, QueryFullProcessImageNameW, GetCurrentThreadId,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, GetFocus, GetKeyState};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, FindWindowW, GetClassNameW, GetCursorPos,
    GetForegroundWindow, GetSystemMetrics, GetWindowTextW, GetWindowLongPtrW, GetWindowRect,
    GetWindowThreadProcessId, RegisterClassW, SetWindowLongPtrW, SetWindowPos,
    WNDCLASSW, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, GWL_EXSTYLE, GWLP_HWNDPARENT, GWL_STYLE,
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER, SWP_NOSIZE,
};

pub type Rect = (i32, i32, i32, i32); // left, top, right, bottom


/// 枚举全部显示器
pub fn monitors() -> Vec<Rect> {
    unsafe extern "system" fn callback(
        hmonitor: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        lparam: isize,
    ) -> i32 {
        let list = unsafe { &mut *(lparam as *mut Vec<Rect>) };
        let mut info: MONITORINFOEXW = unsafe { mem::zeroed() };
        info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
        let p: *mut MONITORINFO = &mut info.monitorInfo;
        if unsafe { GetMonitorInfoW(hmonitor, p) } != 0 {
            let r = info.monitorInfo.rcMonitor;
            list.push((r.left, r.top, r.right, r.bottom));
        }
        1
    }
    let mut list: Vec<Rect> = Vec::new();
    unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(callback),
            &mut list as *mut _ as isize,
        );
    }
    list
}

/// 虚拟桌面整体边界 (minLeft, minTop, maxRight, maxBottom)
pub fn virtual_screen() -> (i32, i32, i32, i32) {
    unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_XVIRTUALSCREEN) + GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN) + GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    }
}

pub fn cursor_pos() -> Option<(i32, i32)> {
    let (mut x, y) = (0i32, 0i32);
    unsafe {
        if GetCursorPos(&mut x as *mut i32 as *mut _) != 0 {
            Some((x, y))
        } else {
            None
        }
    }
}

/// 前台窗口句柄
pub fn foreground_hwnd() -> isize {
    unsafe { GetForegroundWindow() as isize }
}

/// 窗口标题
pub fn window_title(hwnd: isize) -> String {
    let mut buf = [0u16; 512];
    let n = unsafe { GetWindowTextW(hwnd as HWND, buf.as_mut_ptr(), 512) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

/// 窗口类名
pub fn window_class(hwnd: isize) -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { GetClassNameW(hwnd as HWND, buf.as_mut_ptr(), 256) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

/// 窗口位置大小
pub fn window_rect(hwnd: isize) -> Option<RECT> {
    let mut r: RECT = unsafe { mem::zeroed() };
    if unsafe { GetWindowRect(hwnd as HWND, &mut r) } != 0 {
        Some(r)
    } else {
        None
    }
}

/// 前台窗口所在屏幕（AHK GetActiveWindowScreenNumber 逻辑：桌面用鼠标位置，边界偏移 -32）
pub fn active_monitor(monitors: &[Rect]) -> usize {
    let offset = -32;
    let hwnd = foreground_hwnd();
    let (x, y) = if hwnd != 0 {
        let cls = window_class(hwnd);
        if cls == "Progman" || cls == "WorkerW" {
            cursor_pos().unwrap_or((0, 0))
        } else if let Some(r) = window_rect(hwnd) {
            (r.left, r.top)
        } else {
            return 0;
        }
    } else {
        return 0;
    };
    for (i, (l, t, r, b)) in monitors.iter().enumerate() {
        let (l, t, r, b) = (l + offset, t + offset, r + offset, b + offset);
        if x >= l && x < r && y >= t && y < b {
            return i;
        }
    }
    0
}

/// 按进程名判断是否存在（如 LogonUI.exe 表示锁屏）
pub fn process_exists_by_name(name: &str) -> bool {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap.is_null() {
            return false;
        }
        let mut entry: PROCESSENTRY32W = mem::zeroed();
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut found = false;
        if Process32FirstW(snap, &mut entry) != 0 {
            loop {
                let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(0);
                let exe = String::from_utf16_lossy(&entry.szExeFile[..len]);
                if exe.eq_ignore_ascii_case(name) {
                    found = true;
                    break;
                }
                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
        found
    }
}


/// 系统是否锁定（存在 LogonUI.exe 进程）
pub fn system_locked() -> bool {
    process_exists_by_name("LogonUI.exe")
}

/// 本地时间格式化 yyyyMMddHHmm(ss)
pub fn now_string(minutes_only: bool) -> String {
    unsafe {
        let mut st: SYSTEMTIME = mem::zeroed();
        GetLocalTime(&mut st);
        if minutes_only {
            format!(
                "{:04}{:02}{:02}{:02}{:02}",
                st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute
            )
        } else {
            format!(
                "{:04}{:02}{:02}{:02}{:02}{:02}",
                st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
            )
        }
    }
}

/// 当前 unix 毫秒时间戳
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// 前台窗口进程可执行文件完整路径（对应 AHK GetProcPath）
pub fn foreground_proc_path() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return "NotFoundActive".into();
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return "NotFoundActive".into();
        }
        let path = process_path_by_pid(pid);
        if path.contains("explorer.exe") {
            let cls = window_class(hwnd as isize);
            if cls == "Progman" || cls == "WorkerW" {
                return "Desktop".into();
            }
        }
        if path.contains("ApplicationFrameHost.exe") {
            let title = window_title(hwnd as isize);
            return format!("{}:{}", path, title.trim());
        }
        if path.is_empty() {
            let title = window_title(hwnd as isize).trim().to_string();
            if !title.is_empty() {
                return format!("Title:{}", title);
            }
            return "Unknown".into();
        }
        path
    }
}

pub fn process_path_by_pid(pid: u32) -> String {
    unsafe {
        let mut path = [0u16; 1024];
        let mut size = path.len() as u32;
        let h: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return String::new();
        }
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, path.as_mut_ptr(), &mut size);
        CloseHandle(h);
        if ok != 0 {
            String::from_utf16_lossy(&path[..size as usize])
        } else {
            String::new()
        }
    }
}

/// 前台焦点控件是否为密码框（ES_PASSWORD = 0x20）
pub fn focus_is_password() -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.is_null() {
            return false;
        }
        let tid = GetWindowThreadProcessId(fg, std::ptr::null_mut());
        let cur = GetCurrentThreadId();
        if AttachThreadInput(cur, tid, 1) == 0 {
            return false;
        }
        let focus: HWND = GetFocus();
        AttachThreadInput(cur, tid, 0);
        if focus.is_null() {
            return false;
        }
        let style = GetWindowLongPtrW(focus, GWL_STYLE);
        (style & 0x20) != 0
    }
}

/// 指定物理坐标所在显示器的 DPI 缩放（MDT_EFFECTIVE_DPI），失败返回 0.0
pub fn scale_at(x: i32, y: i32) -> f32 {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::{MonitorFromPoint, MONITOR_DEFAULTTONEAREST};
    use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
    unsafe {
        let hm = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
        if hm.is_null() {
            return 0.0;
        }
        let (mut dx, mut dy) = (0u32, 0u32);
        if GetDpiForMonitor(hm, MDT_EFFECTIVE_DPI, &mut dx, &mut dy) == 0 && dx > 0 {
            dx as f32 / 96.0
        } else {
            0.0
        }
    }
}

/// GDI 文本精确测量：按指定像素宽度换行（近似 word-wrap），
/// 返回 (行数, 行高, 最大行宽)，单位为物理像素；失败返回 (0, 0, 0)
pub fn measure_text(text: &str, family: &str, font_px: i32, bold: bool, max_width: i32) -> (i32, i32, i32) {
    use windows_sys::Win32::Foundation::SIZE;
    use windows_sys::Win32::Graphics::Gdi::{
        CreateFontW, DeleteObject, GetDC, GetTextExtentPoint32W, GetTextMetricsW, ReleaseDC,
        SelectObject, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, DEFAULT_CHARSET, FF_DONTCARE,
        FW_BOLD, FW_NORMAL, OUT_TT_PRECIS, HFONT,
    };
    if text.is_empty() || font_px <= 0 {
        return (0, 0, 0);
    }
    unsafe {
        let hdc = GetDC(std::ptr::null_mut());
        if hdc.is_null() {
            return (0, 0, 0);
        }
        let wname = wide(family);
        let hfont: HFONT = CreateFontW(
            -font_px, 0, 0, 0,
            if bold { FW_BOLD as i32 } else { FW_NORMAL as i32 },
            0, 0, 0, DEFAULT_CHARSET as u32, OUT_TT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32,
            CLEARTYPE_QUALITY as u32, FF_DONTCARE as u32, wname.as_ptr(),
        );
        let old = SelectObject(hdc, hfont);
        let mut tm: TEXTMETRICW = mem::zeroed();
        GetTextMetricsW(hdc, &mut tm);
        let line_h = if tm.tmHeight > 0 {
            tm.tmHeight + tm.tmExternalLeading
        } else {
            font_px * 3 / 2
        };
        let (mut lines, mut line_w, mut max_w) = (1i32, 0i32, 0i32);
        for ch in text.chars() {
            // GDI 不支持 SMP（代理对），按方块宽度估算
            if (ch as u32) > 0xFFFF {
                if max_width > 0 && line_w > 0 && line_w + font_px > max_width {
                    max_w = max_w.max(line_w);
                    lines += 1;
                    line_w = font_px;
                } else {
                    line_w += font_px;
                }
                continue;
            }
            let mut buf = [0u16; 2];
            let n = ch.len_utf16();
            ch.encode_utf16(&mut buf[..n]);
            let mut sz = SIZE { cx: 0, cy: 0 };
            if GetTextExtentPoint32W(hdc, buf.as_ptr(), n as i32, &mut sz) == 0 {
                continue;
            }
            if max_width > 0 && line_w > 0 && line_w + sz.cx > max_width {
                max_w = max_w.max(line_w);
                lines += 1;
                line_w = sz.cx;
            } else {
                line_w += sz.cx;
            }
        }
        max_w = max_w.max(line_w);
        SelectObject(hdc, old);
        DeleteObject(hfont);
        ReleaseDC(std::ptr::null_mut(), hdc);
        (lines, line_h.max(font_px), max_w)
    }
}

pub const WS_EX_TRANSPARENT_USIZE: usize = 0x20;
pub const WS_EX_LAYERED_USIZE: usize = 0x00080000;
pub const WS_EX_NOACTIVATE_USIZE: usize = 0x08000000;
pub const WS_EX_TOOLWINDOW_USIZE: usize = 0x80;
pub const WS_EX_APPWINDOW_USIZE: usize = 0x00040000;

thread_local! {
    /// 隐藏 owner 窗口（对应 AHK Gui 的 +Owner）
    static OWNER_HWND: std::cell::Cell<isize> = const { std::cell::Cell::new(0) };
}

unsafe extern "system" fn owner_wndproc(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// 创建（惰性）一个永不显示的 owner 窗口：被它拥有的窗口不会出现在任务栏 / Alt-Tab
fn owner_hwnd() -> HWND {
    OWNER_HWND.with(|c| {
        if c.get() != 0 {
            return c.get() as HWND;
        }
        unsafe {
            let cls = wide("SKBHiddenOwner");
            let hinst = GetModuleHandleW(std::ptr::null());
            let wc = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(owner_wndproc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinst,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: cls.as_ptr(),
            };
            RegisterClassW(&wc);
            let h = CreateWindowExW(
                0x0000_0080u32, // WS_EX_TOOLWINDOW
                cls.as_ptr(),
                cls.as_ptr(),
                0x8000_0000, // WS_POPUP
                0, 0, 0, 0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinst,
                std::ptr::null(),
            );
            c.set(h as isize);
            h
        }
    })
}

/// 按窗口标题设置浮窗样式（对应 AHK：+ToolWindow +Owner +AlwaysOnTop，可选点击穿透）。
/// 同时清除 WS_EX_APPWINDOW 并设置 owner，确保任务栏 / Alt-Tab 中不可见。
pub fn apply_overlay_styles(title: &str, click_through: bool) -> bool {
    unsafe {
        let wname = wide(title);
        let hwnd = FindWindowW(std::ptr::null(), wname.as_ptr());
        if hwnd.is_null() {
            return false;
        }
        let cur = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as usize;
        let mut style = cur & !WS_EX_APPWINDOW_USIZE;
        style |= WS_EX_NOACTIVATE_USIZE | WS_EX_TOOLWINDOW_USIZE;
        if click_through {
            // 关键：点击穿透必须 WS_EX_TRANSPARENT + WS_EX_LAYERED 组合，
            // 单独的 WS_EX_TRANSPARENT 只影响同线程窗口的命中测试，对其它进程无效。
            // 与 winit 官方 set_cursor_hittest(false) 的实现一致（window_state.rs）。
            style |= WS_EX_TRANSPARENT_USIZE | WS_EX_LAYERED_USIZE;
        } else {
            style &= !(WS_EX_TRANSPARENT_USIZE | WS_EX_LAYERED_USIZE);
        }
        let owner = owner_hwnd();
        let owner_ok = owner.is_null()
            || GetWindowLongPtrW(hwnd, GWLP_HWNDPARENT) as isize == owner as isize;
        // 已就位则跳过：避免每轮都 SetWindowPos 造成重绘闪烁
        if style == cur && owner_ok {
            return true;
        }
        // 只有 owner / 工具窗口这类"结构性"变更才需要 SWP_FRAMECHANGED，
        // 单纯恢复 WS_EX_TRANSPARENT（点击穿透）直接写样式即可，避免额外重绘
        let structural = ((cur ^ style) & (WS_EX_TOOLWINDOW_USIZE | WS_EX_APPWINDOW_USIZE)) != 0
            || !owner_ok;
        if style != cur {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style as isize);
        }
        if !owner_ok {
            SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, owner as isize);
        }
        if structural {
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }
        true
    }
}

/// 文本是否包含需要符号字体（Segoe UI Symbol）才能显示的 SMP 字符（🖰 🖱 🖲 等）
pub fn needs_symbol_font(text: &str) -> bool {
    text.chars().any(|c| {
        let cp = c as u32;
        // 杂项符号与象形文字 / 补充符号 / emoji（不含 CJK 扩展 B 及以后）
        (0x1F000..=0x1FAFF).contains(&cp)
    })
}

/// 系统是否安装了指定字体族（GDI 找不到时会回退到默认字体，名字不同即可判定）
pub fn font_family_exists(name: &str) -> bool {
    use windows_sys::Win32::Graphics::Gdi::{
        CreateFontW, DeleteObject, GetDC, GetTextFaceW, ReleaseDC, SelectObject, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_CHARSET, FF_DONTCARE, FW_NORMAL, OUT_TT_PRECIS,
    };
    unsafe {
        let hdc = GetDC(std::ptr::null_mut());
        if hdc.is_null() {
            return false;
        }
        let wname = wide(name);
        let hfont = CreateFontW(
            -12, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0,
            DEFAULT_CHARSET as u32, OUT_TT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32,
            CLEARTYPE_QUALITY as u32, FF_DONTCARE as u32, wname.as_ptr(),
        );
        let old = SelectObject(hdc, hfont);
        let mut buf = [0u16; 80];
        let n = GetTextFaceW(hdc, buf.len() as i32, buf.as_mut_ptr());
        SelectObject(hdc, old);
        DeleteObject(hfont);
        ReleaseDC(std::ptr::null_mut(), hdc);
        if n <= 1 {
            return false;
        }
        let got = String::from_utf16_lossy(&buf[..(n as usize - 1).min(buf.len())]);
        got.eq_ignore_ascii_case(name)
    }
}

/// 从 ICO 字节中挑选最合适的 32bpp 位图，返回 (RGBA, 宽, 高)。
/// 仅支持 BMP 内嵌条目（PNG 内嵌条目跳过），优先选择边长最接近 `prefer` 的条目。
pub fn ico_to_rgba(bytes: &[u8], prefer: u32) -> Option<(Vec<u8>, u32, u32)> {
    if bytes.len() < 6 {
        return None;
    }
    let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    let entry = |i: usize| -> Option<(i32, i32, u16, usize)> {
        let o = 6 + i * 16;
        if o + 16 > bytes.len() {
            return None;
        }
        let w = if bytes[o] == 0 { 256 } else { bytes[o] as i32 };
        let h = if bytes[o + 1] == 0 { 256 } else { bytes[o + 1] as i32 };
        let bpp = u16::from_le_bytes([bytes[o + 6], bytes[o + 7]]);
        let off = u32::from_le_bytes([bytes[o + 12], bytes[o + 13], bytes[o + 14], bytes[o + 15]]) as usize;
        Some((w, h, bpp, off))
    };
    let mut best: Option<(i32, usize)> = None;
    for i in 0..count {
        let Some((w, h, bpp, off)) = entry(i) else { break };
        if w <= 0 || w != h || bpp != 32 {
            continue;
        }
        let Some(data) = bytes.get(off..) else { continue };
        if data.len() < 40 || data.starts_with(&[0x89, b'P', b'N', b'G']) {
            continue;
        }
        let score = (w - prefer as i32).abs();
        if best.map(|(s, _)| score < s).unwrap_or(true) {
            best = Some((score, off));
        }
    }
    let (_, off) = best?;
    let data = bytes.get(off..)?;
    let bmp_w = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let bmp_h = i32::from_le_bytes([data[8], data[9], data[10], data[11]]) / 2;
    let header = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if bmp_w <= 0 || bmp_h <= 0 {
        return None;
    }
    let stride = bmp_w as usize * 4;
    let need = stride * bmp_h as usize;
    let pix = data.get(header..header + need)?;
    let mut rgba = Vec::with_capacity(need);
    // BMP 自底向上 + BGRA
    for y in (0..bmp_h as usize).rev() {
        for x in 0..bmp_w as usize {
            let i = y * stride + x * 4;
            rgba.extend_from_slice(&[pix[i + 2], pix[i + 1], pix[i], pix[i + 3]]);
        }
    }
    Some((rgba, bmp_w as u32, bmp_h as u32))
}

/// 按键状态（Async）
pub fn key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

/// 运行期注册字体文件（gdi32!AddFontResourceW，无需管理员权限，会话内有效）
pub fn add_font_resource(path: &std::path::Path) -> bool {
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn AddFontResourceW(lpfilename: *const u16) -> i32;
    }
    let w = wide(path.to_string_lossy().as_ref());
    unsafe { AddFontResourceW(w.as_ptr()) > 0 }
}

/// 切换键状态（CapsLock 等）
pub fn key_toggled(vk: i32) -> bool {
    unsafe { (GetKeyState(vk) as u16 & 1) != 0 }
}

/// 打开 URL（设置 / 统计页面）
pub fn open_url(url: &str) {
    use std::os::windows::process::CommandExt;
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn();
}

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
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
