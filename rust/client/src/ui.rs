//! 原生浮窗 UI：按键浮窗（渐隐 / 圆角 / 自定义字体背景 / 多屏 / 方向排列）+ 控制键状态窗。
//! 对应 AHK dialog.ahk 的 ShowTxt / CreateGui / FadeUI / ReLayOut / ShowCtrlState。
//! 平台实现：Windows 见 ui/win.rs，macOS 见 ui/mac.rs。
#![allow(dead_code)]
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;

use crate::config::{Config, KeyMapping};
use crate::stats::Shared;

#[cfg(windows)]
mod win;
#[cfg(target_os = "macos")]
mod mac;
#[cfg(windows)]
use win as nat;
#[cfg(target_os = "macos")]
use mac as nat;

/// 浮窗颜色（RGBA，与配置里的颜色参数一致）
#[derive(Clone, Copy, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_argb_u8(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a }
    }
}

struct Overlay {
    comp: nat::PopupWindow,
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
    comp: nat::PopupWindow,
    last_text: String,
    /// 尺寸生效时的配置版本：配置变化（如 ctrlWidth 改 0）后即使文本没变也要重新应用
    cfg_gen: u64,
    /// 位置/尺寸是否已成功应用
    positioned: bool,
    /// 已生效的目标 frame（物理像素）：macOS 上重复提交会触发 AppKit 重算 frame，
    /// 所以目标不变时不再重复设置
    applied: Option<(i32, i32, u32, u32)>,
}

pub struct UiManager {
    shared: Arc<Shared>,
    rx: Receiver<String>,
    overlays: Vec<Overlay>,
    ctrl: Option<CtrlWin>,
    appwin: Option<CtrlWin>,
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
    /// 待确认的新显示器布局：显示配置切换会连续产生多次变化，
    /// 同一布局连续两次检测到才重建窗口，避免切换过程中反复销毁/创建原生窗口
    pending_mons: Option<Vec<crate::winutil::Rect>>,
}

impl UiManager {
    fn new(shared: Arc<Shared>, rx: Receiver<String>) -> UiManager {
        let keymap = KeyMapping::load(&shared.cfg.read().unwrap().data_dir);
        UiManager {
            shared,
            rx,
            overlays: Vec::new(),
            ctrl: None,
            appwin: None,
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
            pending_mons: None,
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
        } else if let Some(last) = self.overlays.last_mut() {
            // 复用最后一个窗口：先中止可能已开始的渐隐并恢复不透明。
            // 休眠/待机唤醒后窗口可能停在渐隐进度上，只更新文本不重置透明度，
            // 新按键就会以"透明度 0"的状态显示出来。
            last.fading = false;
            last.fade_start = None;
            last.comp.set_fade(1.0);
            last.texts.push(display.clone());
            last.tick = now;
        }

        // 更新最后一个窗口的文本与位置
        self.update_last_overlay(&cfg);
        self.relayout(&cfg);
    }

    fn create_overlay(&mut self, _cfg: &Config, texts: Vec<String>, tick: Instant) {
        // 原生窗口按需创建、淡出后销毁，不做预设窗口池
        let id = self.next_id;
        self.next_id += 1;
        let Some(comp) = nat::PopupWindow::new(id) else {
            return;
        };
        comp.set_fade(1.0);
        self.overlays.push(Overlay {
            comp,
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
        {
            let ov = &mut self.overlays[last_idx];
            let (x, y, pw, ph) = Self::apply_overlay_layout(cfg, &ov.comp, &merged, &font);
            ov.x = x;
            ov.y = y;
            ov.w = pw;
            ov.h = ph;
        }
        let ov = &self.overlays[last_idx];
        // 显示时显式恢复为不透明，避免休眠唤醒后沿用旧的渐隐透明度
        ov.comp.set_fade(1.0);
        ov.comp.apply_styles(cfg.ui.gui_trans);
    }

    /// 浮窗字体：配置字体无法显示 🖰/🖱、⎇/⇧ 等符号时回退到 Segoe UI Symbol
    fn overlay_font(&mut self, cfg: &Config, text: &str) -> String {
        let base = self.ensure_font(&cfg.ui.gui_text_font.clone());
        self.resolve_font(&base, text)
    }

    /// 给浮窗套用文本/颜色/尺寸并定位（按键浮窗与连接状态浮窗共用）
    fn apply_overlay_layout(
        cfg: &Config,
        comp: &nat::PopupWindow,
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
        let pw = if ui.gui_width > 0 {
            ((ui.gui_width as f32) * size_k).round().max(1.0) as i32
        } else {
            // guiWidth=0：宽度自适应单行文本（与主动应用路径窗同一测量方式）
            let (_, _, measured) =
                crate::winutil::measure_text(&merged, font, font_px, ui.gui_text_bold, 0);
            let text_w = if measured > 0 {
                measured
            } else {
                let units: f32 = merged
                    .chars()
                    .map(|c| if (c as u32) > 0x2E7F { 1.0 } else { 0.55 })
                    .sum();
                (units * font_px as f32).round() as i32
            };
            (text_w + (12.0 * scale).round() as i32 + text_w / 50).max(40)
        };
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
        comp.set_label(merged.as_str());
        comp.set_bg_color(bg);
        comp.set_txt_color(tc);
        comp.set_edge_color(edge_color);
        comp.set_show_edge(ui.gui_edge);
        comp.set_font_name(font);
        comp.set_font_size(ui.gui_text_size);
        comp.set_font_weight(if ui.gui_text_bold { 700 } else { 400 });
        comp.set_scale(scale);
        comp.set_corner_radius(if ui.gui_dpiscale {
            ui.gui_radius as f32 / scale
        } else {
            ui.gui_radius as f32
        });
        // macOS：顶部锚定（TL/TR）时 Y 与 ctrlY 一致，从菜单栏下方的可视区顶部算偏移。
        // 这里直接换算成绝对物理坐标，后续窗口依次排布（relayout）才能从正确位置开始。
        #[cfg(not(windows))]
        if !matches!(ui.gui_pos.as_str(), "BL" | "BR") {
            y += crate::winutil::screen_visible_top_physical(
                ml + (mr - ml) / 2,
                mt + (mb - mt) / 2,
                scale as f64,
            );
        }
        comp.set_frame(x, y, pw as u32, ph as u32, scale, false);
        comp.show();
        (x, y, pw, ph)
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
                #[cfg(windows)]
                o.comp.set_position(nx, ny);
                // macOS：绝对坐标 + 原生 setFrame，避免 winit 按当前高度换算导致顶部偏移
                #[cfg(not(windows))]
                {
                    let scale = Self::scale_at_pos(nx, ny);
                    nat::set_native_frame(&o.comp, nx, ny, ow, oh, scale, false);
                }
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
        let (gui_life, gui_fade_ms) = {
            let cfg = self.shared.cfg.read().unwrap();
            let ui = &cfg.ui;
            (ui.gui_life, ui.gui_fade_ms)
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
                    ov.comp.set_fade(0.0);
                    ov.fade_start = None;
                    ov.done = true;
                } else {
                    ov.comp.set_fade(1.0 - progress);
                }
            }
        }
        // 淡出完成的窗口直接从列表移除，Drop 时销毁原生窗口
        if self.overlays.iter().any(|ov| ov.done) {
            self.overlays.retain(|ov| !ov.done);
        }
        // 每 5 秒刷新显示器列表；布局变化（热插拔/分辨率/扩展模式）时重建所有浮窗，
        // 避免窗口停留在已消失的屏幕坐标上不可见。ctrl/appwin 由 update_* 自动重定位。
        if now.duration_since(self.last_monitor_check) >= Duration::from_secs(5) {
            self.last_monitor_check = now;
            let new_mons = crate::winutil::monitors();
            // 切换分辨率/显示器时系统可能短暂返回空列表或中间态：
            // 空列表保持现状，其它变化需连续两次检测一致后才真正重建，
            // 避免在显示配置仍在变化的过程中反复销毁/创建原生窗口。
            if !new_mons.is_empty() {
                let changed = MONITORS.with(|m| *m.borrow() != new_mons);
                if changed {
                    if self.pending_mons.as_ref() == Some(&new_mons) {
                        self.pending_mons = None;
                        MONITORS.with(|m| *m.borrow_mut() = new_mons);
                        // 窗口位置依赖旧布局：全部丢弃，下一次按键按新布局重建
                        self.overlays.clear();
                    } else {
                        self.pending_mons = Some(new_mons);
                    }
                } else {
                    self.pending_mons = None;
                }
            }
        }
    }

    /// ShowCtrlState：控制键状态窗
    fn update_ctrl(&mut self) {
        // 每 ~60ms 调用一次：只 clone UiParams（远小于整个 Config），
        // 避免持 cfg 读锁时调用 &mut self 方法的借用冲突。
        let (ui, cfg_gen) = {
            let c = self.shared.cfg.read().unwrap();
            (c.ui.clone(), c.generation)
        };
        if !ui.ctrl_state {
            if let Some(c) = &mut self.ctrl {
                c.comp.hide();
                c.applied = None;
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
        // 避免刷新闪烁（位置尚未成功应用时也要继续尝试）；
        // 配置版本变化时必须重新应用，否则 ctrlWidth 改动后文本不变会一直沿用旧尺寸
        if self
            .ctrl
            .as_ref()
            .map(|c| c.last_text == txt && c.positioned && c.cfg_gen == cfg_gen)
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
                c.comp.hide();
                // 清掉已应用的位置：下次显示同一组按键时仍需重新 show/定位
                c.applied = None;
            }
            return;
        }
        if self.ctrl.is_none() {
            let Some(comp) = nat::PopupWindow::new(self.next_id) else {
                return;
            };
            self.next_id += 1;
            self.ctrl = Some(CtrlWin {
                comp,
                last_text: String::new(),
                cfg_gen: 0,
                positioned: false,
                applied: None,
            });
        }
        let font = {
            let base = self.ensure_font(&ui.ctrl_text_font.clone());
            self.resolve_font(&base, &txt)
        };
        let click_through = ui.gui_trans;
        let win = self.ctrl.as_mut().unwrap();
        win.last_text = txt.clone();
        win.cfg_gen = cfg_gen;
        let (br, bg_, bb) = ui.ctrl_bgcolor;
        let (tr, tg, tb) = ui.ctrl_text_color;
        let bg_col = Color::from_argb_u8(ui.ctrl_opacity as u8, br, bg_, bb);
        win.comp.set_label(txt.as_str());
        win.comp.set_bg_color(bg_col);
        win.comp.set_txt_color(Color::from_argb_u8(ui.ctrl_opacity as u8, tr, tg, tb));
        win.comp.set_edge_color(bg_col);
        win.comp.set_show_edge(ui.gui_edge);
        win.comp.set_font_name(font.as_str());
        win.comp.set_font_size(ui.ctrl_text_size);
        win.comp.set_font_weight(if ui.ctrl_text_bold { 700 } else { 400 });
        // 控制键状态窗：配置里 ctrl_X/ctrl_Y 是屏幕物理像素（与 activeAppShow 一致）。
        // AHK 的 Gui +DPIScale 只影响 Gui 内部控件坐标，外部屏幕坐标永远是物理像素。
        // 尺寸上，Gui 内部控件坐标会自动按 DPI 缩放 → 配置尺寸是**逻辑像素**，
        // 因此物理尺寸 = 配置 × scale；Slint 逻辑属性 = 物理尺寸 / scale。
        let scale = Self::scale_at_pos(ui.ctrl_x, ui.ctrl_y);
        let w_phys = if ui.ctrl_width > 0 {
            (ui.ctrl_width as f32 * scale).round().max(1.0) as i32
        } else {
            // ctrlWidth=0：宽度按 Ctrl 状态文本自适应（与主动应用路径窗同一测量方式）。
            // 加粗必须与 set_font_weight 一致，否则测量偏窄 → Text 会 elide 省略尾部
            let font_px = ((ui.ctrl_text_size as f32) * scale).round().max(1.0) as i32;
            let (_lines, _line_h, measured) =
                crate::winutil::measure_text(&txt, &font, font_px, ui.ctrl_text_bold, 0);
            let text_w = if measured > 0 {
                measured
            } else {
                let units: f32 = txt
                    .chars()
                    .map(|c| if (c as u32) > 0x2E7F { 1.0 } else { 0.55 })
                    .sum();
                (units * font_px as f32).round() as i32
            };
            // 12px 物理边距 + 2% 余量吸收不同引擎的字宽差异；
            // 只保留很小的下限，避免短文本（如 "Ctrl"）被固定的 200px 下限撑宽
            (text_w + (12.0 * scale).round() as i32 + text_w / 50)
                .max((16.0 * scale).round() as i32)
        };
        let h_phys = if ui.ctrl_height != 0 {
            (ui.ctrl_height as f32 * scale).round().max(1.0) as i32
        } else {
            (ui.ctrl_text_size as f32 * scale * 1.8).round().max(1.0) as i32
        };
        win.comp.set_scale(scale);
        win.comp.set_corner_radius(ui.ctrl_radius as f32 / scale);
        // 位置直接用配置物理像素（与 activeAppShow 统一）
        let px = ui.ctrl_x;
        let py = ui.ctrl_y;
        // ctrlY 从菜单栏下方的可视区域顶部算偏移（macOS 原生层处理；Windows 忽略）
        win.comp.set_frame(px, py, w_phys as u32, h_phys as u32, scale, true);
        win.comp.show();
        win.positioned = true;
        win.comp.apply_styles(click_through);
    }

    /// activeAppShow：显示当前激活应用路径
    fn update_app_path(&mut self) {
        // 每 500ms 调用一次：只 clone UiParams
        let ui = self.shared.cfg.read().unwrap().ui.clone();
        if !ui.active_app_show {
            if let Some(c) = &mut self.appwin {
                c.comp.hide();
                c.applied = None;
            }
            return;
        }
        // 窗口尚未完成首次定位时不做 500ms 节流，保证下一 tick 立刻重试
        let need_pos = self.appwin.as_ref().is_some_and(|w| !w.positioned);
        if !need_pos && self.last_app_check.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.last_app_check = Instant::now();
        let path = crate::winutil::foreground_proc_path();
        self.last_app_path = path.clone();
        // 注意：不要因为文本未变就跳过定位。窗口刚创建时 frame 尺寸还是默认值，
        // macOS 下首帧定位会按旧高度做坐标翻转（顶部偏移）；每 500ms 重新应用一次
        // 位置/尺寸即可收敛（与 ctrl 窗一致，开销可忽略）。
        if self.appwin.is_none() {
            let Some(comp) = nat::PopupWindow::new(self.next_id) else {
                return;
            };
            self.next_id += 1;
            self.appwin = Some(CtrlWin {
                comp,
                last_text: String::new(),
                cfg_gen: 0,
                positioned: false,
                applied: None,
            });
        }
        // 字体与 Ctrl 状态窗一致（含符号字体回退）
        let font = {
            let base = self.ensure_font(&ui.ctrl_text_font.clone());
            self.resolve_font(&base, &path)
        };
        let win = self.appwin.as_mut().unwrap();
        win.last_text = path.clone();
        let (br, bg_, bb) = ui.ctrl_bgcolor;
        let bg_col = Color::from_argb_u8(ui.ctrl_opacity as u8, br, bg_, bb);
        let (tr, tg, tb) = ui.ctrl_text_color;
        win.comp.set_label(path.as_str());
        win.comp.set_bg_color(bg_col);
        win.comp
            .set_txt_color(Color::from_argb_u8(ui.ctrl_opacity as u8, tr, tg, tb));
        win.comp.set_edge_color(bg_col);
        win.comp.set_show_edge(ui.gui_edge);
        win.comp.set_font_name(font.as_str());
        win.comp.set_font_size(ui.ctrl_text_size);
        win.comp.set_font_weight(if ui.ctrl_text_bold { 700 } else { 400 });
        // AHK 的 activeAppShow 用 ToolTip 显示：坐标与尺寸都是**物理像素**，不随 DPI 缩放。
        let scale = Self::scale_at_pos(ui.active_app_show_x, ui.active_app_show_y);
        win.comp.set_scale(scale);
        win.comp.set_corner_radius(ui.ctrl_radius as f32 / scale);
        // 宽度按内容自适应：单行实测文本宽度（物理像素）。
        // 加粗必须与 set_font_weight 一致，否则测量偏窄 → Text 会 elide 省略尾部
        let font_size = ui.ctrl_text_size.max(1);
        let font_px = (font_size as f32 * scale).round().max(1.0) as i32;
        let (_lines, _line_h, measured) =
            crate::winutil::measure_text(&path, &font, font_px, ui.ctrl_text_bold, 0);
        let text_w = if measured > 0 {
            measured
        } else {
            let units: f32 = path
                .chars()
                .map(|c| if (c as u32) > 0x2E7F { 1.0 } else { 0.55 })
                .sum();
            (units * font_px as f32).round() as i32
        };
        // 宽度完全按内容自适应：不再按显示器边界截断，长路径允许窗口超出屏幕，
        // 保证文本不会被 elide 省略。12px 物理边距 + 2% 余量吸收不同引擎的字宽差异
        let w_phys = (text_w + (12.0 * scale).round() as i32 + text_w / 50).max(200);
        // 高度与 Ctrl 状态窗一致：配置了 ctrlHeigth 用它，否则按字号自动
        let h_phys = if ui.ctrl_height != 0 {
            (ui.ctrl_height as f32 * scale).round().max(1.0) as i32
        } else {
            (font_size as f32 * scale * 1.8).round().max(1.0) as i32
        };
        let px = ui.active_app_show_x;
        let py = ui.active_app_show_y;
        // 与 Ctrl 状态窗一致：X 绝对坐标，Y 从菜单栏下方可视区顶部算偏移（macOS 原生层处理）
        win.comp.set_frame(px, py, w_phys as u32, h_phys as u32, scale, true);
        win.comp.show();
        win.positioned = true;
    }

    /// 两个浮窗首次显示时窗口句柄可能尚未创建（定位失败会先隐藏），
    /// 此时需要每 tick 重试，而不是等 60ms 的常规刷新周期。
    fn needs_pos_retry(&self) -> bool {
        self.ctrl.as_ref().is_some_and(|w| !w.positioned)
            || self.appwin.as_ref().is_some_and(|w| !w.positioned)
    }

    /// 复查窗口样式（不可激活 / 点击穿透 / 工具窗口）。
    /// winit 时代窗口属性会被事件循环重写；原生化后样式由我们自己创建，
    /// 这里保留每帧复查，成本极低，可兜住外部工具/系统对扩展样式的修改。
    fn apply_styles(&mut self) {
        let click_through = self.shared.cfg.read().unwrap().ui.gui_trans;
        for ov in self.overlays.iter_mut() {
            ov.comp.apply_styles(click_through);
        }
        if let Some(c) = &mut self.ctrl {
            c.comp.apply_styles(click_through);
        }
        if let Some(c) = &mut self.appwin {
            c.comp.apply_styles(true);
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
    /// 定时器回调是否正在执行：Windows 下窗口操作会同步派发消息，
    /// 若中途再次触发定时器会造成 &mut 别名（UB），这里直接跳过重入
    static IN_TICK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

struct TickGuard;
impl Drop for TickGuard {
    fn drop(&mut self) {
        IN_TICK.with(|f| f.set(false));
    }
}

/// 运行 UI 主循环（阻塞，直到退出）
pub fn run(shared: Arc<Shared>, rx: Receiver<String>) {
    // 原生窗口需要显式声明 DPI 感知（原 Slint/winit 在启动时设置）
    #[cfg(windows)]
    crate::winutil::set_dpi_aware();
    // 预加载显示器信息，并保存启动快照（用于判断布局是否恢复）
    let mons = crate::winutil::monitors();
    MONITORS.with(|m| *m.borrow_mut() = mons.clone());
    MONITORS_SNAPSHOT.with(|m| *m.borrow_mut() = mons);

    let mut manager = UiManager::new(shared.clone(), rx);
    let mut tick_count = 0u64;
    let tick = move || {
        // 窗口操作在 Windows 上会同步派发消息，可能在本回调未返回时再次触发定时器；
        // 重入会让同一 &mut UiManager 出现别名（UB），这里直接跳过本次
        if IN_TICK.with(|f| f.replace(true)) {
            return;
        }
        let _tick_guard = TickGuard;
        tick_count += 1;
        if shared.exit.load(Ordering::Relaxed) {
            nat::request_exit();
            return;
        }
        // 配置热重载：generation 变化时刷新窗口参数
        let gen = shared.cfg.read().unwrap().generation;
        if gen != manager.generation {
            manager.generation = gen;
            // 旧窗口直接销毁（Drop 会关闭原生窗口），下一次按键按新配置重建
            manager.overlays.clear();
            if let Some(c) = &mut manager.ctrl {
                c.comp.hide();
            }
            manager.ctrl = None;
            if let Some(c) = &mut manager.appwin {
                c.comp.hide();
            }
            manager.appwin = None;
        }

        // 消费显示队列（CallShow）
        while let Ok(token) = manager.rx.try_recv() {
            manager.show_txt(&token);
        }

        manager.tick_overlays();
        if tick_count % 3 == 0 || manager.needs_pos_retry() {
            // ~60ms 刷新控制键状态（AHK 为 50ms）
            manager.update_ctrl();
            manager.update_app_path();
        }
        // 逐帧复查窗口扩展样式（内部未变化时不做系统调用）
        manager.apply_styles();
        // macOS 托盘（NSStatusItem）必须在主线程、事件循环启动后创建并轮询菜单；
        // Windows 为 no-op（托盘在自己线程的消息循环里）
        crate::tray::poll(&shared);
    };

    nat::run_loop(Box::new(tick));
}

/// 请求退出 UI 主循环（托盘菜单使用）。
pub fn request_exit() {
    nat::request_exit();
}
