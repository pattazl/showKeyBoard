//! macOS 原生浮窗（NSWindow + NSView 自绘）与事件循环。
//!
//! 对应原 Slint 的 `KeyOverlay` / `CtrlOverlay`：
//! - 无边框、透明背景、始终置顶、不抢焦点、不进入 Mission Control
//! - 圆角背景 + 可选 1px 边框 + 自定义字体/颜色，文本垂直居中、左对齐（左右各 4pt 边距）
//! - 渐隐通过 NSWindow.alphaValue 实现
//! - 定位使用 Quartz 顶部原点物理像素直接换算，避免 AppKit 自动夹紧/翻转带来的偏移
//!
//! 所有调用都发生在 UI 主线程（run_loop 内），因此视图绘制状态保存在线程局部表中，
//! 由 `set_*` 写入、`drawRect:` 读取。
#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

use super::Color;

/// NSStatusWindowLevel：高于普通窗口，低于系统弹窗/菜单
const NS_WINDOW_LEVEL_STATUS: i64 = 25;
const NS_COLLECTION_CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
const NS_COLLECTION_STATIONARY: u64 = 1 << 4;
const NS_COLLECTION_FULLSCREEN_AUXILIARY: u64 = 1 << 8;
const NS_BACKING_BUFFERED: u64 = 2;
const NS_WINDOW_ANIMATION_NONE: i64 = 2;
/// NSFontTraitMask：Bold
const NS_FONT_BOLD_MASK: u64 = 2;
/// NSStringDrawingUsesLineFragmentOrigin | NSStringDrawingUsesFontLeading
const NS_STRING_DRAWING_OPTS: u64 = 1 | 2;

struct ViewState {
    label: String,
    bg: Color,
    txt: Color,
    edge: Color,
    show_edge: bool,
    font_name: String,
    font_size: f32,
    font_weight: i32,
    corner: f32,
}

thread_local! {
    /// 以 NSView 指针为键保存绘制状态（UI 全在主线程，无需加锁）
    static VIEW_STATES: RefCell<HashMap<usize, ViewState>> =
        RefCell::new(HashMap::new());
}

// ---------------- 绘制 ----------------

fn ns_color(c: Color) -> id {
    unsafe {
        msg_send![class!(NSColor),
            colorWithCalibratedRed: (c.r as f64) / 255.0
            green: (c.g as f64) / 255.0
            blue: (c.b as f64) / 255.0
            alpha: (c.a as f64) / 255.0]
    }
}

fn make_font(name: &str, size: f32, weight: i32) -> id {
    unsafe {
        let size = size.max(1.0) as f64;
        let bold = weight >= 600;
        if !name.is_empty() {
            let fm: id = msg_send![class!(NSFontManager), sharedFontManager];
            let family: id = NSString::alloc(nil).init_str(name);
            let traits: u64 = if bold { NS_FONT_BOLD_MASK } else { 0 };
            let w: i64 = if bold { 9 } else { 5 };
            let font: id = msg_send![fm,
                fontWithFamily: family
                traits: traits
                weight: w
                size: size];
            let _: () = msg_send![family, release];
            if font != nil {
                return font;
            }
        }
        if bold {
            msg_send![class!(NSFont), boldSystemFontOfSize: size]
        } else {
            msg_send![class!(NSFont), systemFontOfSize: size]
        }
    }
}

extern "C" fn is_opaque(_this: &mut Object, _cmd: Sel) -> bool {
    false
}

extern "C" fn draw_rect(this: &mut Object, _cmd: Sel, _dirty: NSRect) {
    unsafe {
        let bounds: NSRect = msg_send![this, bounds];
        let key = (this as *mut Object) as usize;
        let state = VIEW_STATES.with(|m| {
            m.borrow().get(&key).map(|s| {
                (
                    s.label.clone(),
                    s.bg,
                    s.txt,
                    s.edge,
                    s.show_edge,
                    s.font_name.clone(),
                    s.font_size,
                    s.font_weight,
                    s.corner,
                )
            })
        });
        let Some((
            label,
            bg,
            txt,
            edge,
            show_edge,
            font_name,
            font_size,
            font_weight,
            corner,
        )) = state
        else {
            return;
        };

        // 背景（圆角，alpha=0 表示透明背景）
        let path: id = msg_send![class!(NSBezierPath),
            bezierPathWithRoundedRect: bounds
            xRadius: corner as f64
            yRadius: corner as f64];
        if bg.a > 0 {
            let c = ns_color(bg);
            let _: () = msg_send![c, setFill];
            let _: () = msg_send![path, fill];
        }
        if show_edge {
            let c = ns_color(edge);
            let _: () = msg_send![c, setStroke];
            let _: () = msg_send![path, setLineWidth: 1.0f64];
            let _: () = msg_send![path, stroke];
        }
        if label.is_empty() {
            return;
        }

        // 文本属性：NSFontAttributeName / NSForegroundColorAttributeName
        let font = make_font(&font_name, font_size, font_weight);
        let color = ns_color(txt);
        let key_font: id = NSString::alloc(nil).init_str("NSFont");
        let key_color: id = NSString::alloc(nil).init_str("NSColor");
        let attrs: id = msg_send![class!(NSMutableDictionary), dictionary];
        let _: () = msg_send![attrs, setObject: font forKey: key_font];
        let _: () = msg_send![attrs, setObject: color forKey: key_color];
        let _: () = msg_send![key_font, release];
        let _: () = msg_send![key_color, release];

        let text: id = NSString::alloc(nil).init_str(&label);
        let max_w = (bounds.size.width - 8.0).max(1.0);
        let text_rect: NSRect = msg_send![text,
            boundingRectWithSize: NSSize::new(max_w, 1.0e7)
            options: NS_STRING_DRAWING_OPTS
            attributes: attrs];
        let th = text_rect.size.height.max(1.0);
        let ty = bounds.origin.y + ((bounds.size.height - th) * 0.5).max(0.0);
        let draw = NSRect::new(
            NSPoint::new(bounds.origin.x + 4.0, ty),
            NSSize::new(max_w, th),
        );
        let _: () = msg_send![text, drawInRect: draw withAttributes: attrs];
        let _: () = msg_send![text, release];
    }
}

fn view_class() -> &'static Class {
    static CLS: OnceLock<&'static Class> = OnceLock::new();
    CLS.get_or_init(|| unsafe {
        let mut decl = ClassDecl::new("SkbPopupView", class!(NSView))
            .expect("register SkbPopupView failed");
        decl.add_method(
            sel!(drawRect:),
            draw_rect as extern "C" fn(&mut Object, Sel, NSRect),
        );
        decl.add_method(
            sel!(isOpaque),
            is_opaque as extern "C" fn(&mut Object, Sel) -> bool,
        );
        decl.register()
    })
}

// ---------------- 浮窗 ----------------

pub struct PopupWindow {
    win: id,
    view: id,
}

impl PopupWindow {
    pub fn new(_id: usize) -> Option<PopupWindow> {
        unsafe {
            let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(10.0, 10.0));
            let win: id = msg_send![class!(NSWindow), alloc];
            // styleMask = 0（无边框）：不会抢焦点，也不显示标题栏
            let win: id = msg_send![win,
                initWithContentRect: frame
                styleMask: 0u64
                backing: NS_BACKING_BUFFERED
                defer: false];
            if win == nil {
                return None;
            }
            let _: () = msg_send![win, setReleasedWhenClosed: false];
            let _: () = msg_send![win, setOpaque: false];
            let clear: id = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![win, setBackgroundColor: clear];
            let _: () = msg_send![win, setHasShadow: false];
            let _: () = msg_send![win, setLevel: NS_WINDOW_LEVEL_STATUS];
            let _: () = msg_send![win,
                setCollectionBehavior: (NS_COLLECTION_CAN_JOIN_ALL_SPACES
                    | NS_COLLECTION_STATIONARY
                    | NS_COLLECTION_FULLSCREEN_AUXILIARY)];
            let _: () = msg_send![win, setHidesOnDeactivate: false];
            let _: () = msg_send![win, setIgnoresMouseEvents: true];
            let _: () = msg_send![win, setAnimationBehavior: NS_WINDOW_ANIMATION_NONE];

            let cls = view_class();
            let view: id = msg_send![cls, alloc];
            let view: id = msg_send![view, initWithFrame: frame];
            if view == nil {
                let _: () = msg_send![win, release];
                return None;
            }
            // setContentView: 会 retain，随后释放我们自己的引用
            let _: () = msg_send![win, setContentView: view];
            let _: () = msg_send![view, release];

            VIEW_STATES.with(|m| {
                m.borrow_mut().insert(
                    view as usize,
                    ViewState {
                        label: String::new(),
                        bg: Color::default(),
                        txt: Color::from_argb_u8(255, 255, 255, 255),
                        edge: Color::default(),
                        show_edge: false,
                        font_name: String::new(),
                        font_size: 20.0,
                        font_weight: 400,
                        corner: 0.0,
                    },
                );
            });
            Some(PopupWindow { win, view })
        }
    }

    fn update(&self, f: impl FnOnce(&mut ViewState)) {
        let key = self.view as usize;
        VIEW_STATES.with(|m| {
            if let Some(st) = m.borrow_mut().get_mut(&key) {
                f(st);
            }
        });
        unsafe {
            let _: () = msg_send![self.view, setNeedsDisplay: true];
        }
    }

    pub fn set_label(&self, s: &str) {
        let s = s.to_string();
        self.update(|st| st.label = s);
    }

    pub fn set_bg_color(&self, c: Color) {
        self.update(|st| st.bg = c);
    }

    pub fn set_txt_color(&self, c: Color) {
        self.update(|st| st.txt = c);
    }

    pub fn set_edge_color(&self, c: Color) {
        self.update(|st| st.edge = c);
    }

    pub fn set_show_edge(&self, b: bool) {
        self.update(|st| st.show_edge = b);
    }

    pub fn set_font_name(&self, name: &str) {
        let name = name.to_string();
        self.update(|st| st.font_name = name);
    }

    pub fn set_font_size(&self, px: i32) {
        self.update(|st| st.font_size = px as f32);
    }

    pub fn set_font_weight(&self, weight: i32) {
        self.update(|st| st.font_weight = weight);
    }

    /// 原生 AppKit 坐标本身就是点（pt），Retina 由系统处理，这里无需缩放
    pub fn set_scale(&self, _scale: f32) {}

    pub fn set_corner_radius(&self, r: f32) {
        self.update(|st| st.corner = r.max(0.0));
    }

    /// 用 Quartz 顶部原点物理像素定位（x 为屏幕左侧绝对坐标）。
    /// `y_from_visible_top=true` 时 y 表示「菜单栏下方可视区顶部」的偏移。
    pub fn set_frame(
        &self,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        scale: f32,
        y_from_visible_top: bool,
    ) {
        let scale = if scale > 0.0 { scale as f64 } else { 1.0 };
        let w_pt = (w as f64 / scale).round().max(1.0);
        let h_pt = (h as f64 / scale).round().max(1.0);
        let x_pt = (x as f64 / scale).round();
        let mut y_top_pt = (y as f64 / scale).round();
        if y_from_visible_top {
            y_top_pt +=
                crate::winutil::screen_visible_top_physical(x, y, scale) as f64 / scale;
        }
        // AppKit 全局坐标以主屏左下角为原点，Quartz 顶部原点需翻转
        let screen_h = core_graphics::display::CGDisplay::main().bounds().size.height;
        let y_appkit = screen_h - y_top_pt - h_pt;
        unsafe {
            let rect =
                NSRect::new(NSPoint::new(x_pt, y_appkit), NSSize::new(w_pt, h_pt));
            // 位置与尺寸一次提交，避免分步设置导致 frame 被重算
            let _: () = msg_send![self.win, setFrame: rect display: cocoa::base::YES];
        }
    }

    pub fn set_fade(&self, alpha: f32) {
        unsafe {
            let _: () = msg_send![self.win, setAlphaValue: alpha.clamp(0.0, 1.0) as f64];
        }
    }

    pub fn show(&self) {
        unsafe {
            let _: () = msg_send![self.win, orderFrontRegardless];
        }
    }

    pub fn hide(&self) {
        unsafe {
            let _: () = msg_send![self.win, orderOut: nil];
        }
    }

    /// 每帧复查窗口样式：点击穿透 + 始终置顶
    pub fn apply_styles(&self, click_through: bool) {
        unsafe {
            let _: () = msg_send![self.win, setIgnoresMouseEvents: click_through];
            let _: () = msg_send![self.win, setLevel: NS_WINDOW_LEVEL_STATUS];
        }
    }
}

impl Drop for PopupWindow {
    fn drop(&mut self) {
        let key = self.view as usize;
        unsafe {
            let _: () = msg_send![self.win, orderOut: nil];
            let _: () = msg_send![self.win, close];
            let _: () = msg_send![self.win, release];
        }
        VIEW_STATES.with(|m| {
            m.borrow_mut().remove(&key);
        });
    }
}

/// relayout 用：直接用原生 frame 重建窗口位置（不经过 winit 的坐标换算）
pub fn set_native_frame(
    comp: &PopupWindow,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    scale: f32,
    y_from_visible_top: bool,
) {
    comp.set_frame(
        x,
        y,
        w.max(1) as u32,
        h.max(1) as u32,
        scale,
        y_from_visible_top,
    );
}

// ---------------- 事件循环 ----------------

static EXIT: AtomicBool = AtomicBool::new(false);

/// 请求退出 UI 主循环（托盘菜单 / 跨夜重启使用）
pub fn request_exit() {
    EXIT.store(true, Ordering::Relaxed);
}

/// UI 主循环：每轮调用一次 `tick`，再派发一轮 AppKit 事件。
///
/// 不能用 CFRunLoopRunInMode 单纯泵 run loop：NSStatusItem 的点击、菜单弹出、
/// 悬浮提示都依赖 NSApplication 的事件分发（等价 `-[NSApplication run]` 内部的
/// nextEventMatchingMask + sendEvent），只泵 run loop 时图标可见但点击无响应。
/// 每轮都建独立的 autorelease pool，避免长期运行堆积自动释放对象。
pub fn run_loop(mut tick: Box<dyn FnMut()>) {
    unsafe {
        let app: id = msg_send![class!(NSApplication), sharedApplication];
        if app != nil {
            // NSApplicationActivationPolicyAccessory = 1（菜单栏应用）
            let _: () = msg_send![app, setActivationPolicy: 1i64];
            let _: () = msg_send![app, finishLaunching];
        }
        // NSDefaultRunLoopMode 的取值就是 "kCFRunLoopDefaultMode"
        let mode: id = NSString::alloc(nil).init_str("kCFRunLoopDefaultMode");
        while !EXIT.load(Ordering::Relaxed) {
            let pool: id = NSAutoreleasePool::new(nil);
            tick();
            // 最多等 20ms 取一个事件；取到就交给 AppKit 分发（状态栏菜单在此弹出）
            let until: id =
                msg_send![class!(NSDate), dateWithTimeIntervalSinceNow: 0.02f64];
            let event: id = msg_send![app,
                nextEventMatchingMask: u64::MAX
                untilDate: until
                inMode: mode
                dequeue: true];
            if event != nil {
                let _: () = msg_send![app, sendEvent: event];
            }
            let _: () = msg_send![app, updateWindows];
            let _: () = msg_send![pool, release];
        }
        let _: () = msg_send![mode, release];
    }
}
