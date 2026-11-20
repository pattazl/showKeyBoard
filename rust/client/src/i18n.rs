//! 客户端提示文案：中文 / 英文。
//!
//! ini 中 `defaultLang` 为 `en-US` 时输出英文，其它值（含缺省 `zh-CN`）输出中文。
//! 平台相关文案（托盘菜单、气泡、macOS 授权对话框）统一从这里取，避免两边各写一份。

/// 界面语言
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Zh,
    En,
}

impl Lang {
    pub fn from_ini(value: &str) -> Self {
        if value.trim().eq_ignore_ascii_case("en-US") {
            Lang::En
        } else {
            Lang::Zh
        }
    }

    pub fn is_en(self) -> bool {
        matches!(self, Lang::En)
    }
}

/// 托盘菜单文案
pub struct MenuTexts {
    pub settings: &'static str,
    pub statistics: &'static str,
    pub show_key: &'static str,
    pub auto_start: &'static str,
    pub restart: &'static str,
    pub restart_all: &'static str,
    pub exit: &'static str,
}

pub fn menu_texts(lang: Lang) -> MenuTexts {
    match lang {
        Lang::En => MenuTexts {
            settings: "Setting",
            statistics: "Statistics",
            show_key: "Show/Hide Key",
            auto_start: "Auto Start",
            restart: "Reset Client",
            restart_all: "Full Reset",
            exit: "Exit",
        },
        Lang::Zh => MenuTexts {
            settings: "参数设置",
            statistics: "数据统计",
            show_key: "按键显示",
            auto_start: "开机启动",
            restart: "重启程序",
            restart_all: "完全重启",
            exit: "退出程序",
        },
    }
}

/// 启动气泡（needTraytip）
pub fn balloon_started(lang: Lang, ver: &str, hotkey: &str) -> String {
    match lang {
        Lang::En => {
            if hotkey.is_empty() {
                format!("ShowKeyBoard v{} started", ver)
            } else {
                format!("ShowKeyBoard v{} started, press {} to show/hide keys", ver, hotkey)
            }
        }
        Lang::Zh => {
            if hotkey.is_empty() {
                format!("ShowKeyBoard v{} 已启动", ver)
            } else {
                format!("ShowKeyBoard v{} 已启动，热键 {} 显示/隐藏", ver, hotkey)
            }
        }
    }
}

// ---------------- 后台服务状态 ----------------

pub fn status_connected(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Backend connected",
        Lang::Zh => "后台服务已连接",
    }
}

pub fn status_starting(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Starting backend service…",
        Lang::Zh => "正在启动后台服务…",
    }
}

pub fn status_started_connecting(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Backend started, connecting…",
        Lang::Zh => "后台服务已启动，正在连接…",
    }
}

pub fn status_lost_reconnecting(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Backend connection lost, reconnecting…",
        Lang::Zh => "后台服务连接中断，正在重连…",
    }
}

pub fn status_connecting_attempt(lang: Lang, attempt: u32) -> String {
    match lang {
        Lang::En => format!("Connecting to backend… (attempt {})", attempt),
        Lang::Zh => format!("正在连接后台服务…（第 {} 次）", attempt),
    }
}

pub fn status_connect_failed(lang: Lang, err: &str) -> String {
    match lang {
        Lang::En => format!("Backend connection failed: {}", err),
        Lang::Zh => format!("后台服务连接失败：{}", err),
    }
}

pub fn status_start_failed(lang: Lang, err: &str) -> String {
    match lang {
        Lang::En => format!("Failed to start backend: {}", err),
        Lang::Zh => format!("后台服务启动失败：{}", err),
    }
}

// ---------------- 后台服务启动/连接错误 ----------------

pub fn err_service_dir(lang: Lang, path: &str) -> String {
    match lang {
        Lang::En => format!("Backend service directory not found: {}", path),
        Lang::Zh => format!("未找到后台服务目录 {}", path),
    }
}

pub fn err_service_script(lang: Lang, path: &str) -> String {
    match lang {
        Lang::En => format!("Backend service script not found: {}", path),
        Lang::Zh => format!("未找到后台服务脚本 {}", path),
    }
}

pub fn err_spawn(lang: Lang, exec: &str, err: &str) -> String {
    match lang {
        Lang::En => format!("Failed to start {}: {}", exec, err),
        Lang::Zh => format!("启动 {} 失败：{}", exec, err),
    }
}

pub fn err_bad_response(lang: Lang, body: &str) -> String {
    let brief = body.trim();
    match lang {
        Lang::En => format!(
            "Unexpected response from backend: {}",
            if brief.is_empty() { "<empty>" } else { brief }
        ),
        Lang::Zh => format!(
            "服务返回异常响应：{}",
            if brief.is_empty() { "<空>" } else { brief }
        ),
    }
}

pub fn err_connect(lang: Lang, url: &str, err: &str) -> String {
    match lang {
        Lang::En => format!("Cannot connect to {}: {}", url, err),
        Lang::Zh => format!("无法连接 {}：{}", url, err),
    }
}

#[cfg(target_os = "macos")]
pub mod macos {
    use super::Lang;

    /// 辅助功能授权对话框：返回 (AppleScript 脚本, 用户点“去授权”时 stdout 中出现的按钮名)
    pub fn accessibility_dialog(lang: Lang) -> (&'static str, &'static str) {
        match lang {
            Lang::En => (
                concat!(
                    r#"display dialog "#,
                    r#""ShowKeyBoard needs to monitor keyboard and mouse activity.\n\n"#,
                    r#""Accessibility" permission has not been granted yet.\n"#,
                    r#"Click "Grant Access" to open System Settings. After enabling this app, "#
                    r#"capturing starts automatically within 10 seconds.""#,
                    r#" with title "ShowKeyBoard Needs Accessibility Permission""#,
                    r#" buttons {"Later", "Grant Access"} default button "Grant Access" with icon caution"#
                ),
                "Grant Access",
            ),
            Lang::Zh => (
                concat!(
                    r#"display dialog "#,
                    r#""ShowKeyBoard 需要监控键盘鼠标动作。\n\n"#,
                    r#"当前尚未获得「辅助功能」授权。\n"#,
                    r#"点击「去授权」打开系统设置，勾选本程序后，程序会在 10 秒内自动检测并开始捕获。""#,
                    r#" with title "ShowKeyBoard 需要辅助功能权限""#,
                    r#" buttons {"稍后", "去授权"} default button "去授权" with icon caution"#
                ),
                "去授权",
            ),
        }
    }
}
