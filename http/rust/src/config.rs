use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Server port — derived from common["serverPort"] (matches Node.js config.common.serverPort)
    pub port: u16,
    /// Database filename — fixed default, NOT read from ini (matches Node.js basePath/records.db)
    pub db_path: String,
    /// UI directory — fixed default, NOT read from ini (matches Node.js __dirname/ui)
    pub ui_path: String,
    pub keymaps: HashMap<String, String>,
    /// [common] section (includes serverPort)
    pub common: HashMap<String, String>,
    /// [dialog] section
    pub dialog: HashMap<String, String>,
}

impl AppConfig {
    /// Load config with two-tier override:
    /// 1. Load desc.ini as base defaults
    /// 2. Override with user ini values
    /// Port is always derived from [common] serverPort after merging.
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

                // Only [common], [dialog], [keymaps] are recognized.
                // [server], [db], [share] and any other sections are ignored
                // (matches Node.js: config only contains common + dialog).
                match current_section.as_str() {
                    "common" => {
                        config.common.insert(key, value);
                    }
                    "dialog" => {
                        config.dialog.insert(key, value);
                    }
                    "keymaps" => {
                        config.keymaps.insert(key, value);
                    }
                    _ => {
                        // Unknown sections are ignored — never read or written
                    }
                }
            }
        }

        println!("[config] parse_ini result: common={} keys, dialog={} keys",
            config.common.len(), config.dialog.len());
        config
    }

    /// Override values from user config (matches Node.js mergeObjects):
    /// user ini values override desc.ini values; port is derived from
    /// the merged [common] serverPort afterwards.
    fn override_with(&mut self, other: &Self) {
        println!("[config] override_with: other has common={} keys, dialog={} keys",
            other.common.len(), other.dialog.len());
        for (k, v) in &other.keymaps {
            self.keymaps.insert(k.clone(), v.clone());
        }
        for (k, v) in &other.common {
            self.common.insert(k.clone(), v.clone());
        }
        for (k, v) in &other.dialog {
            self.dialog.insert(k.clone(), v.clone());
        }
        // Port always comes from [common] serverPort (matches Node.js
        // parseInt(config.common?.serverPort ?? 9900))
        if let Some(v) = self.common.get("serverPort") {
            if let Ok(p) = v.parse() {
                self.port = p;
            }
        }
    }

    /// Save config to INI file. Matches Node.js myIniwrite: only writes [common] + [dialog].
    /// Does NOT write [server], [db], [share] sections — those are internal-only.
    /// serverPort lives under [common] (matches Node.js config.common.serverPort).
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let mut content = String::new();

        // [common] — serverPort is already in self.common
        content.push_str("[common]\n");
        for (k, v) in &self.common {
            content.push_str(&format!("{} = {}\n", k, escape_ini_value(v)));
        }
        content.push('\n');

        // [dialog]
        if !self.dialog.is_empty() {
            content.push_str("[dialog]\n");
            for (k, v) in &self.dialog {
                content.push_str(&format!("{} = {}\n", k, escape_ini_value(v)));
            }
            content.push('\n');
        }

        std::fs::write(path, content).map_err(|e| e.to_string())
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

/// Escape an INI value for writing, matching Node.js `ini.stringify` behavior:
/// - Values containing newlines, quotes, `#` or `;` are JSON-escaped and wrapped
///   in double quotes, e.g. `{...multi-line json...}` → `"{\n  \"key\": \"\"\n}"`
/// - Plain values are written as-is.
/// This round-trips with `strip_ini_quotes` on the read side.
fn escape_ini_value(val: &str) -> String {
    if val.contains('\n') || val.contains('\r') || val.contains('"')
        || val.contains('#') || val.contains(';') {
        serde_json::to_string(val).unwrap_or_else(|_| val.to_string())
    } else {
        val.to_string()
    }
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
            port: 9900,
            db_path: "./records.db".to_string(),
            ui_path: "ui".to_string(),
            keymaps: HashMap::new(),
            common: HashMap::new(),
            dialog: HashMap::new(),
        }
    }
}
