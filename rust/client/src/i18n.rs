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
            exit: "Exit",
        },
        Lang::Zh => MenuTexts {
            settings: "参数设置",
            statistics: "数据统计",
            show_key: "按键显示",
            auto_start: "开机启动",
            restart: "重启程序",
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
                    r#""Accessibility permission has not been granted yet.\n"#,
                    r#"Click 'Grant Access' to open System Settings. After enabling this app, "#,
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
