//! Slint UI：按键浮窗（渐隐 / 圆角 / 自定义字体背景 / 多屏 / 方向排列）+ 控制键状态窗。
//! 对应 AHK dialog.ahk 的 ShowTxt / CreateGui / FadeUI / ReLayOut / ShowCtrlState。
#![allow(dead_code)]
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use slint::{Color, PhysicalPosition, PhysicalSize, WindowSize};

slint::include_modules!();

use crate::config::{Config, KeyMapping};
use crate::stats::Shared;

struct Overlay {
    comp: KeyOverlay,
    title: String,
    texts: Vec<String>,
    tick: Instant,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    fading: bool,
    fade_start: Option<Instant>,
    styled: bool,
    done: bool,
}

struct CtrlWin {
    comp: CtrlOverlay,
    title: String,
    last_text: String,
    styled: bool,
}

/// 回收待复用的浮窗（窗口/GL 画布复用，避免每次新建造成内存膨胀）
struct IdleWin {
    comp: KeyOverlay,
    title: String,
}

/// 浮窗窗口池上限（启动时预热并永久复用）。
/// 总窗口数 = 池窗口 + 控制键窗 + 应用路径窗 ≤ 10（用户要求上限）。
/// 运行期不再新建 HWND —— 窗口数恒定，内存不随按键增长，也不会出现任务栏按钮。
const PREWARM_WINDOWS: usize = 8;
/// 空闲池上限（与窗口总数一致）
const IDLE_KEEP: usize = PREWARM_WINDOWS;
/// 空闲时把窗口挪到这个屏幕外坐标（不 hide，避免 winit 重写扩展样式）
const OFFSCREEN_X: i32 = -32000;
const OFFSCREEN_Y: i32 = -32000;

pub struct UiManager {
    shared: Arc<Shared>,
    rx: Receiver<String>,
    overlays: Vec<Overlay>,
    idle: Vec<IdleWin>,
    /// 正在预热（已 show 但还需补样式）的窗口
    prewarm: Vec<IdleWin>,
    ctrl: Option<CtrlWin>,
    appwin: Option<CtrlWin>,
    keymap: KeyMapping,
    generation: u64,
    next_id: usize,
    fonts_loaded: HashSet<String>,
    /// 是否安装了 “Segoe UI Symbol”（用于 SMP 符号字形回退）
    symbol_font_ok: Option<bool>,
    last_app_path: String,
    last_app_check: Instant,
}

impl UiManager {
    fn new(shared: Arc<Shared>, rx: Receiver<String>) -> UiManager {
        let keymap = KeyMapping::load(
            &std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        );
        UiManager {
            shared,
            rx,
            overlays: Vec::new(),
            idle: Vec::new(),
            prewarm: Vec::new(),
            ctrl: None,
            appwin: None,
            keymap,
            generation: 0,
            next_id: 0,
            fonts_loaded: HashSet::new(),
            symbol_font_ok: None,
            last_app_path: String::new(),
            last_app_check: Instant::now() - Duration::from_secs(10),
        }
    }

    /// 自定义字体：配置值若是存在的字体文件路径则运行期注册（AddFontResourceW），
    /// 否则作为系统字体族名使用
    fn ensure_font(&mut self, name: &str) -> String {
        if self.fonts_loaded.contains(name) {
            return name.to_string();
        }
        let p = std::path::Path::new(name);
        if p.exists() {
            if crate::winutil::add_font_resource(p) {
                self.fonts_loaded.insert(name.to_string());
                // 字体族名取文件名（不含扩展名）
                return p
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| name.to_string());
            }
        }
        name.to_string()
    }

    fn mon_rect(cfg: &Config) -> (i32, i32, i32, i32) {
        let monitors = MONITORS.with(|m| m.borrow().clone());
        if monitors.is_empty() {
            return (0, 0, 1920, 1080);
        }
        let num = cfg.ui.gui_monitor_num;
        let idx = if num == 0 {
            crate::winutil::active_monitor(&monitors)
        } else if (num as usize) > monitors.len() || num < 0 {
            0
        } else {
            (num - 1) as usize
        };
        monitors.get(idx).copied().unwrap_or(monitors[0])
    }

    /// GetKeyText：相同按键合并为 ×N
    fn merged_text(cfg: &Config, arr: &[String]) -> String {
        let mut text = String::new();
        let mut pre_ch = String::new();
        let mut pre_diff = String::new();
        let mut pre_all = String::new();
        let mut count = 1;
        for item in arr {
            if *item == pre_ch && cfg.ui.merge_char {
                count += 1;
                text = format!("{}\u{00d7}{}", pre_diff, count);
            } else {
                count = 1;
                text = item.clone();
                if !pre_all.is_empty() {
                    text = format!("{}{}{}", pre_all, cfg.ui.txt_split, item);
                }
                pre_diff = text.clone();
            }
            pre_all = text.clone();
            pre_ch = item.clone();
        }
        text
    }

    /// 指定物理坐标所在显示器的 DPI 缩放（失败返回 1.0）
    fn scale_at_pos(x: i32, y: i32) -> f32 {
        let s = crate::winutil::scale_at(x, y);
        if s > 0.0 { s } else { 1.0 }
    }

    /// 字体回退：Slint 只支持单一字体族（SansSerif/SystemUi 均不含 🖰 🖱 🖲 等 SMP 符号），
    /// 因此文本含这类符号时改用 Segoe UI Symbol（与 AHK Edit 的字体链接效果一致）
    fn resolve_font(&mut self, family: &str, text: &str) -> String {
        if !crate::winutil::needs_symbol_font(text) {
            return family.to_string();
        }
        let ok = *self
            .symbol_font_ok
            .get_or_insert_with(|| crate::winutil::font_family_exists("Segoe UI Symbol"));
        if ok {
            "Segoe UI Symbol".to_string()
        } else {
            family.to_string()
        }
    }

    /// 文本高度估算（宽度自动换行）
    fn estimate_height(text: &str, width: i32, font_size: i32) -> i32 {
        let usable = (width - 8).max(40) as f32;
        let char_w = font_size as f32 * 0.55;
        let mut chars_per_line = (usable / char_w).max(4.0) as i32;
        let _ = &mut chars_per_line;
        // 以"宽度单位"计算总宽
        let mut units = 0.0f32;
        for ch in text.chars() {
            if (ch as u32) > 0x2E7F {
                units += 2.0; // CJK
            } else {
                units += 1.0;
            }
        }
        let lines = ((units / chars_per_line as f32).ceil() as i32).max(1);
        lines * ((font_size as f32 * 1.5) as i32) + 10
    }

    /// ShowTxt 主流程
    fn show_txt(&mut self, token: &str) {
        if token.is_empty() {
            return;
        }
        let cfg = self.shared.cfg.read().unwrap().clone();
        if cfg.ui.show_key_only_web {
            return;
        }
        // skipShow 过滤
        if cfg.ui.skip_show.iter().any(|s| s == token) {
            return;
        }
        let display = self.keymap.convert(token);
        let now = Instant::now();

        // 判断复用还是新建窗口
        let need_new = match self.overlays.last() {
            Some(last) => now.duration_since(last.tick).as_millis() as u64 >= cfg.ui.gui_interval,
            None => true,
        };

        if need_new {
            // 新窗口（guiArr.push）
            let mut texts = Vec::new();
            texts.push(display.clone());
            self.create_overlay(&cfg, texts, now);
        } else {
            // 复用最后一个窗口
            if let Some(last) = self.overlays.last_mut() {
                last.texts.push(display.clone());
                last.tick = now;
            }
        }

        // 更新最后一个窗口的文本与位置
        self.update_last_overlay(&cfg);
        self.relayout(&cfg);
    }

    fn create_overlay(&mut self, _cfg: &Config, texts: Vec<String>, tick: Instant) {
        // 窗口总数恒定：优先用空闲池，池空则回收最旧的叠层。
        // 这样运行期不再新建 HWND，既不涨内存，也不会在任务栏闪出按钮。
        let (comp, title, reused) = if let Some(w) = self.idle.pop() {
            (w.comp, w.title, true)
        } else if !self.overlays.is_empty() {
            let old = self.overlays.remove(0);
            (old.comp, old.title, true)
        } else {
            let id = self.next_id;
            self.next_id += 1;
            let comp = KeyOverlay::new().expect("create KeyOverlay");
            let title = format!("SKBOverlay{}", id);
            comp.set_overlay_id(title.as_str().into());
            (comp, title, false)
        };
        comp.set_fade(1.0);
        self.overlays.push(Overlay {
            comp,
            title,
            texts,
            tick,
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            fading: false,
            fade_start: None,
            styled: reused,
            done: false,
        });
    }

    fn update_last_overlay(&mut self, cfg: &Config) {
        let len = self.overlays.len();
        if len == 0 {
            return;
        }
        let ui = &cfg.ui;
        let last_idx = len - 1;
        let merged = {
            let texts = &self.overlays[last_idx].texts;
            Self::merged_text(cfg, texts).trim().to_string()
        };

        // 字体：配置字体无法显示 🖰/🖱 等 SMP 符号时回退到 Segoe UI Symbol
        let font = {
            let base = self.ensure_font(&ui.gui_text_font.clone());
            self.resolve_font(&base, &merged)
        };
        let (br, bg_, bb) = ui.gui_bgcolor;
        let (tr, tg, tb) = ui.gui_text_color;
        let bg_alpha: u8 = if ui.gui_bg_trans { 0 } else { ui.gui_opacity as u8 };
        let bg = Color::from_argb_u8(bg_alpha, br, bg_, bb);
        let tc = Color::from_argb_u8(ui.gui_opacity as u8, tr, tg, tb);

        let (ml, mt, mr, mb) = Self::mon_rect(cfg);
        let w = ui.gui_width;
        // DPI 缩放：配置值为物理像素，Slint 属性为逻辑像素，二者需统一换算
        let scale = {
            let mon_s = crate::winutil::scale_at(ml + (mr - ml) / 2, mt + (mb - mt) / 2);
            let win_s = self.overlays[last_idx].comp.window().scale_factor();
            if mon_s > 0.0 { mon_s } else if win_s > 0.0 { win_s } else { 1.0 }
        };
        let font_px = ((ui.gui_text_size as f32) * scale).round().max(1.0) as i32;
        // Text 左右各 4px 逻辑边距
        let usable_w = (w as f32 - 8.0 * scale).round().max(40.0) as i32;
        let h = if ui.gui_height > 0 {
            ui.gui_height
        } else {
            // GDI 精确测量文本换行后的真实高度
            let (lines, line_h, _) =
                crate::winutil::measure_text(&merged, &font, font_px, ui.gui_text_bold, usable_w);
            if lines > 0 {
                lines * line_h + 10
            } else {
                Self::estimate_height(&merged, w, ui.gui_text_size)
            }
        };

        // 位置
        let mut x = ui.gui_pos_offset_x + ml;
        let mut y = ui.gui_pos_offset_y + mt;
        match ui.gui_pos.as_str() {
            "TR" => x += mr - ml - w,
            "BL" => y += mb - mt - h,
            "BR" => {
                x += mr - ml - w;
                y += mb - mt - h;
            }
            _ => {}
        }

        {
            let ov = &mut self.overlays[last_idx];
            ov.x = x;
            ov.y = y;
            ov.w = w;
            ov.h = h;
            ov.comp.set_label(merged.as_str().into());
            ov.comp.set_bg_color(bg);
            ov.comp.set_txt_color(tc);
            ov.comp.set_font_name(font.as_str().into());
            ov.comp.set_font_size(ui.gui_text_size);
            ov.comp.set_font_weight(if ui.gui_text_bold { 700 } else { 400 });
            ov.comp.set_corner_radius(ui.gui_radius as f32 / scale);
            ov.comp.set_o_width(w as f32 / scale);
            ov.comp.set_o_height(h as f32 / scale);
            ov.comp.window().set_position(PhysicalPosition::new(x, y));
            ov.comp
                .window()
                .set_size(WindowSize::Physical(PhysicalSize::new(w as u32, h as u32)));
            ov.comp.show().ok();
            // 每次 show 后都复查一次：winit 在显示窗口时会用自身 flags 重写扩展样式，
            // 会清掉点击穿透位；复用窗口时这里是幂等操作，不会重绘闪烁
            let title = ov.title.clone();
            if crate::winutil::apply_overlay_styles(&title, ui.gui_trans) {
                ov.styled = true;
            }
        }
    }

    /// ReLayOut：把之前的窗口依次往排布方向移动
    fn relayout(&mut self, cfg: &Config) {
        let ui = &cfg.ui;
        let len = self.overlays.len();
        if len < 2 {
            return;
        }
        let last = len - 1;
        let (mut px, mut py, mut pw, mut ph) = {
            let o = &self.overlays[last];
            (o.x, o.y, o.w, o.h)
        };
        for i in (0..last).rev() {
            let (gx, gy, ow, oh) = {
                let o = &self.overlays[i];
                (o.x, o.y, o.w, o.h)
            };
            let (mut nx, mut ny) = (gx, gy);
            if ui.gui_pos_xy == "Y" {
                match ui.gui_pos.chars().next().unwrap_or('B') {
                    'T' => ny = py + ph + ui.gui_margin,   // 从上往下排
                    _ => ny = py - oh - ui.gui_margin,     // 从下往上排
                }
            } else {
                match ui.gui_pos.chars().nth(1).unwrap_or('R') {
                    'L' => nx = px + pw + ui.gui_margin,   // 从左往右
                    _ => nx = px - ow - ui.gui_margin,     // 从右往左
                }
            }
            {
                let o = &mut self.overlays[i];
                o.x = nx;
                o.y = ny;
                o.comp.window().set_position(PhysicalPosition::new(nx, ny));
            }
            px = nx;
            pw = ow;
            py = ny;
            ph = oh;
        }
    }

    /// FadeUI / CloseSelf：生命周期与渐隐
    fn tick_overlays(&mut self) {
        let cfg = self.shared.cfg.read().unwrap().clone();
        let ui = &cfg.ui;
        let now = Instant::now();
        for ov in self.overlays.iter_mut() {
            if !ov.fading {
                if now.duration_since(ov.tick).as_millis() as u64 > ui.gui_life {
                    ov.fading = true;
                    ov.fade_start = Some(now);
                }
            } else if let Some(start) = ov.fade_start {
                let progress = if ui.gui_fade_ms == 0 {
                    1.0
                } else {
                    now.duration_since(start).as_millis() as f32 / ui.gui_fade_ms as f32
                };
                if progress >= 1.0 {
                    // 不 hide（hide/show 会让 winit 重写扩展样式并在任务栏闪出按钮），
                    // 只把窗口移出屏幕并置为全透明
                    ov.comp.set_fade(0.0);
                    ov.comp.set_label("".into());
                    ov.comp.window().set_position(PhysicalPosition::new(OFFSCREEN_X, OFFSCREEN_Y));
                    ov.fade_start = None;
                    ov.done = true;
                } else {
                    ov.comp.set_fade(1.0 - progress);
                }
            }
        }
        // 移除完成的窗口（isFade == 2），并把它们放进空闲池复用
        if self.overlays.iter().any(|ov| ov.done) {
            let mut finished = Vec::new();
            let mut keep = Vec::with_capacity(self.overlays.len());
            for ov in self.overlays.drain(..) {
                if ov.done {
                    finished.push(ov);
                } else {
                    keep.push(ov);
                }
            }
            self.overlays = keep;
            for ov in finished {
                if self.idle.len() < IDLE_KEEP {
                    self.idle.push(IdleWin { comp: ov.comp, title: ov.title });
                }
            }
        }
        let target = Self::pool_target(ui);
        self.prewarm_step(ui.gui_trans, target);
    }

    /// 窗口池大小：按 guiLife/guiInterval 的并发量估算（2~8），
    /// 既不会因为窗口太少而提前挤掉旧浮窗，也不会白白占内存
    fn pool_target(ui: &crate::config::UiParams) -> usize {
        let interval = ui.gui_interval.max(100);
        let life = ui.gui_life + ui.gui_fade_ms;
        let n = (life / interval) as usize + 1;
        n.clamp(2, PREWARM_WINDOWS)
    }

    /// 预热空闲窗口：先以离屏、全透明的方式 show 出来，等系统真正创建窗口后
    /// 补上 +ToolWindow/+Owner 样式，然后一直保持"可见但透明且移出屏幕"的状态。
    /// 关键点：之后只在屏幕内/外移动位置，不再 hide/show——winit 每次切换可见性都会
    /// 用自身 flags 重写 WS_EX_STYLE（把 WS_EX_APPWINDOW 加回来），那会让任务栏闪出按钮。
    /// 所有窗口在同一次 show（同一轮事件循环）里创建：任务栏只会闪一次，
    /// 而不是逐个窗口各闪一次。
    fn prewarm_step(&mut self, click_through: bool, target: usize) {
        // 1) 给已 show 的预热窗口补样式；成功后进入空闲池
        if !self.prewarm.is_empty() {
            let mut pending = Vec::new();
            for w in self.prewarm.drain(..) {
                if crate::winutil::apply_overlay_styles(&w.title, click_through) {
                    w.comp.set_fade(0.0);
                    w.comp.window().set_position(PhysicalPosition::new(OFFSCREEN_X, OFFSCREEN_Y));
                    self.idle.push(w);
                } else {
                    // 窗口句柄还没建好（事件循环延迟创建），下一轮再试
                    pending.push(w);
                }
            }
            self.prewarm = pending;
            return;
        }
        // 2) 一次性把剩余窗口全部 show 出来
        let total = self.idle.len() + self.overlays.len();
        if total < target {
            for _ in total..target {
                let id = self.next_id;
                self.next_id += 1;
                let comp = KeyOverlay::new().expect("create KeyOverlay");
                let title = format!("SKBOverlay{}", id);
                comp.set_overlay_id(title.as_str().into());
                comp.set_fade(0.0);
                comp.set_o_width(1.0);
                comp.set_o_height(1.0);
                comp.window().set_position(PhysicalPosition::new(OFFSCREEN_X, OFFSCREEN_Y));
                comp.window().set_size(WindowSize::Physical(PhysicalSize::new(1, 1)));
                comp.show().ok();
                self.prewarm.push(IdleWin { comp, title });
            }
        }
    }

    /// ShowCtrlState：控制键状态窗
    fn update_ctrl(&mut self) {
        let cfg = self.shared.cfg.read().unwrap().clone();
        let ui = &cfg.ui;
        if !ui.ctrl_state {
            if let Some(c) = &mut self.ctrl {
                c.comp.set_label("".into());
                c.comp.window().set_position(PhysicalPosition::new(OFFSCREEN_X, OFFSCREEN_Y));
            }
            return;
        }
        // 计算按下的控制键
        let mut txt = String::new();
        for key in &ui.ctrl_list {
            let pressed = match key.as_str() {
                "CapsLock" | "NumLock" | "ScrollLock" => {
                    let vk = match key.as_str() {
                        "CapsLock" => 0x14,
                        "NumLock" => 0x90,
                        _ => 0x91,
                    };
                    crate::winutil::key_toggled(vk)
                }
                _ => {
                    let vks: &[i32] = match key.as_str() {
                        "Ctrl" | "Control" => &[0x11],
                        "LCtrl" => &[0xA2, 0x11],
                        "RCtrl" => &[0xA3],
                        "Alt" => &[0x12],
                        "LAlt" => &[0xA4, 0x12],
                        "RAlt" => &[0xA5],
                        "Shift" => &[0x10],
                        "LShift" => &[0xA0, 0x10],
                        "RShift" => &[0xA1],
                        "LWin" => &[0x5B],
                        "RWin" => &[0x5C],
                        _ => &[],
                    };
                    vks.iter().any(|vk| crate::winutil::key_down(*vk))
                }
            };
            if pressed {
                txt.push_str(&self.keymap.convert(key));
                txt.push_str(&ui.txt_split);
            }
        }
        let txt = txt.trim().to_string();
        if txt == self.ctrl.as_ref().map(|c| c.last_text.clone()).unwrap_or_default() && self.ctrl.is_some() {
            return; // 避免刷新闪烁
        }
        if self.ctrl.is_none() {
            let comp = CtrlOverlay::new().expect("create CtrlOverlay");
            let title = "SKBCtrlState".to_string();
            comp.set_overlay_id(title.as_str().into());
            self.ctrl = Some(CtrlWin { comp, title, last_text: String::new(), styled: false });
        }
        let font = {
            let base = self.ensure_font(&ui.ctrl_text_font.clone());
            self.resolve_font(&base, &txt)
        };
        let click_through = ui.gui_trans;
        let win = self.ctrl.as_mut().unwrap();
        win.last_text = txt.clone();
        let (br, bg_, bb) = ui.ctrl_bgcolor;
        let (tr, tg, tb) = ui.ctrl_text_color;
        win.comp.set_label(txt.as_str().into());
        win.comp.set_bg_color(Color::from_argb_u8(ui.ctrl_opacity as u8, br, bg_, bb));
        win.comp.set_txt_color(Color::from_argb_u8(ui.ctrl_opacity as u8, tr, tg, tb));
        win.comp.set_font_name(font.as_str().into());
        win.comp.set_font_size(ui.ctrl_text_size);
        win.comp.set_font_weight(if ui.ctrl_text_bold { 700 } else { 400 });
        win.comp.set_corner_radius(ui.ctrl_radius as f32);
        let w = ui.ctrl_width;
        let scale = Self::scale_at_pos(ui.ctrl_x, ui.ctrl_y);
        let h = if ui.ctrl_height != 0 {
            ui.ctrl_height
        } else {
            (ui.ctrl_text_size as f32 * scale * 1.8).round() as i32
        };
        win.comp.set_o_width(w as f32 / scale);
        win.comp.set_o_height(h as f32 / scale);
        // 不 hide：显示一次后只改变位置/尺寸，避免 winit 在显示时重写扩展样式
        // （那会把 WS_EX_APPWINDOW 加回来，导致任务栏闪一下）
        let (x, y) = if txt.is_empty() {
            (OFFSCREEN_X, OFFSCREEN_Y)
        } else {
            (ui.ctrl_x, ui.ctrl_y)
        };
        win.comp.window().set_position(PhysicalPosition::new(x, y));
        win.comp
            .window()
            .set_size(WindowSize::Physical(PhysicalSize::new(w as u32, h as u32)));
        win.comp.show().ok();
        let title = win.title.clone();
        if crate::winutil::apply_overlay_styles(&title, click_through) {
            win.styled = true;
        }
    }

    /// activeAppShow：显示当前激活应用路径
    fn update_app_path(&mut self) {
        let cfg = self.shared.cfg.read().unwrap().clone();
        let ui = &cfg.ui;
        if !ui.active_app_show {
            if let Some(c) = &mut self.appwin {
                // 同样不 hide，只移出屏幕
                c.comp.set_label("".into());
                c.comp.window().set_position(PhysicalPosition::new(OFFSCREEN_X, OFFSCREEN_Y));
            }
            return;
        }
        if self.last_app_check.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.last_app_check = Instant::now();
        let path = crate::winutil::foreground_proc_path();
        if path == self.last_app_path && self.appwin.is_some() {
            return;
        }
        self.last_app_path = path.clone();
        if self.appwin.is_none() {
            let comp = CtrlOverlay::new().expect("create AppPathOverlay");
            let title = "SKBAppPath".to_string();
            comp.set_overlay_id(title.as_str().into());
            self.appwin = Some(CtrlWin { comp, title, last_text: String::new(), styled: false });
        }
        let font = self.ensure_font(&ui.ctrl_text_font.clone());
        let win = self.appwin.as_mut().unwrap();
        win.last_text = path.clone();
        let (br, bg_, bb) = ui.ctrl_bgcolor;
        win.comp.set_label(path.as_str().into());
        win.comp.set_bg_color(Color::from_argb_u8(ui.ctrl_opacity as u8, br, bg_, bb));
        win.comp.set_txt_color(Color::from_argb_u8(ui.ctrl_opacity as u8, ui.gui_text_color.0, ui.gui_text_color.1, ui.gui_text_color.2));
        win.comp.set_font_name(font.as_str().into());
        win.comp.set_font_size(ui.ctrl_text_size.max(12));
        win.comp.set_font_weight(400);
        win.comp.set_corner_radius(ui.ctrl_radius as f32);
        let w = ui.ctrl_width.max(200);
        let scale = Self::scale_at_pos(ui.active_app_show_x, ui.active_app_show_y);
        let h = (ui.ctrl_text_size.max(12) as f32 * scale * 1.8).round() as i32;
        win.comp.set_o_width(w as f32 / scale);
        win.comp.set_o_height(h as f32 / scale);
        win.comp.window().set_position(PhysicalPosition::new(ui.active_app_show_x, ui.active_app_show_y));
        win.comp
            .window()
            .set_size(WindowSize::Physical(PhysicalSize::new(w as u32, h as u32)));
        win.comp.show().ok();
        let title = win.title.clone();
        if crate::winutil::apply_overlay_styles(&title, true) {
            win.styled = true;
        }
    }

    /// 是否还有窗口尚未成功应用样式（窗口句柄由事件循环延迟创建）
    fn has_unstyled(&self) -> bool {
        self.overlays.iter().any(|o| !o.styled)
            || self.ctrl.as_ref().map(|c| !c.styled).unwrap_or(false)
            || self.appwin.as_ref().map(|c| !c.styled).unwrap_or(false)
    }

    /// 复查窗口样式（点击穿透 / 工具窗口 / owner）。
    /// apply_overlay_styles 内部先比较当前样式，已就位时直接返回，不会造成闪烁，
    /// 因此可以周期性调用，防止 winit 在 show/属性变更时覆写 ex-style。
    fn apply_styles(&mut self) {
        let click_through = self.shared.cfg.read().unwrap().ui.gui_trans;
        for ov in self.overlays.iter_mut() {
            if crate::winutil::apply_overlay_styles(&ov.title, click_through) {
                ov.styled = true;
            }
        }
        // 控制键窗常驻可见（只是移到屏幕外），文本为空时同样必须套样式，
        // 否则它会在任务栏留下一个按钮
        if let Some(c) = &mut self.ctrl {
            if crate::winutil::apply_overlay_styles(&c.title, click_through) {
                c.styled = true;
            }
        }
        if let Some(c) = &mut self.appwin {
            if crate::winutil::apply_overlay_styles(&c.title, true) {
                c.styled = true;
            }
        }
    }
}

thread_local! {
    static MONITORS: std::cell::RefCell<Vec<crate::winutil::Rect>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// 在 Slint 事件循环中运行 UI（阻塞，直到退出）
pub fn run(shared: Arc<Shared>, rx: Receiver<String>) {
    // 预加载显示器信息
    let mons = crate::winutil::monitors();
    MONITORS.with(|m| *m.borrow_mut() = mons);

    let mut manager = UiManager::new(shared.clone(), rx);

    let timer = slint::Timer::default();
    let mut tick_count = 0u64;
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(20),
        move || {
            tick_count += 1;
            let shared_ref = shared.clone();
            if shared_ref.exit.load(Ordering::Relaxed) {
                slint::quit_event_loop().ok();
                return;
            }
            // 配置热重载：generation 变化时刷新窗口参数
            let gen = shared_ref.cfg.read().unwrap().generation;
            if gen != manager.generation {
                manager.generation = gen;
                // 清理旧窗口，让新配置生效
                for ov in manager.overlays.iter_mut() {
                    ov.comp.hide().ok();
                }
                manager.overlays.clear();
                manager.idle.clear();
                manager.prewarm.clear();
                if let Some(c) = &mut manager.ctrl {
                    c.comp.hide().ok();
                }
                manager.ctrl = None;
                if let Some(c) = &mut manager.appwin {
                    c.comp.hide().ok();
                }
                manager.appwin = None;
            }

            // 消费显示队列（CallShow）
            while let Ok(token) = manager.rx.try_recv() {
                manager.show_txt(&token);
            }

            manager.tick_overlays();
            if tick_count % 3 == 0 {
                // ~60ms 刷新控制键状态（AHK 为 50ms）
                manager.update_ctrl();
                manager.update_app_path();
            }
            // 新建窗口需要尽快补样式（句柄由事件循环延迟创建），之后就低频兜底复查
            if manager.has_unstyled() || tick_count % 25 == 0 {
                manager.apply_styles();
            }
        },
    );

    // 注意：不能用 run_event_loop()——winit 后端在最后一个窗口隐藏后会退出事件循环，
    // 导致所有弹框渐隐后新弹框永远无法出现；until_quit 保持循环运行直到显式退出
    slint::run_event_loop_until_quit().expect("slint event loop");
}
