//! 统计核心：对应 AHK common.ahk / dialog.ahk 中的
//! PushTxt / RecordKey / AddRecord / AddKeyToMap / GetMinuteData(Core) / recordHistory
#![allow(dead_code)]
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use serde::Serialize;

use crate::config::Config;
use crate::winutil;

/// 分钟记录（对应 AHK MinuteRecords 元素）
#[derive(Clone, Debug, Serialize)]
pub struct MinuteRec {
    #[serde(rename = "Minute")]
    pub minute: String,
    #[serde(rename = "MouseCount")]
    pub mouse_count: i64, // 注意：非最终态时为全量累计值，变化时转为差值
    #[serde(rename = "KeyCount")]
    pub key_count: i64,
    #[serde(rename = "Distance")]
    pub distance: i64,
    #[serde(rename = "LastFlag", skip_serializing_if = "Option::is_none")]
    pub last_flag: Option<i32>,
    #[serde(rename = "Apps")]
    pub apps: BTreeMap<String, AppCount>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct AppCount {
    #[serde(rename = "Mouse")]
    pub mouse: i64,
    #[serde(rename = "Key")]
    pub key: i64,
}

/// 全局统计（对应 AHK AllKeyRecord + 相关全局变量）
pub struct Stats {
    pub keys: BTreeMap<String, i64>,
    pub minute_recs: Vec<MinuteRec>,
    pub record_history: Vec<String>, // 新的在前（InsertAt(1)）
    pub tick: i64,
    pub update_time: String,
    pub mouse_distance: i64,
    pub mouse_start: (i32, i32),
    pub global_key_count: i64,
    pub global_mouse_count: i64,
    pub pre_push_key: String,
    pub dirty: bool,
    pub proc_cache: winutil::ProcPathCache,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            keys: BTreeMap::new(),
            minute_recs: Vec::new(),
            record_history: Vec::new(),
            tick: winutil::now_ms(),
            update_time: String::new(),
            mouse_distance: 0,
            mouse_start: (0, 0),
            global_key_count: 0,
            global_mouse_count: 0,
            pre_push_key: String::new(),
            dirty: false,
            proc_cache: winutil::ProcPathCache::new(),
        }
    }

    fn add_key(&mut self, key: &str) {
        if key.is_empty() {
            return;
        }
        *self.keys.entry(key.to_string()).or_insert(0) += 1;
    }

    /// GetProcPath + preAppName 转换
    fn proc_path(&mut self, cfg: &Config) -> String {
        if winutil::system_locked() {
            return String::new(); // 锁屏不记录
        }
        let mut path = self.proc_cache.get();
        if path.is_empty() {
            return path;
        }
        if cfg.pre_app_name_enable {
            for (k, v) in &cfg.pre_app_name_list {
                if let Some(pat) = k.strip_prefix("Reg:") {
                    if let Ok(re) = regex::Regex::new(pat) {
                        if re.is_match(&path) {
                            path = v.clone();
                            break;
                        }
                    }
                } else if k == &path {
                    path = v.clone();
                    break;
                }
            }
        }
        path
    }

    /// GetMinuteDataCore(isMouse, isPress) 的移植
    pub fn minute_data(&mut self, cfg: &Config, is_mouse: bool, is_press: bool) {
        if !cfg.record_minute {
            return;
        }
        let curr_minute = winutil::now_string(true);
        let (init_mouse, init_key) = if is_press {
            (if is_mouse { 1 } else { 0 }, if is_mouse { 0 } else { 1 })
        } else {
            (0, 0)
        };
        let app_path = self.proc_path(cfg);
        if app_path.is_empty() {
            return;
        }
        let mut apps = BTreeMap::new();
        apps.insert(
            app_path.clone(),
            AppCount { mouse: init_mouse, key: init_key },
        );
        let curr_data = MinuteRec {
            minute: curr_minute.clone(),
            mouse_count: self.global_mouse_count,
            key_count: self.global_key_count,
            distance: self.mouse_distance,
            last_flag: None,
            apps,
        };

        let len = self.minute_recs.len();
        let max_len = 10usize;
        if len > max_len {
            self.minute_recs.remove(0);
        }
        let len = self.minute_recs.len();
        if len == 0 {
            self.minute_recs.push(curr_data);
            return;
        }
        let is_new_minute = self.minute_recs[len - 1].minute != curr_minute;
        if is_new_minute {
            // 更新之前的数据：由总数变成差值
            let last = self.minute_recs.last_mut().unwrap();
            last.mouse_count = self.global_mouse_count - last.mouse_count;
            last.key_count = self.global_key_count - last.key_count;
            last.distance = self.mouse_distance - last.distance;
            let empty = last.mouse_count == 0 && last.key_count == 0 && last.distance < 50;
            if empty {
                self.minute_recs[len - 1] = curr_data;
            } else {
                self.minute_recs.push(curr_data);
            }
        } else if is_press {
            let last = self.minute_recs.last_mut().unwrap();
            match last.apps.get_mut(&app_path) {
                Some(c) => {
                    if is_mouse {
                        c.mouse += 1;
                    } else {
                        c.key += 1;
                    }
                }
                None => {
                    last.apps.insert(app_path, AppCount { mouse: init_mouse, key: init_key });
                }
            }
        }
    }

    /// AddRecord：统计 + 分钟 + 进程维度
    fn add_record(&mut self, cfg: &Config, key: &str, is_mouse: bool) {
        if cfg.stat_proc_info {
            // GetAppInfo：以 App-Key- / App-Mouse- 维度汇总
            let path = self.proc_path(cfg);
            if !path.is_empty() {
                let app_key = if is_mouse {
                    format!("App-Mouse-{}", path)
                } else {
                    format!("App-Key-{}", path)
                };
                self.add_key(&app_key);
            }
        }
        if cfg.record_minute {
            if is_mouse {
                self.global_mouse_count += 1;
            } else {
                self.global_key_count += 1;
            }
            self.minute_data(cfg, is_mouse, true);
        }
        self.add_key(key);
    }

    /// RecordKey：分解修饰键统计（<^ -> LControl 等，重复计数修正）
    pub fn record_key(&mut self, cfg: &Config, txt: &str, is_mouse: bool) {
        self.add_record(cfg, txt, is_mouse);
        const MAPPING: [(&str, &str); 8] = [
            ("<^", "LControl"), (">^", "RControl"),
            ("<#", "LWin"), (">#", "RWin"),
            ("<!", "LAlt"), (">!", "RAlt"),
            ("<+", "LShift"), (">+", "RShift"),
        ];
        let mut left = txt.to_string();
        for (sym, name) in MAPPING {
            if left.contains(sym) {
                if self.pre_push_key == name {
                    if let Some(c) = self.keys.get_mut(name) {
                        *c -= 1; // 重复计数修正（与 AHK 一致）
                    }
                }
                self.add_record(cfg, name, is_mouse);
                left = left.replace(sym, "");
            }
        }
        if left != txt {
            self.add_record(cfg, &left, is_mouse);
        }
        self.pre_push_key = txt.to_string();
        self.update_time = winutil::now_string(false);
        if cfg.record_minute {
            // MinuteRecords 引用由序列化时动态附加
        }
        if cfg.record_history_max > 0 {
            if self.record_history.len() >= cfg.record_history_max {
                self.record_history.pop(); // 移除结尾（与 AHK RemoveAt(arrLen) 一致）
            }
            self.record_history.insert(0, txt.to_string());
        }
        self.dirty = true; // AutoSendData
    }
}

/// 跨线程共享状态
pub struct Shared {
    pub cfg: RwLock<Config>,
    pub stats: Mutex<Stats>,
    pub monitors: RwLock<Vec<winutil::Rect>>,
    pub screen_bounds: Mutex<(i32, i32, i32, i32)>, // minLeft,minTop,maxRight,maxBottom
    pub ui_tx: crossbeam_channel::Sender<String>,
    /// 实时按键文本 -> WebSocket 线程（对应 AHK handleWS.sendText）
    pub ws_tx: crossbeam_channel::Sender<String>,
    /// WebSocket 是否已连接（未连接时丢弃按键文本，不积压）
    pub ws_ready: AtomicBool,
    pub server_state: AtomicI32, // -1 未连接 0 失败 1 成功
    pub exit: AtomicBool,
    /// 重启模式：0 正常退出，1 重启程序（保留后台服务），2 完全重启（先退出后台服务）
    pub restart_mode: AtomicI32,
    pub ws_reload: AtomicBool,   // 收到 IniMonitor
    pub pcinfo_dirty: AtomicI32, // SendPCInfo flag（0 不发，否则发送对应 flag）
    pub show_key_generation: AtomicU64,
    pub version: String,
    pub major_version: String,
}

impl Shared {
    pub fn new(
        cfg: Config,
        ui_tx: crossbeam_channel::Sender<String>,
        ws_tx: crossbeam_channel::Sender<String>,
    ) -> Arc<Shared> {
        let full = env_full_version();
        let major = {
            let parts: Vec<&str> = full.split('.').collect();
            if parts.len() >= 2 {
                format!("{}.{}", parts[0], parts[1])
            } else {
                full.clone()
            }
        };
        Arc::new(Shared {
            cfg: RwLock::new(cfg),
            stats: Mutex::new(Stats::new()),
            monitors: RwLock::new(Vec::new()),
            screen_bounds: Mutex::new((0, 0, 0, 0)),
            ui_tx,
            ws_tx,
            ws_ready: AtomicBool::new(false),
            server_state: AtomicI32::new(-1),
            exit: AtomicBool::new(false),
            restart_mode: AtomicI32::new(0),
            ws_reload: AtomicBool::new(false),
            pcinfo_dirty: AtomicI32::new(0),
            show_key_generation: AtomicU64::new(0),
            version: full,
            major_version: major,
        })
    }
}

pub fn env_full_version() -> String {
    match std::env::var("FULL_VERSION") {
        Ok(v) if !v.is_empty() => v,
        _ => format!("{} (dev)", env!("CARGO_PKG_VERSION")),
    }
}

/// PushTxt 的完整移植：显示过滤 + 记录
pub fn push_txt(shared: &Shared, txt: &str, is_mouse: bool) {
    let cfg = shared.cfg.read().unwrap().clone();

    // 1) monitorProc / activeWindowProc 过滤
    let mut active_window_proc_result = false;
    if !cfg.monitor_proc.is_empty() || !cfg.active_window_proc.is_empty() {
        let hwnd = winutil::foreground_hwnd();
        if hwnd == 0 {
            return;
        }
        let pid = unsafe {
            let mut pid = 0u32;
            windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd as _, &mut pid);
            pid
        };
        let proc_path = winutil::process_path_by_pid(pid);
        let proc_name = std::path::Path::new(&proc_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Ok(re) = regex::Regex::new(&cfg.active_window_proc) {
            if !cfg.active_window_proc.is_empty() && re.is_match(&proc_name) {
                active_window_proc_result = true;
            }
        }
        if let Ok(re) = regex::Regex::new(&cfg.monitor_proc) {
            if !cfg.monitor_proc.is_empty() && !re.is_match(&proc_name) {
                return; // 不在监控进程清单内
            }
        }
    }

    let is_mouse_show = !is_mouse || (cfg.show_mouse_event >> 1 & 1) == 1;
    let is_mouse_record = !is_mouse || (cfg.show_mouse_event & 1) == 1;

    // 2) 界面显示
    if cfg.need_show_key && is_mouse_show {
        let mut mask_txt = txt.to_string();
        if cfg.hide_in_win_pwd && !is_mouse && winutil::focus_is_password() {
            mask_txt = "*".to_string();
        }
        if cfg.active_window_proc.is_empty() || active_window_proc_result {
            let _ = shared.ui_tx.send(mask_txt);
        }
    }

    // 3) 记录统计
    if cfg.need_record_key && is_mouse_record {
        let mut stats = shared.stats.lock().unwrap();
        stats.record_key(&cfg, txt, is_mouse);
    }
}

/// 鼠标移动距离累计（GetDistanceCore）
pub fn add_mouse_distance(shared: &Shared, x: i32, y: i32) {
    let bounds = *shared.screen_bounds.lock().unwrap();
    let (min_left, min_top, max_right, max_bottom) = bounds;
    if x < min_left || x > max_right || y < min_top || y > max_bottom {
        if y > max_bottom * 100 || x > max_right * 100 {
            return; // 锁屏等异常坐标，直接丢弃
        }
        shared.pcinfo_dirty.store(1, Ordering::Relaxed); // SendPCInfo(1)
        return;
    }
    let mut stats = shared.stats.lock().unwrap();
    let dx = (x - stats.mouse_start.0) as f64;
    let dy = (y - stats.mouse_start.1) as f64;
    let dist = (dx * dx + dy * dy).sqrt() as i64;
    if dist > 0 {
        let new_total = stats.mouse_distance + dist;
        stats.mouse_distance = new_total;
        stats.keys.insert("mouseDistance".into(), new_total);
        stats.mouse_start = (x, y);
    }
}
