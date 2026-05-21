use std::collections::HashMap;
use std::path::Path;

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
        let mut config = Self::default();

        if !path.exists() {
            return config;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return config,
        };

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
                let value = value.trim().to_string();

                match current_section.as_str() {
                    "common" => {
                        match key.as_str() {
                            "serverPort" => {
                                if let Ok(port) = value.parse() {
                                    config.port = port;
                                }
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

        config
    }

    /// Override non-default values from another config
    fn override_with(&mut self, other: &Self) {
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
