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
}

impl AppConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        
        if !path.exists() {
            return Self::default();
        }
        
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let mut config = Self::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            
            if let Some((section_key, value)) = line.split_once('=') {
                let section_key = section_key.trim();
                
                if let Some((section, key)) = section_key.split_once('.') {
                    let section = section.trim();
                    let key = key.trim();
                    let value = value.trim();
                    
                    match section {
                        "server" => {
                            match key {
                                "port" => {
                                    if let Ok(port) = value.parse() {
                                        config.port = port;
                                    }
                                }
                                "ui" => config.ui_path = value.to_string(),
                                _ => {}
                            }
                        }
                        "db" => {
                            if key == "path" {
                                config.db_path = value.to_string();
                            }
                        }
                        "share" => {
                            if key == "auto" {
                                config.auto_share = value == "1" || value.to_lowercase() == "true";
                            }
                        }
                        "keymaps" => {
                            config.keymaps.insert(key.to_string(), value.to_string());
                        }
                        "setting" => {
                            config.settings.insert(key.to_string(), value.to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
        
        config
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let mut content = String::new();
        
        content.push_str("[server]\n");
        content.push_str(&format!("port = {}\n", self.port));
        content.push_str(&format!("ui = {}\n\n", self.ui_path));
        
        content.push_str("[db]\n");
        content.push_str(&format!("path = {}\n\n", self.db_path));
        
        if !self.keymaps.is_empty() {
            content.push_str("[keymaps]\n");
            for (k, v) in &self.keymaps {
                content.push_str(&format!("{} = {}\n", k, v));
            }
            content.push('\n');
        }
        
        content.push_str("[share]\n");
        content.push_str(&format!("auto = {}\n\n", if self.auto_share { "1" } else { "0" }));
        
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
        }
    }
}
