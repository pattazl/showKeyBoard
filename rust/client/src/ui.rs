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
    done: bool,
}

struct CtrlWin {
    comp: CtrlOverlay,
    title: String,
    last_text: String,
    /// 位置/尺寸是否已按物理像素成功应用（窗口刚创建时 winit 窗口尚未就绪，需要下一轮重试）
    positioned: bool,
}

/// 连接状态提示窗：复用按键浮窗组件与空闲窗口池，显示后台服务启动/连接过程与异常。
/// 由 net 线程通过 UI 文本管道推送 `\u{1}`（常驻）/ `\u{2}`（短暂后渐隐）前缀的消息驱动。
struct StatusWin {
    comp: KeyOverlay,
    title: String,
    /// 自动渐隐时间点（None 表示常驻，直到下一条状态把它替换/自己结束）
    hide_at: Option<Instant>,
    /// 渐隐开始时间
    fade_start: Option<Instant>,
}

/// 回收待复用的浮窗（窗口/GL 画布复用，避免每次新建造成内存膨胀）
struct IdleWin {
    comp: KeyOverlay,
    title: String,
}

/// 浮窗窗口池上限（启动时预热并永久复用）。
/// 池窗口（含状态浮窗）最多 16 个；另加控制键窗 + 应用路径窗，总窗口数 ≤ 16+2。
/// 运行期不再新建 HWND —— 窗口数恒定，内存不随按键增长，也不会出现任务栏按钮。
const PREWARM_WINDOWS: usize = 16;
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
    status: Option<StatusWin>,
    keymap: KeyMapping,
    generation: u64,
    next_id: usize,
    /// 已成功注册的自定义字体文件名（字体族名 = stem）
    fonts_loaded: HashSet<String>,
    /// 系统已存在 / 已确认不存在的字体族名缓存
    fonts_confirmed: HashSet<String>,
    /// 是否安装了 "Segoe UI Symbol"（用于 SMP 符号字形回退，仅 Windows）
    #[cfg_attr(not(windows), allow(dead_code))]
    symbol_font_ok: Option<bool>,
    last_app_path: String,
    last_app_check: Instant,
    /// WebSocket 推送状态：当前 guiInterval 窗口内累计的显示文本与起始时间
    /// （对应 AHK ShowTxt 的 lastTextArr / lastTextTick）
    ws_last_texts: Vec<String>,
    ws_last_tick: Option<Instant>,
    /// 上次刷新显示器列表的时间（每 5 秒检查一次布局变化）
    last_monitor_check: Instant,
}

impl UiManager {
    fn new(shared: Arc<Shared>, rx: Receiver<String>) -> UiManager {
        let keymap = KeyMapping::load(&shared.cfg.read().unwrap().data_dir);
        UiManager {
            shared,
            rx,
            overlays: Vec::new(),
            idle: Vec::new(),
            prewarm: Vec::new(),
            ctrl: None,
            appwin: None,
            status: None,
            keymap,
            generation: 0,
            next_id: 0,
            fonts_loaded: HashSet::new(),
            fonts_confirmed: HashSet::new(),
            symbol_font_ok: None,
            last_app_path: String::new(),
            last_app_check: Instant::now() - Duration::from_secs(10),
            ws_last_texts: Vec::new(),
            ws_last_tick: None,
            last_monitor_check: Instant::now(),
        }
    }

    /// 自定义字体：配置值若是存在的字体文件路径则运行期注册（AddFontResourceW），
    /// 否则作为系统字体族名使用；字体不存在时回退到 "Tahoma"（Windows 全版本预装）
    fn ensure_font(&mut self, name: &str) -> String {
        if self.fonts_loaded.contains(name) {
            return name.to_string();
        }
        let p = std::path::Path::new(name);
        if p.exists() {
            if crate::winutil::add_font_resource(p) {
                let family = p
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| name.to_string());
                self.fonts_loaded.insert(family.clone());
                return family;
            }
        }
        let low = name.to_lowercase();
        if self.fonts_confirmed.contains(&low) {
            return name.to_string();
        }
        if crate::winutil::font_family_exists(name) {
            self.fonts_confirmed.insert(low);
            return name.to_string();
        }
        "Tahoma".to_string()
    }

    fn mon_rect(cfg: &Config) -> (i32, i32, i32, i32) {
        let monitors = MONITORS.with(|m| m.borrow().clone());
        if monitors.is_empty() {
            return (0, 0, 1920, 1080);
        }
        // 布局变化（热插拔/分辨率/扩展模式）时强制落到默认显示器（monitors[0]），
        // 避免按 guiMonitorNum 取到已消失的屏幕坐标导致浮窗飞出可视区；
        // 恢复到启动时的布局后，继续按原 guiMonitorNum 配置显示。
        let layout_changed =
            MONITORS_SNAPSHOT.with(|s| *s.borrow() != monitors);
        let num = if layout_changed { 1 } else { cfg.ui.gui_monitor_num };
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
    /// 因此文本含这类符号时切换到平台符号字体
    /// （Windows: Segoe UI Symbol；macOS: Apple Symbols 等，与 AHK Edit 的字体链接效果一致）
    fn resolve_font(&mut self, family: &str, text: &str) -> String {
        if !crate::winutil::needs_symbol_font(text) {
            return family.to_string();
        }
        #[cfg(windows)]
        {
            let ok = *self
                .symbol_font_ok
                .get_or_insert_with(|| crate::winutil::font_family_exists("Segoe UI Symbol"));
            if ok {
                "Segoe UI Symbol".to_string()
            } else {
                family.to_string()
            }
        }
        #[cfg(not(windows))]
        {
            // macOS：缓存由 winutil::symbol_font_family 内部 OnceLock 处理
            crate::winutil::symbol_font_family()
                .unwrap_or(family)
                .to_string()
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

    /// WebSocket 实时按键推送（对应 AHK ShowTxt 中的 handleWS.sendText(wsText)）：
    /// 同一 guiInterval 窗口内的按键累计为一条文本；复用旧窗口时加 "0::" 前缀，
    /// 前端据此更新已有浮窗而不是新建。发送走有界管道，绝不阻塞 UI 线程。
    fn send_ws(&mut self, cfg: &Config, display: &str, now: Instant) {
        let need_new = match self.ws_last_tick {
            Some(t) => now.duration_since(t).as_millis() as u64 >= cfg.ui.gui_interval,
            None => true,
        };
        if need_new {
            self.ws_last_texts.clear();
            self.ws_last_tick = Some(now);
        }
        self.ws_last_texts.push(display.to_string());
        let new_text = Self::merged_text(cfg, &self.ws_last_texts);
        let new_text = new_text.trim();
        if new_text.is_empty() {
            return;
        }
        // 未连接时直接丢弃（与 AHK handleWS=0 时不发送一致）
        if !self.shared.ws_ready.load(Ordering::Relaxed) {
            return;
        }
        let ws_text = if need_new {
            new_text.to_string()
        } else {
            format!("0::{}", new_text)
        };
        let _ = self.shared.ws_tx.try_send(ws_text);
    }

    /// ShowTxt 主流程
    fn show_txt(&mut self, token: &str) {
        if token.is_empty() {
            return;
        }
        // net 线程推送的后台服务状态：\u{1} 常驻（连接中/异常），\u{2} 短暂显示后渐隐
        if let Some(rest) = token.strip_prefix('\u{1}') {
            self.show_status(rest, true);
            return;
        }
        if let Some(rest) = token.strip_prefix('\u{2}') {
            self.show_status(rest, false);
            return;
        }
        // show_txt 调用频率低于 push_txt（仅需显示的按键才走到这里），
        // 且后续 create_overlay/update_last_overlay/relayout 均为 &mut self，
        // 无法在持 cfg 读锁的同时调用，故此处 clone Config。
        // 真正的热路径 push_txt（每次按键）已去除 clone。
        let cfg = self.shared.cfg.read().unwrap().clone();
        // skipShow 过滤（AHK 在 CallShow 中过滤，WebSocket 也不推送）
        if cfg.ui.skip_show.iter().any(|s| s == token) {
            return;
        }
        let display = self.keymap.convert(token);
        let now = Instant::now();

        // 先推送到后端（showKeyOnlyWeb=1 时也要发送，AHK 中该判断在 sendText 之后）
        self.send_ws(&cfg, &display, now);

        if cfg.ui.show_key_only_web {
            return;
        }

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
        let (comp, title) = if let Some(w) = self.idle.pop() {
            (w.comp, w.title)
        } else if !self.overlays.is_empty() {
            let old = self.overlays.remove(0);
            (old.comp, old.title)
        } else {
            let id = self.next_id;
            self.next_id += 1;
            let comp = KeyOverlay::new().expect("create KeyOverlay");
            let title = format!("SKBOverlay{}", id);
            comp.set_overlay_id(title.as_str().into());
            (comp, title)
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
            done: false,
        });
    }

    fn update_last_overlay(&mut self, cfg: &Config) {
        let len = self.overlays.len();
        if len == 0 {
            return;
        }
        let last_idx = len - 1;
        let merged = Self::merged_text(cfg, &self.overlays[last_idx].texts);
        let font = self.overlay_font(cfg, &merged);
        let ov = &mut self.overlays[last_idx];
        let (x, y, pw, ph) = Self::apply_overlay_layout(cfg, &ov.comp, &merged, &font);
        ov.x = x;
        ov.y = y;
        ov.w = pw;
        ov.h = ph;
        let title = ov.title.clone();
        crate::winutil::apply_overlay_styles(&ov.comp.window(), &title, cfg.ui.gui_trans);
    }

    /// 浮窗字体：配置字体无法显示 🖰/🖱、⎇/⇧ 等符号时回退到 Segoe UI Symbol
    fn overlay_font(&mut self, cfg: &Config, text: &str) -> String {
        let base = self.ensure_font(&cfg.ui.gui_text_font.clone());
        self.resolve_font(&base, text)
    }

    /// 给浮窗套用文本/颜色/尺寸并定位（按键浮窗与连接状态浮窗共用）
    fn apply_overlay_layout(
        cfg: &Config,
        comp: &KeyOverlay,
        text: &str,
        font: &str,
    ) -> (i32, i32, i32, i32) {
        let ui = &cfg.ui;
        let merged = crate::winutil::strip_variation_selectors(text.trim());

        let (br, bg_, bb) = ui.gui_bgcolor;
        let (tr, tg, tb) = ui.gui_text_color;
        let bg_alpha: u8 = if ui.gui_bg_trans { 0 } else { ui.gui_opacity as u8 };
        let bg = Color::from_argb_u8(bg_alpha, br, bg_, bb);
        let tc = Color::from_argb_u8(ui.gui_opacity as u8, tr, tg, tb);

        let (ml, mt, mr, mb) = Self::mon_rect(cfg);
        // DPI 缩放一律以“目标显示器”为准：窗口可能还停在另一块不同缩放的屏幕上，
        // 用窗口当前的 scale_factor 会把尺寸算错（高 DPI 下层层叠加后互相重叠）
        let scale = Self::scale_at_pos(ml + (mr - ml) / 2, mt + (mb - mt) / 2);
        // guiDpiscale=true：配置宽高按物理像素固定；false：按逻辑像素（随系统 DPI 放大）
        let size_k = if ui.gui_dpiscale { 1.0 } else { scale };
        // Slint 的 font-size 是逻辑像素，GDI 测量需要物理像素
        let font_px = ((ui.gui_text_size as f32) * scale).round().max(1.0) as i32;
        let pw = ((ui.gui_width as f32) * size_k).round().max(1.0) as i32;
        // Text 左右各 4px 逻辑边距
        let usable_w = (pw as f32 - 8.0 * scale).round().max(40.0) as i32;
        let ph = if ui.gui_height > 0 {
            ((ui.gui_height as f32) * size_k).round() as i32
        } else {
            // GDI 精确测量文本换行后的真实高度（物理像素）
            let (lines, line_h, _) =
                crate::winutil::measure_text(&merged, font, font_px, ui.gui_text_bold, usable_w);
            if lines > 0 {
                lines * line_h + (10.0 * scale).round() as i32
            } else {
                Self::estimate_height(&merged, pw, font_px)
            }
        };

        // 位置：偏移量与宽高使用同一单位约定（guiDpiscale=1 为物理像素，0 为逻辑像素）
        let ox = ((ui.gui_pos_offset_x as f32) * size_k).round() as i32;
        let oy = ((ui.gui_pos_offset_y as f32) * size_k).round() as i32;
        let mut x = ml + ox;
        let mut y = mt + oy;
        match ui.gui_pos.as_str() {
            "TR" => x = mr + ox - pw,
            "BL" => y = mb + oy - ph,
            "BR" => {
                x = mr + ox - pw;
                y = mb + oy - ph;
            }
            _ => {}
        }

        let edge_color = Color::from_argb_u8(ui.gui_opacity as u8, br, bg_, bb);
        comp.set_label(merged.as_str().into());
        comp.set_bg_color(bg);
        comp.set_txt_color(tc);
        comp.set_edge_color(edge_color);
        comp.set_show_edge(ui.gui_edge);
        comp.set_font_name(font.into());
        comp.set_font_size(ui.gui_text_size);
        comp.set_font_weight(if ui.gui_text_bold { 700 } else { 400 });
        comp.set_corner_radius(if ui.gui_dpiscale {
            ui.gui_radius as f32 / scale
        } else {
            ui.gui_radius as f32
        });
        // Slint 尺寸属性是逻辑像素：物理尺寸 / 显示器缩放
        comp.set_o_width(pw as f32 / scale);
        comp.set_o_height(ph as f32 / scale);
        // Windows：保持原实现（Slint Physical 接口）
        // macOS：Slint 的 set_size(Physical) 会先按窗口 scale_factor（屏外窗口退化为
        // 1.0）转 Logical，再被 winit 按真实 scale 乘回去，导致尺寸翻倍；
        // 因此 show 后再走 winit 物理像素接口一次性校正位置与尺寸（无歧义换算）
        comp.window().set_position(PhysicalPosition::new(x, y));
        comp.window()
            .set_size(WindowSize::Physical(PhysicalSize::new(pw as u32, ph as u32)));
        comp.show().ok();
        #[cfg(not(windows))]
        {
            use slint::winit_030::WinitWindowAccessor;
            comp.window().with_winit_window(|ww| {
                use slint::winit_030::winit::dpi::{
                    PhysicalPosition as WinitPos, PhysicalSize as WinitSize,
                };
                ww.set_outer_position(WinitPos::new(x, y));
                let _ = ww.request_inner_size(WinitSize::new(pw as u32, ph as u32));
            });
        }
        (x, y, pw, ph)
    }

    /// 显示后台服务连接状态：复用空闲浮窗（不新增 HWND），sticky=true 时常驻到状态变化
    fn show_status(&mut self, text: &str, sticky: bool) {
        // 非热路径（仅连接状态变化时调用），clone Config 以满足
        // overlay_font/apply_overlay_layout 的 &Config 签名。
        let cfg = self.shared.cfg.read().unwrap().clone();
        if self.status.is_none() {
            let (comp, title) = if let Some(w) = self.idle.pop() {
                (w.comp, w.title)
            } else {
                let id = self.next_id;
                self.next_id += 1;
                let comp = KeyOverlay::new().expect("create StatusOverlay");
                let title = format!("SKBOverlay{}", id);
                comp.set_overlay_id(title.as_str().into());
                (comp, title)
            };
            self.status = Some(StatusWin {
                comp,
                title,
                hide_at: None,
                fade_start: None,
            });
        }
        let font = self.overlay_font(&cfg, text);
        let win = self.status.as_mut().unwrap();
        win.hide_at = if sticky {
            None
        } else {
            Some(Instant::now() + Duration::from_millis(2000))
        };
        win.fade_start = None;
        win.comp.set_fade(1.0);
        let _ = Self::apply_overlay_layout(&cfg, &win.comp, text, &font);
        crate::winutil::apply_overlay_styles(&win.comp.window(), &win.title, cfg.ui.gui_trans);
    }

    /// 状态浮窗生命周期：到点渐隐，结束后归还空闲窗口池
    fn tick_status(&mut self, fade_ms: u64) {
        let now = Instant::now();
        let mut finished = false;
        if let Some(s) = self.status.as_mut() {
            if let Some(t) = s.hide_at {
                if now >= t {
                    s.hide_at = None;
                    s.fade_start = Some(now);
                }
            }
            if let Some(start) = s.fade_start {
                let progress = if fade_ms == 0 {
                    1.0
                } else {
                    now.duration_since(start).as_millis() as f32 / fade_ms as f32
                };
                if progress >= 1.0 {
                    s.comp.set_fade(0.0);
                    s.comp.set_label("".into());
                    s.comp
                        .window()
                        .set_position(PhysicalPosition::new(OFFSCREEN_X, OFFSCREEN_Y));
                    finished = true;
                } else {
                    s.comp.set_fade(1.0 - progress);
                }
            }
        }
        if finished {
            if let Some(s) = self.status.take() {
                if self.idle.len() < IDLE_KEEP {
                    self.idle.push(IdleWin {
                        comp: s.comp,
                        title: s.title,
                    });
                }
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
        // 间距与宽高同一单位：guiDpiscale=1 为物理像素，0 为逻辑像素
        let margin = ((ui.gui_margin as f32)
            * if ui.gui_dpiscale { 1.0 } else { Self::scale_at_pos(px, py) })
        .round() as i32;
        for i in (0..last).rev() {
            let (gx, gy, ow, oh) = {
                let o = &self.overlays[i];
                (o.x, o.y, o.w, o.h)
            };
            let (mut nx, mut ny) = (gx, gy);
            if ui.gui_pos_xy == "Y" {
                match ui.gui_pos.chars().next().unwrap_or('B') {
                    'T' => ny = py + ph + margin,   // 从上往下排
                    _ => ny = py - oh - margin,     // 从下往上排
                }
            } else {
                match ui.gui_pos.chars().nth(1).unwrap_or('R') {
                    'L' => nx = px + pw + margin,   // 从左往右
                    _ => nx = px - ow - margin,     // 从右往左
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
        // 每 20ms 调用一次：只提取需要的原始值，不持有 cfg 读锁，
        // 也不 clone 整个 Config（避免持续堆分配碎片）。
        let (gui_life, gui_fade_ms, gui_trans, target) = {
            let cfg = self.shared.cfg.read().unwrap();
            let ui = &cfg.ui;
            (ui.gui_life, ui.gui_fade_ms, ui.gui_trans, Self::pool_target(ui))
        };
        let now = Instant::now();
        for ov in self.overlays.iter_mut() {
            if !ov.fading {
                if now.duration_since(ov.tick).as_millis() as u64 > gui_life {
                    ov.fading = true;
                    ov.fade_start = Some(now);
                }
            } else if let Some(start) = ov.fade_start {
                let progress = if gui_fade_ms == 0 {
                    1.0
                } else {
                    now.duration_since(start).as_millis() as f32 / gui_fade_ms as f32
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
        // 每 5 秒刷新显示器列表；布局变化（热插拔/分辨率/扩展模式）时重建所有浮窗，
        // 避免窗口停留在已消失的屏幕坐标上不可见。ctrl/appwin 由 update_* 自动重定位。
        if now.duration_since(self.last_monitor_check) >= Duration::from_secs(5) {
            self.last_monitor_check = now;
            let new_mons = crate::winutil::monitors();
            let changed = MONITORS.with(|m| *m.borrow() != new_mons);
            if changed {
                MONITORS.with(|m| *m.borrow_mut() = new_mons);
                for ov in self.overlays.iter_mut() {
                    ov.comp.hide().ok();
                }
                self.overlays.clear();
                self.idle.clear();
                self.prewarm.clear();
            }
        }
        self.prewarm_step(gui_trans, target);
    }

    /// 窗口池大小：按 guiLife/guiInterval 的并发量估算（2~16），
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
                if crate::winutil::apply_overlay_styles(&w.comp.window(), &w.title, click_through) {
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
        // 2) 一次性把剩余窗口全部 show 出来（状态浮窗占用的窗口也计入总数）
        let total = self.idle.len() + self.overlays.len() + usize::from(self.status.is_some());
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
        // 每 ~60ms 调用一次：只 clone UiParams（远小于整个 Config），
        // 避免持 cfg 读锁时调用 &mut self 方法的借用冲突。
        let ui = self.shared.cfg.read().unwrap().ui.clone();
        if !ui.ctrl_state {
            if let Some(c) = &self.ctrl {
                c.comp.hide().ok();
            }
            return;
        }
        // 切换键（CapsLock/NumLock/ScrollLock）状态直接查询系统 toggle 位，
        // 与 AHK 的 GetKeyState(keyName,"T") 每 50ms 轮询语义一致，实时且无竞态。
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
                    // macOS 侧钩子把左右控制键分别映射为 0xA2/0xA3 等（不产生 0x11），
                    // 因此通用键名（Ctrl/Alt/Shift）需同时包含左右 VK，Windows 下同样有效
                    let vks: &[i32] = match key.as_str() {
                        "Ctrl" | "Control" => &[0x11, 0xA2, 0xA3],
                        "LCtrl" => &[0xA2, 0x11],
                        "RCtrl" => &[0xA3],
                        "Alt" => &[0x12, 0xA4, 0xA5],
                        "LAlt" => &[0xA4, 0x12],
                        "RAlt" => &[0xA5],
                        "Shift" => &[0x10, 0xA0, 0xA1],
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
        let txt = crate::winutil::strip_variation_selectors(txt.trim());
        // 避免刷新闪烁（位置尚未成功应用时也要继续尝试）
        if self
            .ctrl
            .as_ref()
            .map(|c| c.last_text == txt && c.positioned)
            .unwrap_or(false)
        {
            return;
        }
        // 没有按下的控制键：直接隐藏（AHK 也是 hide）。
        // 这里不能像浮窗那样移到屏幕外——窗口在屏外时 scale_factor 会退化为 1.0，
        // 导致后续按显示器 DPI 计算的尺寸/位置全部被缩小。
        if txt.is_empty() {
            if let Some(c) = &mut self.ctrl {
                c.last_text.clear();
                c.comp.hide().ok();
            }
            return;
        }
        if self.ctrl.is_none() {
            let comp = CtrlOverlay::new().expect("create CtrlOverlay");
            let title = "SKBCtrlState".to_string();
            comp.set_overlay_id(title.as_str().into());
            self.ctrl = Some(CtrlWin {
                comp,
                title,
                last_text: String::new(),
                positioned: false,
            });
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
        let bg_col = Color::from_argb_u8(ui.ctrl_opacity as u8, br, bg_, bb);
        win.comp.set_label(txt.as_str().into());
        win.comp.set_bg_color(bg_col);
        win.comp.set_txt_color(Color::from_argb_u8(ui.ctrl_opacity as u8, tr, tg, tb));
        win.comp.set_edge_color(bg_col);
        win.comp.set_show_edge(ui.gui_edge);
        win.comp.set_font_name(font.as_str().into());
        win.comp.set_font_size(ui.ctrl_text_size);
        win.comp.set_font_weight(if ui.ctrl_text_bold { 700 } else { 400 });
        // 控制键状态窗：配置里 ctrl_X/ctrl_Y 是屏幕物理像素（与 activeAppShow 一致）。
        // AHK 的 Gui +DPIScale 只影响 Gui 内部控件坐标，外部屏幕坐标永远是物理像素。
        // 尺寸上，Gui 内部控件坐标会自动按 DPI 缩放 → 配置尺寸是**逻辑像素**，
        // 因此物理尺寸 = 配置 × scale；Slint 逻辑属性 = 物理尺寸 / scale。
        let scale = Self::scale_at_pos(ui.ctrl_x, ui.ctrl_y);
        let w_phys = (ui.ctrl_width as f32 * scale).round().max(1.0) as i32;
        let h_phys = if ui.ctrl_height != 0 {
            (ui.ctrl_height as f32 * scale).round().max(1.0) as i32
        } else {
            (ui.ctrl_text_size as f32 * scale * 1.8).round().max(1.0) as i32
        };
        win.comp.set_corner_radius(ui.ctrl_radius as f32 / scale);
        win.comp.set_o_width(w_phys as f32 / scale);
        win.comp.set_o_height(h_phys as f32 / scale);
        // 位置直接用配置物理像素（与 activeAppShow 统一）
        let px = ui.ctrl_x;
        let py = ui.ctrl_y;
        win.comp.window().set_position(PhysicalPosition::new(px, py));
        win.comp
            .window()
            .set_size(WindowSize::Physical(PhysicalSize::new(
                w_phys as u32,
                h_phys as u32,
            )));
        win.comp.show().ok();
        // Windows：Slint 的 Physical 接口即真实物理像素（与浮窗一致）。
        // macOS：屏外窗口 scale_factor 会退化为 1.0，需再用 winit 物理接口校正一次。
        #[cfg(not(windows))]
        {
            use slint::winit_030::WinitWindowAccessor;
            win.comp.window().with_winit_window(|ww| {
                use slint::winit_030::winit::dpi::{
                    PhysicalPosition as WinitPos, PhysicalSize as WinitSize,
                };
                ww.set_outer_position(WinitPos::new(px, py));
                let _ = ww.request_inner_size(WinitSize::new(w_phys as u32, h_phys as u32));
            });
        }
        win.positioned = true;
        let title = win.title.clone();
        crate::winutil::apply_overlay_styles(&win.comp.window(), &title, click_through);
    }

    /// activeAppShow：显示当前激活应用路径
    fn update_app_path(&mut self) {
        // 每 500ms 调用一次：只 clone UiParams
        let ui = self.shared.cfg.read().unwrap().ui.clone();
        if !ui.active_app_show {
            if let Some(c) = &mut self.appwin {
                c.comp.hide().ok();
            }
            return;
        }
        if self.last_app_check.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.last_app_check = Instant::now();
        let path = crate::winutil::foreground_proc_path();
        let already = self
            .appwin
            .as_ref()
            .map(|c| c.last_text == path && c.positioned)
            .unwrap_or(false);
        if already {
            return;
        }
        self.last_app_path = path.clone();
        if self.appwin.is_none() {
            let comp = CtrlOverlay::new().expect("create AppPathOverlay");
            let title = "SKBAppPath".to_string();
            comp.set_overlay_id(title.as_str().into());
            self.appwin = Some(CtrlWin {
                comp,
                title,
                last_text: String::new(),
                positioned: false,
            });
        }
        let font = self.ensure_font(&ui.ctrl_text_font.clone());
        let win = self.appwin.as_mut().unwrap();
        win.last_text = path.clone();
        let (br, bg_, bb) = ui.ctrl_bgcolor;
        let bg_col = Color::from_argb_u8(ui.ctrl_opacity as u8, br, bg_, bb);
        win.comp.set_label(path.as_str().into());
        win.comp.set_bg_color(bg_col);
        win.comp.set_txt_color(Color::from_argb_u8(ui.ctrl_opacity as u8, ui.gui_text_color.0, ui.gui_text_color.1, ui.gui_text_color.2));
        win.comp.set_edge_color(bg_col);
        win.comp.set_show_edge(ui.gui_edge);
        win.comp.set_font_name(font.as_str().into());
        win.comp.set_font_size(ui.ctrl_text_size.max(12));
        win.comp.set_font_weight(400);
        // AHK 的 activeAppShow 用 ToolTip 显示：坐标与尺寸都是**物理像素**，不随 DPI 缩放。
        let scale = Self::scale_at_pos(ui.active_app_show_x, ui.active_app_show_y);
        win.comp.set_corner_radius(ui.ctrl_radius as f32 / scale);
        // 宽度按内容自适应：单行实测文本宽度（物理像素）；macOS 无 GDI 测量，按字符宽度估算
        let font_size = ui.ctrl_text_size.max(12);
        let font_px = (font_size as f32 * scale).round().max(1.0) as i32;
        let (_lines, _line_h, measured) =
            crate::winutil::measure_text(&path, &font, font_px, false, 0);
        let text_w = if measured > 0 {
            measured
        } else {
            let units: f32 = path
                .chars()
                .map(|c| if (c as u32) > 0x2E7F { 1.0 } else { 0.55 })
                .sum();
            (units * font_px as f32).round() as i32
        };
        // 上限 = 所在显示器右边界 - 窗口 x - 余量；超宽路径由 Text 的 elide 截断
        let max_w = {
            let mons = MONITORS.with(|m| m.borrow().clone());
            let mon = mons
                .iter()
                .copied()
                .find(|r| {
                    ui.active_app_show_x >= r.0
                        && ui.active_app_show_x < r.2
                        && ui.active_app_show_y >= r.1
                        && ui.active_app_show_y < r.3
                })
                .or_else(|| mons.first().copied());
            match mon {
                Some((_ml, _mt, mr, _mb)) => (mr - ui.active_app_show_x - 16).max(200),
                None => 1920,
            }
        };
        // Text 左右共 12px 物理边距；内容不足 200 时保持最小宽度
        let w_phys = (text_w + (12.0 * scale).round() as i32).clamp(200, max_w);
        let h_phys = (font_size as f32 * scale * 1.8).round().max(1.0) as i32;
        win.comp.set_o_width(w_phys as f32 / scale);
        win.comp.set_o_height(h_phys as f32 / scale);
        let px = ui.active_app_show_x;
        let py = ui.active_app_show_y;
        win.comp.window().set_position(PhysicalPosition::new(px, py));
        win.comp
            .window()
            .set_size(WindowSize::Physical(PhysicalSize::new(
                w_phys as u32,
                h_phys as u32,
            )));
        win.comp.show().ok();
        // Windows：Slint 的 Physical 接口即真实物理像素（与浮窗一致）。
        // macOS：屏外窗口 scale_factor 会退化为 1.0，需再用 winit 物理接口校正一次。
        #[cfg(not(windows))]
        {
            use slint::winit_030::WinitWindowAccessor;
            win.comp.window().with_winit_window(|ww| {
                use slint::winit_030::winit::dpi::{
                    PhysicalPosition as WinitPos, PhysicalSize as WinitSize,
                };
                ww.set_outer_position(WinitPos::new(px, py));
                let _ = ww.request_inner_size(WinitSize::new(w_phys as u32, h_phys as u32));
            });
        }
        win.positioned = true;
        let title = win.title.clone();
        crate::winutil::apply_overlay_styles(&win.comp.window(), &title, true);
    }

    /// 复查窗口样式（不可激活 / 点击穿透 / 工具窗口 / owner）。
    /// winit 在 show()、窗口属性变化时会按自身 WindowFlags 整体重写 GWL_EXSTYLE，
    /// 把这里加的 WS_EX_NOACTIVATE / WS_EX_TRANSPARENT 抹掉，于是窗口会抢焦点、
    /// 挡住鼠标点击（表现为"影响输入"）。因此每帧都复查一次：
    /// apply_overlay_styles 内部先比较当前样式，已就位时直接返回，不会闪烁。
    fn apply_styles(&mut self) {
        let click_through = self.shared.cfg.read().unwrap().ui.gui_trans;
        for ov in self.overlays.iter_mut() {
            crate::winutil::apply_overlay_styles(&ov.comp.window(), &ov.title, click_through);
        }
        // 控制键窗常驻可见（只是移到屏幕外），文本为空时同样必须套样式，
        // 否则它会在任务栏留下一个按钮
        if let Some(c) = &mut self.ctrl {
            crate::winutil::apply_overlay_styles(&c.comp.window(), &c.title, click_through);
        }
        if let Some(c) = &mut self.appwin {
            crate::winutil::apply_overlay_styles(&c.comp.window(), &c.title, true);
        }
        if let Some(s) = &self.status {
            crate::winutil::apply_overlay_styles(&s.comp.window(), &s.title, click_through);
        }
    }
}

thread_local! {
    /// 当前显示器列表（每 2 秒刷新一次，跟随热插拔/分辨率变化）
    static MONITORS: std::cell::RefCell<Vec<crate::winutil::Rect>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// 启动时的显示器快照（不变），用于判断布局是否"恢复"回启动时的状态
    static MONITORS_SNAPSHOT: std::cell::RefCell<Vec<crate::winutil::Rect>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// 在 Slint 事件循环中运行 UI（阻塞，直到退出）
pub fn run(shared: Arc<Shared>, rx: Receiver<String>) {
    // 预加载显示器信息，并保存启动快照（用于判断布局是否恢复）
    let mons = crate::winutil::monitors();
    MONITORS.with(|m| *m.borrow_mut() = mons.clone());
    MONITORS_SNAPSHOT.with(|m| *m.borrow_mut() = mons);

    let mut manager = UiManager::new(shared.clone(), rx);

    let timer = slint::Timer::default();
    let mut tick_count = 0u64;
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(20),
        move || {
            tick_count += 1;
            // macOS：winit 在事件循环启动时会把激活策略重置为 Regular（Dock 出现图标），
            // 前 3 秒内持续纠正为 Accessory（20ms/次，代价可忽略）
            #[cfg(target_os = "macos")]
            if tick_count <= 150 {
                crate::winutil::set_dock_hidden();
            }
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
                if let Some(s) = manager.status.take() {
                    s.comp.hide().ok();
                }
            }

            // 消费显示队列（CallShow）
            while let Ok(token) = manager.rx.try_recv() {
                manager.show_txt(&token);
            }

            manager.tick_overlays();
            let fade_ms = shared_ref.cfg.read().unwrap().ui.gui_fade_ms;
            manager.tick_status(fade_ms);
            if tick_count % 3 == 0 {
                // ~60ms 刷新控制键状态（AHK 为 50ms）
                manager.update_ctrl();
                manager.update_app_path();
            }
            // winit 会在 show()/属性变更后重写 ex-style，逐帧复查（内部已就位即返回）
            manager.apply_styles();
            // macOS 托盘（NSStatusItem）必须在主线程、事件循环启动后创建并轮询菜单；
            // Windows 为 no-op（托盘在自己线程的消息循环里）
            crate::tray::poll(&shared_ref);
        },
    );

    // 注意：不能用 run_event_loop()——winit 后端在最后一个窗口隐藏后会退出事件循环，
    // 导致所有弹框渐隐后新弹框永远无法出现；until_quit 保持循环运行直到显式退出
    slint::run_event_loop_until_quit().expect("slint event loop");
}
