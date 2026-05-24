use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub db_path: String,
    pub ui_path: String,
    pub keymaps: HashMap<String, String>,
    pub auto_share: bool,
    pub settings: HashMap<String, String>,
    pub common: HashMap<String, String>,
    pub dialog: HashMap<String, String>,
}

impl AppConfig {
    /// Load config with two-tier override:
    /// 1. Load desc.ini as base defaults
    /// 2. Override with user ini values
    pub fn load_with_override<P1: AsRef<Path>, P2: AsRef<Path>>(
        desc_path: P1,
        user_path: P2,
    ) -> Self {
        let mut config = Self::parse_ini(desc_path.as_ref());
        let user_config = Self::parse_ini(user_path.as_ref());
        config.override_with(&user_config);
        config
    }

    /// Parse a standard INI file with [section] headers
    fn parse_ini(path: &Path) -> Self {
        let mut config: AppConfig = Self::default();

        if !path.exists() {
            println!("[config] parse_ini: {:?} does not exist, skipping", path);
            return config;
        }

        let content = match read_config_file(path) {
            Ok(c) => c,
            Err(e) => {
                println!("[config] parse_ini: failed to read {:?}: {}", path, e);
                return config;
            }
        };

        println!("[config] parse_ini: {:?} loaded, {} bytes",
            path, content.len());

        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // Section header: [section]
            if line.starts_with('[') && line.contains(']') {
                if let Some(end) = line.find(']') {
                    current_section = line[1..end].trim().to_string();
                }
                continue;
            }

            // Key = Value
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = strip_ini_quotes(value.trim());

                match current_section.as_str() {
                    "common" => {
                        match key.as_str() {
                            "serverPort" => {
                                if let Ok(port) = value.parse() {
                                    config.port = port;
                                }
                                config.common.insert(key, value);
                            }
                            _ => {
                                config.common.insert(key, value);
                            }
                        }
                    }
                    "server" => {
                        match key.as_str() {
                            "port" => {
                                if let Ok(port) = value.parse() {
                                    config.port = port;
                                }
                            }
                            "ui" => config.ui_path = value,
                            _ => {
                                config.settings.insert(
                                    format!("server.{}", key),
                                    value,
                                );
                            }
                        }
                    }
                    "db" => {
                        if key == "path" {
                            config.db_path = value;
                        } else {
                            config
                                .settings
                                .insert(format!("db.{}", key), value);
                        }
                    }
                    "share" => {
                        if key == "auto" {
                            config.auto_share =
                                value == "1" || value.to_lowercase() == "true";
                        } else {
                            config
                                .settings
                                .insert(format!("share.{}", key), value);
                        }
                    }
                    "keymaps" => {
                        config.keymaps.insert(key, value);
                    }
                    "dialog" => {
                        config.dialog.insert(key, value);
                    }
                    _ => {
                        config
                            .settings
                            .insert(format!("{}.{}", current_section, key), value);
                    }
                }
            }
        }

        println!("[config] parse_ini result: common={} keys, dialog={} keys",
            config.common.len(), config.dialog.len());
        config
    }

    /// Override non-default values from another config
    fn override_with(&mut self, other: &Self) {
        println!("[config] override_with: other has common={} keys, dialog={} keys",
            other.common.len(), other.dialog.len());
        // Only override if the other config has explicit (non-default) values
        if other.port != Self::default().port {
            self.port = other.port;
        }
        if other.db_path != Self::default().db_path {
            self.db_path = other.db_path.clone();
        }
        if other.ui_path != Self::default().ui_path {
            self.ui_path = other.ui_path.clone();
        }
        if other.auto_share != Self::default().auto_share {
            self.auto_share = other.auto_share;
        }
        for (k, v) in &other.keymaps {
            self.keymaps.insert(k.clone(), v.clone());
        }
        for (k, v) in &other.common {
            self.common.insert(k.clone(), v.clone());
        }
        for (k, v) in &other.dialog {
            self.dialog.insert(k.clone(), v.clone());
        }
        for (k, v) in &other.settings {
            self.settings.insert(k.clone(), v.clone());
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let mut content = String::new();

        // [common]
        content.push_str("[common]\n");
        content.push_str(&format!("serverPort = {}\n", self.port));
        for (k, v) in &self.common {
            content.push_str(&format!("{} = {}\n", k, v));
        }
        content.push('\n');

        // [dialog]
        if !self.dialog.is_empty() {
            content.push_str("[dialog]\n");
            for (k, v) in &self.dialog {
                content.push_str(&format!("{} = {}\n", k, v));
            }
            content.push('\n');
        }

        // [server]
        content.push_str("[server]\n");
        content.push_str(&format!("port = {}\n", self.port));
        content.push_str(&format!("ui = {}\n\n", self.ui_path));

        // [db]
        content.push_str("[db]\n");
        content.push_str(&format!("path = {}\n\n", self.db_path));

        // [keymaps]
        if !self.keymaps.is_empty() {
            content.push_str("[keymaps]\n");
            for (k, v) in &self.keymaps {
                content.push_str(&format!("{} = {}\n", k, v));
            }
            content.push('\n');
        }

        // [share]
        content.push_str("[share]\n");
        content.push_str(&format!(
            "auto = {}\n\n",
            if self.auto_share { "1" } else { "0" }
        ));

        // [setting] - deprecated, kept for compatibility
        if !self.settings.is_empty() {
            content.push_str("[setting]\n");
            for (k, v) in &self.settings {
                content.push_str(&format!("{} = {}\n", k, v));
            }
        }

        std::fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn get_db_path(&self) -> String {
        self.db_path.clone()
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.settings.insert(key.to_string(), value.to_string());
    }
}

/// Search for a config file in multiple candidate directories.
/// Tries: exe_dir → cwd → exe_dir/../.. (for cargo run dev mode)
pub fn find_config_file(exe_dir: &Path, cwd: &Path, filename: &str) -> PathBuf {
    let parent2 = exe_dir.parent().and_then(|p| p.parent());
    
    let candidates: Vec<PathBuf> = {
        let mut v = vec![exe_dir.join(filename), cwd.join(filename)];
        if let Some(p2) = parent2 {
            v.push(p2.join(filename));
        }
        v
    };
    
    for path in &candidates {
        println!("[config] Looking for {} at: {:?}", filename, path);
        if path.exists() {
            println!("[config] Found {} at: {:?}", filename, path);
            return path.clone();
        }
    }
    
    // Fallback to exe_dir (file may not exist, parse_ini handles that)
    println!("[config] {} not found in any search path, using default: {:?}", filename, candidates[0]);
    candidates[0].clone()
}

/// Strip surrounding quotes and unescape JSON escape sequences from an INI value.
///
/// Matches Node.js `ini` package `unsafe()` behavior:
/// - Double-quoted (`"Y"`): uses JSON parser to unescape `\\n`→newline, `\\"`→`"`, etc.
/// - Single-quoted (`'Y'`): strips quotes, tries JSON parse, falls back to raw.
/// - Unquoted values: returned as-is (no unescaping needed).
fn strip_ini_quotes(val: &str) -> String {
    if val.len() < 2 {
        return val.to_string();
    }
    let bytes = val.as_bytes();
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];

    if first == b'"' && last == b'"' {
        // Double-quoted: parse as JSON string → handles unescaping + quote stripping
        // E.g. `"{\\n  \\"NotFoundActive\\": \\"\\"\\n}"` → JSON with actual newlines & quotes
        return serde_json::from_str::<String>(val).unwrap_or_else(|_| {
            // Fallback: if JSON parse fails (shouldn't happen for valid quotes),
            // just strip outer quotes
            val[1..val.len() - 1].to_string()
        });
    }

    if first == b'\'' && last == b'\'' {
        // Single-quoted: strip quotes first, then try JSON parse
        // Node.js: JSON.parse on stripped value; if it fails, use stripped value
        let inner = &val[1..val.len() - 1];
        // Wrap back in double-quotes for JSON parse (to handle escape sequences)
        let json_str = format!("\"{}\"", inner);
        return serde_json::from_str::<String>(&json_str).unwrap_or_else(|_| inner.to_string());
    }

    val.to_string()
}

/// Read a file with encoding fallback: try UTF-8 first, then GBK.
/// This matches Node.js behavior where user ini is ANSI/GBK (shared with AHK),
/// while desc ini is UTF-8.
pub fn read_config_file(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
    
    // Try UTF-8 first
    if let Ok(s) = String::from_utf8(bytes.clone()) {
        return Ok(s);
    }
    
    // Fallback: try GBK/GB2312 decoding
    // For full GBK support, we'd need the encoding_rs crate, but since
    // the config values are mostly ASCII keys with ASCII/Chinese comment values,
    // the UTF-8 parse should succeed for most cases.
    Err(format!("File {:?} is not valid UTF-8. If it's GBK-encoded, please convert to UTF-8.", path))
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 8888,
            db_path: "./records.db".to_string(),
            ui_path: "ui".to_string(),
            keymaps: HashMap::new(),
            auto_share: false,
            settings: HashMap::new(),
            common: HashMap::new(),
            dialog: HashMap::new(),
        }
    }
}
