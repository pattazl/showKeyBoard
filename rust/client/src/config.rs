//! 配置读取：完全使用 showKeyBoard.ini（含 showKeyBoard.desc.ini 默认值机制）。
//! 读取兼容 ANSI/GBK 与 UTF-8（含 BOM）；写入统一保存为 UTF-8（无 BOM）。
#![allow(dead_code)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
#[cfg(windows)]
use std::sync::atomic::AtomicIsize;
#[cfg(windows)]
use std::sync::atomic::Ordering;

#[cfg(windows)]
use crate::winutil;

/// preAppNameList 的单条替换规则：pattern 为 "Reg:..." 时预编译正则，
/// 避免每次按键都重新编译（热路径上的正则编译会造成大量堆分配与内存碎片）。
#[derive(Clone, Debug)]
pub struct PreAppRule {
    pub pattern: String,
    pub replacement: String,
    pub regex: Option<Arc<regex::Regex>>,
}

pub(crate) fn decode_ini(bytes: &[u8]) -> String {
    // 优先 UTF-8（含 BOM），失败则按 GBK 解码
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.trim_start_matches('\u{feff}').to_string();
    }
    let (cow, _, _) = encoding_rs::GBK.decode(bytes);
    cow.into_owned()
}

/// 追加/更新 INI 中的键值（重写整个文件）。
/// 读入时兼容 ANSI/GBK 与 UTF-8（含 BOM），写回时统一为 UTF-8（无 BOM）。
pub(crate) fn write_ini_value(path: &std::path::Path, section: &str, key: &str, value: &str) {
    let mut content = std::fs::read(path)
        .map(|b| decode_ini(&b))
        .unwrap_or_default();

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut in_section = false;
    let mut replaced = false;
    for line in lines.iter_mut() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            in_section = t[1..t.len() - 1].trim().eq_ignore_ascii_case(section);
            continue;
        }
        if in_section && t.starts_with(key) && t[key.len()..].trim_start().starts_with('=') {
            *line = format!("{}={}", key, value);
            replaced = true;
        }
    }
    if !replaced {
        if !lines.iter().any(|l| {
            let t = l.trim();
            t.starts_with('[') && t.ends_with(']') && t[1..t.len() - 1].trim().eq_ignore_ascii_case(section)
        }) {
            lines.push(format!("[{}]", section));
        }
        lines.push(format!("{}={}", key, value));
    }
    content = lines.join("\r\n");
    let _ = std::fs::write(path, content.as_bytes());
}

/// 把 ini 里的 httpDistPath 转成当前平台可用的路径。
/// Windows：兼容 AHK 写法（`\httpdist\dist\` 视为相对工作目录）；
/// macOS/Linux：反斜杠统一转成 `/`，且绝对路径保持绝对。
#[cfg(windows)]
fn resolve_http_path(work: &Path, spec: &str, first: bool) -> PathBuf {
    if first {
        work.join(spec.trim_start_matches(['\\', '/']).replace('/', "\\"))
    } else {
        work.join(spec.trim().replace('/', "\\"))
    }
}

#[cfg(not(windows))]
fn resolve_http_path(work: &Path, spec: &str, first: bool) -> PathBuf {
    let spec = spec.trim().replace('\\', "/");
    if first {
        // 与 Windows 分支语义一致：相对写法（\httpdist\dist\）需去掉开头分隔符，
        // 否则 PathBuf::join 遇到 "/httpdist/dist/" 会整体替换为根目录下的绝对路径
        work.join(spec.trim_start_matches('/'))
    } else {
        // 第二次尝试允许绝对路径（PathBuf::join 对绝对路径整体替换，符合预期）
        work.join(spec)
    }
}

/// 极简 INI 解析（节 -> 键 -> 值），注释 ; 或 #
fn parse_ini(text: &str) -> HashMap<String, HashMap<String, String>> {
    let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim().to_string();
            let mut val = line[pos + 1..].trim().to_string();
            // JS ini 写入时对包含 # 的值加了引号，去掉成对引号
            if val.len() >= 2 && val.starts_with('"') && val.ends_with('"') {
                val = val[1..val.len() - 1].to_string();
            }
            map.entry(section.clone()).or_default().insert(key, val);
        }
    }
    map
}

pub struct Ini {
    map: HashMap<String, HashMap<String, String>>,
}

impl Ini {
    pub fn load(path: &Path) -> Ini {
        let text = std::fs::read(path)
            .map(|b| decode_ini(&b))
            .unwrap_or_default();
        Ini { map: parse_ini(&text) }
    }
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.map.get(section)?.get(key).map(|s| s.as_str())
    }
}

fn split_pipe(s: &str) -> Vec<String> {
    s.split('|').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

/// 解析 "RRGGBB" -> (r,g,b)
pub fn parse_color(s: &str) -> (u8, u8, u8) {
    let s = s.trim().trim_start_matches('#');
    if s.len() >= 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            return (r, g, b);
        }
    }
    (0x11, 0xAA, 0x99)
}

/// UI 参数（dialog 节 + 部分 common 节），热重载使用
#[derive(Clone, Debug)]
pub struct UiParams {
    pub gui_width: i32,
    pub gui_height: i32,          // 0 自动
    pub gui_bgcolor: (u8, u8, u8),
    pub gui_bg_trans: bool,
    pub gui_trans: bool,
    pub gui_opacity: i32,         // 0-255
    pub gui_text_font: String,
    pub gui_text_size: i32,
    pub gui_text_bold: bool,
    pub gui_text_color: (u8, u8, u8),
    pub gui_life: u64,
    pub gui_interval: u64,
    pub gui_pos: String,          // TL TR BL BR
    pub gui_pos_xy: String,       // X / Y
    pub gui_pos_offset_x: i32,
    pub gui_pos_offset_y: i32,
    pub gui_dpiscale: bool,
    pub gui_monitor_num: i32,
    pub gui_margin: i32,
    pub gui_edge: bool,
    pub gui_radius: i32,
    pub gui_fade_ms: u64,
    pub txt_split: String,
    pub merge_char: bool,
    pub ctrl_state: bool,
    pub ctrl_x: i32,
    pub ctrl_y: i32,
    pub ctrl_text_size: i32,
    pub ctrl_width: i32,
    pub ctrl_height: i32,
    pub ctrl_bgcolor: (u8, u8, u8),
    pub ctrl_opacity: i32,
    pub ctrl_text_font: String,
    pub ctrl_text_bold: bool,
    pub ctrl_text_color: (u8, u8, u8),
    pub ctrl_radius: i32,
    pub ctrl_list: Vec<String>,
    pub skip_show: Vec<String>,
    pub active_app_show: bool,
    pub active_app_show_x: i32,
    pub active_app_show_y: i32,
    pub show_key_only_web: bool,
}

impl Default for UiParams {
    fn default() -> Self {
        Self {
            gui_width: 300, gui_height: 0,
            gui_bgcolor: (0x11, 0xAA, 0x99), gui_bg_trans: false, gui_trans: true,
            gui_opacity: 150, gui_text_font: "Verdana".into(), gui_text_size: 26,
            gui_text_bold: true, gui_text_color: (0xFF, 0, 0),
            gui_life: 7000, gui_interval: 1000, gui_pos: "BR".into(), gui_pos_xy: "Y".into(),
            gui_pos_offset_x: 0, gui_pos_offset_y: -50, gui_dpiscale: false,
            gui_monitor_num: 2, gui_margin: 5, gui_edge: true,
            gui_radius: 0, gui_fade_ms: 500, txt_split: " ".into(), merge_char: true,
            ctrl_state: true, ctrl_x: 10, ctrl_y: 10, ctrl_text_size: 20, ctrl_width: 240,
            ctrl_height: 0, ctrl_bgcolor: (0x11, 0xAA, 0x99), ctrl_opacity: 150,
            ctrl_text_font: "Verdana".into(), ctrl_text_bold: true, ctrl_text_color: (0xFF, 0, 0),
            ctrl_radius: 0,
            ctrl_list: ["Ctrl", "Alt", "LWin", "Shift", "RWin", "CapsLock", "NumLock", "ScrollLock"].iter().map(|s| s.to_string()).collect(),
            skip_show: vec![],
            active_app_show: false, active_app_show_x: 0, active_app_show_y: 0,
            show_key_only_web: false,
        }
    }
}

/// 运行期配置（DescRead 全量对应）
#[derive(Clone, Debug)]
pub struct Config {
    pub ini_path: PathBuf,
    pub desc_path: Option<PathBuf>, // showKeyBoard.desc.ini
    pub http_path: PathBuf,         // node 脚本所在目录
    /// 用户数据目录（%USERPROFILE%\Documents\ShowKeyBoard），与服务端一致
    pub data_dir: PathBuf,
    pub need_record_key: bool,

    // common
    pub joy_method: i32,
    pub need_traytip: bool,
    pub skip_record: Vec<String>,
    pub skip_ctrl_key: bool,
    pub show_mouse_event: i32,
    pub record_mouse_move: bool,
    pub record_minute: bool,
    pub need_show_key: bool,
    pub hotkey4_show: String,
    pub server_port: u16,
    pub active_window_proc: String,
    pub monitor_proc: String,
    pub max_keypress_count: i32,
    pub max_ctrlpress_count: i32,
    pub auto_save_2db: i32,
    pub remote_type: i32,
    pub hide_in_win_pwd: bool,
    pub stat_proc_info: bool,
    pub default_lang: String,
    pub server_exec_name: String,
    pub pre_app_name_enable: bool,
    pub pre_app_name_list: Vec<PreAppRule>, // 替换规则（含预编译正则）
    pub active_window_proc_re: Option<Arc<regex::Regex>>,
    pub monitor_proc_re: Option<Arc<regex::Regex>>,
    pub record_history_max: usize,
    pub check_server_max: i32,
    pub max_count_of_connect_fail: i32,
    pub show_http_debug: bool,
    pub share_db_path: String,
    pub share_db_name: String,
    pub share_db_hour: i32,
    pub share_db_exec: String,

    pub ui: UiParams,
    pub generation: u64,
}

impl Config {
    /// DescRead：先 desc.ini 再用户 ini
    fn desc_read(user: &Ini, desc: &Option<Ini>, section: &str, key: &str, default: &str) -> String {
        let mut val = default.to_string();
        if let Some(d) = desc {
            if let Some(v) = d.get(section, key) {
                val = v.to_string();
            }
        }
        if let Some(v) = user.get(section, key) {
            val = v.to_string();
        }
        val
    }

    fn desc_read_i32(user: &Ini, desc: &Option<Ini>, section: &str, key: &str, default: i32) -> i32 {
        Self::desc_read(user, desc, section, key, &default.to_string())
            .trim()
            .parse()
            .unwrap_or(default)
    }

    fn desc_read_bool(user: &Ini, desc: &Option<Ini>, section: &str, key: &str, default: bool) -> bool {
        matches!(
            Self::desc_read(user, desc, section, key, if default { "1" } else { "0" }).trim(),
            "1" | "true" | "True" | "TRUE"
        )
    }

    pub fn load() -> Config {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        // 与 AHK 一致：以工作目录为基准；启动器已保证工作目录 = exe 目录
        let work = std::env::current_dir().unwrap_or(exe_dir);
        let dir = data_dir();
        // 用户配置位于用户数据目录（与服务端一致）；首次运行迁移安装目录中的旧配置
        let ini_path = dir.join("showKeyBoard.ini");
        if !ini_path.exists() {
            let legacy = work.join("showKeyBoard.ini");
            if legacy.exists() {
                let _ = std::fs::copy(&legacy, &ini_path);
            }
        }
        let user_ini = Ini::load(&ini_path);

        // KeyList.txt 同样位于用户数据目录；首次运行从安装目录迁移旧文件
        // （macOS 区分大小写，统一使用 KeyList.txt）
        let key_list_path = dir.join("KeyList.txt");
        if !key_list_path.exists() {
            for name in ["KeyList.txt", "keyList.txt"] {
                let legacy = work.join(name);
                if legacy.exists() {
                    let _ = std::fs::copy(&legacy, &key_list_path);
                    break;
                }
            }
        }

        let http_dist_path = Self::desc_read(&user_ini, &None, "common", "httpDistPath", "\\httpdist\\dist\\");
        let mut http_path = resolve_http_path(&work, &http_dist_path, true);
        if !http_path.exists() {
            http_path = resolve_http_path(&work, &http_dist_path, false);
        }

        let desc_path = {
            let p = http_path.join("showKeyBoard.desc.ini");
            if p.exists() { Some(Ini::load(&p)) } else { None }
        };
        let desc = &desc_path;

        let mut need_record_key = Self::desc_read_bool(&user_ini, desc, "common", "needRecordKey", true);
        if !http_path.exists() {
            need_record_key = false; // 后端不存在则不启动记录
        }

        let joy_method = Self::desc_read_i32(&user_ini, desc, "common", "joyMethod", 2);
        let server_port = Self::desc_read_i32(&user_ini, desc, "common", "serverPort", 19999) as u16;

        let pre_app_name_enable = Self::desc_read_bool(&user_ini, desc, "common", "preAppNameEnable", false);
        let mut pre_app_name_list = Vec::new();
        if pre_app_name_enable {
            let raw = Self::desc_read(&user_ini, desc, "common", "preAppNameList", "{}");
            let raw = raw.replace("\\\\", "\\").replace("\\\"", "\"");
            if let Ok(serde_json::Value::Object(obj)) = serde_json::from_str::<serde_json::Value>(&raw) {
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        let regex = k.strip_prefix("Reg:").and_then(|pat| {
                            regex::Regex::new(pat).ok().map(Arc::new)
                        });
                        pre_app_name_list.push(PreAppRule {
                            pattern: k,
                            replacement: s.to_string(),
                            regex,
                        });
                    }
                }
            }
        }

        let active_window_proc = Self::desc_read(&user_ini, desc, "common", "activeWindowProc", "");
        let monitor_proc = Self::desc_read(&user_ini, desc, "common", "monitorProc", "");
        // 正则只在加载/热重载时编译一次，热路径直接复用（Arc 共享，clone 廉价）
        let active_window_proc_re = if active_window_proc.is_empty() {
            None
        } else {
            regex::Regex::new(&active_window_proc).ok().map(Arc::new)
        };
        let monitor_proc_re = if monitor_proc.is_empty() {
            None
        } else {
            regex::Regex::new(&monitor_proc).ok().map(Arc::new)
        };

        let show_mouse_event = Self::desc_read_i32(&user_ini, desc, "common", "showMouseEvent", 1);
        let ui = UiParams {
            gui_width: Self::desc_read_i32(&user_ini, desc, "dialog", "guiWidth", 300),
            gui_height: Self::desc_read_i32(&user_ini, desc, "dialog", "guiHeigth", 0),
            gui_bgcolor: parse_color(&Self::desc_read(&user_ini, desc, "dialog", "guiBgcolor", "11AA99")),
            gui_bg_trans: Self::desc_read_i32(&user_ini, desc, "dialog", "guiBgTrans", 0) == 1,
            gui_trans: Self::desc_read_i32(&user_ini, desc, "dialog", "guiTrans", 1) == 1,
            gui_opacity: Self::desc_read_i32(&user_ini, desc, "dialog", "guiOpacity", 150).clamp(0, 255),
            gui_text_font: Self::desc_read(&user_ini, desc, "dialog", "guiTextFont", "Verdana"),
            gui_text_size: Self::desc_read_i32(&user_ini, desc, "dialog", "guiTextSize", 26),
            gui_text_bold: Self::desc_read(&user_ini, desc, "dialog", "guiTextWeight", "bold").eq_ignore_ascii_case("bold"),
            gui_text_color: parse_color(&Self::desc_read(&user_ini, desc, "dialog", "guiTextColor", "FF0000")),
            gui_life: Self::desc_read_i32(&user_ini, desc, "dialog", "guiLife", 7000).max(0) as u64,
            gui_interval: Self::desc_read_i32(&user_ini, desc, "dialog", "guiInterval", 1000).max(0) as u64,
            gui_pos: Self::desc_read(&user_ini, desc, "dialog", "guiPos", "BR").to_uppercase(),
            gui_pos_xy: Self::desc_read(&user_ini, desc, "dialog", "guiPosXY", "Y").to_uppercase(),
            gui_pos_offset_x: Self::desc_read_i32(&user_ini, desc, "dialog", "guiPosOffsetX", 0),
            gui_pos_offset_y: Self::desc_read_i32(&user_ini, desc, "dialog", "guiPosOffsetY", -50),
            gui_dpiscale: Self::desc_read_i32(&user_ini, desc, "dialog", "guiDpiscale", 0) == 1,
            gui_monitor_num: Self::desc_read_i32(&user_ini, desc, "dialog", "guiMonitorNum", 2),
            gui_margin: Self::desc_read_i32(&user_ini, desc, "dialog", "guiMargin", 5),
            gui_edge: Self::desc_read_i32(&user_ini, desc, "dialog", "guiEdge", 1) == 1,
            gui_radius: Self::desc_read_i32(&user_ini, desc, "dialog", "guiRadius", 0),
            gui_fade_ms: Self::desc_read_i32(&user_ini, desc, "dialog", "guiFadeMs", 500).max(0) as u64,
            txt_split: Self::desc_read(&user_ini, desc, "dialog", "txtSplit", " "),
            merge_char: Self::desc_read_i32(&user_ini, desc, "dialog", "mergeChar", 1) == 1,
            ctrl_state: Self::desc_read_i32(&user_ini, desc, "common", "ctrlState", 1) == 1,
            ctrl_x: Self::desc_read_i32(&user_ini, desc, "dialog", "ctrlX", 10),
            ctrl_y: Self::desc_read_i32(&user_ini, desc, "dialog", "ctrlY", 10),
            ctrl_text_size: Self::desc_read_i32(&user_ini, desc, "dialog", "ctrlTextSize", 20),
            ctrl_width: Self::desc_read_i32(&user_ini, desc, "dialog", "ctrlWidth", 240),
            ctrl_height: Self::desc_read_i32(&user_ini, desc, "dialog", "ctrlHeight", 0),
            ctrl_bgcolor: parse_color(&Self::desc_read(&user_ini, desc, "dialog", "ctrlBgcolor", "11AA99")),
            ctrl_opacity: Self::desc_read_i32(&user_ini, desc, "dialog", "ctrlOpacity", 150).clamp(0, 255),
            ctrl_text_font: Self::desc_read(&user_ini, desc, "dialog", "ctrlTextFont", "Verdana"),
            ctrl_text_bold: Self::desc_read(&user_ini, desc, "dialog", "ctrlTextWeight", "bold").eq_ignore_ascii_case("bold"),
            ctrl_text_color: parse_color(&Self::desc_read(&user_ini, desc, "dialog", "ctrlTextColor", "FF0000")),
            ctrl_radius: Self::desc_read_i32(&user_ini, desc, "dialog", "ctrlRadius", 0),
            ctrl_list: split_pipe(&Self::desc_read(&user_ini, desc, "dialog", "ctrlList", "Ctrl|Alt|LWin|Shift|RWin|CapsLock|NumLock|ScrollLock")),
            skip_show: split_pipe(&Self::desc_read(&user_ini, desc, "dialog", "skipShow", "")),
            active_app_show: Self::desc_read_i32(&user_ini, desc, "common", "activeAppShow", 0) == 1,
            active_app_show_x: Self::desc_read_i32(&user_ini, desc, "dialog", "activeAppShowX", 0),
            active_app_show_y: Self::desc_read_i32(&user_ini, desc, "dialog", "activeAppShowY", 0),
            show_key_only_web: Self::desc_read_i32(&user_ini, desc, "common", "showKeyOnlyWeb", 0) == 1,
        };

        Config {
            ini_path,
            desc_path: None, // desc 已解析为 Ini，外部无需保存
            http_path,
            data_dir: dir,
            need_record_key,
            joy_method,
            need_traytip: Self::desc_read_bool(&user_ini, desc, "common", "needTraytip", true),
            skip_record: split_pipe(&Self::desc_read(&user_ini, desc, "common", "skipRecord", "")),
            skip_ctrl_key: Self::desc_read_i32(&user_ini, desc, "common", "skipCtrlKey", 0) == 1,
            show_mouse_event,
            record_mouse_move: Self::desc_read_bool(&user_ini, desc, "common", "recordMouseMove", false),
            record_minute: Self::desc_read_bool(&user_ini, desc, "common", "recordMinute", true),
            need_show_key: Self::desc_read_bool(&user_ini, desc, "common", "needShowKey", true),
            hotkey4_show: Self::desc_read(&user_ini, desc, "common", "hotkey4Show", "^!+s"),
            server_port,
            active_window_proc,
            monitor_proc,
            active_window_proc_re,
            monitor_proc_re,
            max_keypress_count: Self::desc_read_i32(&user_ini, desc, "common", "maxKeypressCount", 10),
            max_ctrlpress_count: Self::desc_read_i32(&user_ini, desc, "common", "maxCtrlpressCount", 1),
            auto_save_2db: Self::desc_read_i32(&user_ini, desc, "common", "autoSave2Db", 180),
            remote_type: Self::desc_read_i32(&user_ini, desc, "common", "remoteType", 0),
            hide_in_win_pwd: Self::desc_read_bool(&user_ini, desc, "common", "hideInWinPwd", false),
            stat_proc_info: Self::desc_read_bool(&user_ini, desc, "common", "statProcInfo", true),
            default_lang: Self::desc_read(&user_ini, desc, "common", "defaultLang", "zh-CN"),
            server_exec_name: Self::desc_read(
                &user_ini,
                desc,
                "common",
                "serverExecName",
                if cfg!(windows) { "node.exe" } else { "node" },
            ),
            pre_app_name_enable,
            pre_app_name_list,
            record_history_max: Self::desc_read_i32(&user_ini, desc, "common", "recordHistoryMax", 50).max(0) as usize,
            check_server_max: Self::desc_read_i32(&user_ini, desc, "common", "CheckServerMax", 9),
            max_count_of_connect_fail: Self::desc_read_i32(&user_ini, desc, "common", "maxCountOfConnectFail", 60),
            show_http_debug: Self::desc_read_bool(&user_ini, desc, "common", "showHttpDebug", false),
            share_db_path: Self::desc_read(&user_ini, desc, "common", "shareDbPath", "./share"),
            share_db_name: Self::desc_read(&user_ini, desc, "common", "shareDbName", ""),
            share_db_hour: Self::desc_read_i32(&user_ini, desc, "common", "shareDbHour", 0),
            share_db_exec: Self::desc_read(&user_ini, desc, "common", "shareDbExec", ""),
            ui,
            generation: 1,
        }
    }

    /// 服务器地址
    pub fn server_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.server_port)
    }
    pub fn server_ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.server_port)
    }

    /// skipKeys：与 AHK getAllSkipKey 一致，"{Name}" 集合
    pub fn skip_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        if self.skip_ctrl_key {
            for k in &self.ui.ctrl_list {
                keys.push(k.clone());
                // Ctrl -> LControl/RControl 等展开
                match k.as_str() {
                    "Ctrl" => { keys.push("LControl".into()); keys.push("RControl".into()); }
                    "Alt" => { keys.push("LAlt".into()); keys.push("RAlt".into()); }
                    "Shift" => { keys.push("LShift".into()); keys.push("RShift".into()); }
                    "LWin" => { keys.push("RWin".into()); }
                    "LCtrl" => { keys.push("RCtrl".into()); }
                    _ => {}
                }
            }
        }
        for k in &self.skip_record {
            keys.push(k.clone());
        }
        keys
    }

    pub fn server_url_print(&self) -> String {
        format!("http://127.0.0.1:{}", self.server_port)
    }

    pub fn gen(&self) -> u64 {
        self.generation
    }
}

/// KeyList.txt 按键转换映射
pub struct KeyMapping {
    map: Vec<(String, String)>, // 保持文件顺序，转换时长键优先
}

impl KeyMapping {
    /// 优先从用户数据目录读取 KeyList.txt，未找到时回退到当前工作目录（安装目录）
    pub fn load(dir: &Path) -> KeyMapping {
        let file = {
            let p = dir.join("KeyList.txt");
            if p.exists() {
                p
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join("KeyList.txt")
            }
        };
        let mut map = Vec::new();
        if let Ok(bytes) = std::fs::read(&file) {
            let text = String::from_utf8_lossy(&bytes);
            for line in text.lines() {
                if let Some(pos) = line.find(':') {
                    let k = line[..pos].trim();
                    let v = line[pos + 1..].trim();
                    if !k.is_empty() && !v.is_empty() {
                        map.push((k.to_string(), v.to_string()));
                    }
                }
            }
            // 长键名优先替换，避免子串误替换
            map.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        }
        KeyMapping { map }
    }
    /// ConvertTxt：完全匹配优先，其次子串替换
    pub fn convert(&self, t: &str) -> String {
        for (k, v) in &self.map {
            if k == t {
                return v.clone();
            }
        }
        let mut out = t.to_string();
        for (k, v) in &self.map {
            if out.contains(k.as_str()) {
                out = out.replace(k.as_str(), v);
            }
        }
        out
    }
}

/// 单实例互斥（对应 AHK #SingleInstance Ignore）
/// - Windows：命名互斥体
/// - macOS：锁文件 + pid 存活检测（崩溃残留会被自动清理）
pub struct SingleInstance {
    #[cfg(windows)]
    _handle: windows_sys::Win32::Foundation::HANDLE,
    #[cfg(target_os = "macos")]
    path: std::path::PathBuf,
}

/// Windows 保存互斥句柄，供重启前主动释放（新进程需要重新获取）
#[cfg(windows)]
static INSTANCE_HANDLE: AtomicIsize = AtomicIsize::new(0);

/// macOS 保存锁文件路径，供重启前主动释放
#[cfg(target_os = "macos")]
static INSTANCE_PATH: std::sync::Mutex<Option<std::path::PathBuf>> = std::sync::Mutex::new(None);

#[cfg(windows)]
impl SingleInstance {
    pub fn acquire(name: &str) -> Option<SingleInstance> {
        use windows_sys::Win32::System::Threading::CreateMutexW;
        let wname = winutil::wide(name);
        unsafe {
            let h = CreateMutexW(std::ptr::null(), 0, wname.as_ptr());
            if h.is_null() {
                return None;
            }
            // ERROR_ALREADY_EXISTS(183) 表示已有实例
            let err = windows_sys::Win32::Foundation::GetLastError();
            if err == 183 {
                return None;
            }
            INSTANCE_HANDLE.store(h as isize, Ordering::Relaxed);
            Some(SingleInstance { _handle: h })
        }
    }
}

#[cfg(target_os = "macos")]
impl SingleInstance {
    pub fn acquire(name: &str) -> Option<SingleInstance> {
        use std::io::Write;
        let path = std::env::temp_dir().join(format!("{}.lock", name));
        for _ in 0..2 {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut f) => {
                    let _ = write!(f, "{}", std::process::id());
                    *INSTANCE_PATH.lock().unwrap() = Some(path.clone());
                    return Some(SingleInstance { path });
                }
                Err(_) => {
                    // 锁文件已存在：检查记录的 pid 是否存活，崩溃残留则清理后重试
                    let alive = std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|s| s.trim().parse::<i32>().ok())
                        .map(|pid| unsafe { libc::kill(pid, 0) } == 0)
                        .unwrap_or(false);
                    if alive {
                        return None;
                    }
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
        None
    }
}

/// 重启前释放单实例互斥，允许新进程立即启动（对应 AHK Reload）
#[cfg(windows)]
pub fn release_single_instance() {
    let h = INSTANCE_HANDLE.swap(0, Ordering::Relaxed);
    if h != 0 {
        unsafe { windows_sys::Win32::Foundation::CloseHandle(h as _) };
    }
}

/// 重启前释放单实例锁文件，允许新进程立即启动
#[cfg(target_os = "macos")]
pub fn release_single_instance() {
    if let Some(p) = INSTANCE_PATH.lock().unwrap().take() {
        let _ = std::fs::remove_file(p);
    }
}

/// 用户数据目录（与服务端一致）：Windows 为 %USERPROFILE%\Documents\ShowKeyBoard
pub fn data_dir() -> PathBuf {
    let home = if cfg!(windows) {
        std::env::var_os("USERPROFILE")
    } else {
        std::env::var_os("HOME")
    };
    let dir = match home {
        Some(h) => PathBuf::from(h).join("Documents").join("ShowKeyBoard"),
        None => std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ShowKeyBoard"),
    };
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// 写 PID 文件（kb.pid），getKeyInput 兼容保留
pub fn write_pid_file(dir: &Path) {
    let pid_file = dir.join("kb.pid");
    let _ = std::fs::write(pid_file, std::process::id().to_string());
}
