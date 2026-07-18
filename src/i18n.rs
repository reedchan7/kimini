//! Host UI language (English / Simplified Chinese).
//!
//! Resolution order:
//! 1. `$KIMINI_LANG` (`en`, `zh`, `zh-CN`, `zh-Hans`, …)
//! 2. Saved preference (`lang` under the app config dir)
//! 3. System locale (`LANG` / `LC_ALL` / macOS `AppleLocale`)
//! 4. English (default)
//!
//! Chinese copy lives only in [`Strings::zh`]. Product docs stay English.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Supported UI languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh",
        }
    }

    /// Parse a BCP-47 / locale-ish tag into a supported language.
    pub fn parse_tag(raw: &str) -> Option<Self> {
        let s = raw.trim();
        if s.is_empty() {
            return None;
        }
        let primary = s
            .split(['_', '-', '.'])
            .next()
            .unwrap_or(s)
            .to_ascii_lowercase();
        match primary.as_str() {
            "en" => Some(Self::En),
            "zh" | "cn" => Some(Self::Zh),
            _ => None,
        }
    }

    pub fn from_env() -> Option<Self> {
        env::var("KIMINI_LANG")
            .ok()
            .as_deref()
            .and_then(Self::parse_tag)
    }

    pub fn from_system() -> Self {
        for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(v) = env::var(key)
                && let Some(lang) = Self::parse_tag(&v)
            {
                return lang;
            }
        }

        #[cfg(target_os = "macos")]
        if let Some(lang) = apple_locale() {
            return lang;
        }

        Self::En
    }

    /// Resolve active language: env → preference → system → English.
    pub fn resolve() -> Self {
        if let Some(lang) = Self::from_env() {
            return lang;
        }
        if let Some(lang) = load_preference() {
            return lang;
        }
        Self::from_system()
    }

    pub fn strings(self) -> Strings {
        match self {
            Self::En => Strings::en(),
            Self::Zh => Strings::zh(),
        }
    }
}

/// Host-owned UI copy (menus + Settings window). Web content is not translated.
#[derive(Debug, Clone, Copy)]
pub struct Strings {
    pub about: &'static str,
    pub settings: &'static str,
    pub settings_title: &'static str,
    pub edit: &'static str,
    pub navigate: &'static str,
    pub window: &'static str,
    pub reload: &'static str,
    pub back: &'static str,
    pub forward: &'static str,
    pub language: &'static str,
    pub language_hint: &'static str,
    pub lang_en: &'static str,
    pub lang_zh: &'static str,
    pub launch_probing: &'static str,
    pub launch_starting: &'static str,
    pub launch_waiting: &'static str,
    pub launch_no_kimi: &'static str,
    pub launch_invalid_url: &'static str,
    pub launch_manual: &'static str,
    pub launch_manual_hint: &'static str,
    pub launch_connect: &'static str,
}

impl Strings {
    pub const fn en() -> Self {
        Self {
            about: "About Kimini",
            settings: "Settings…",
            settings_title: "Kimini Settings",
            edit: "Edit",
            navigate: "Navigate",
            window: "Window",
            reload: "Reload",
            back: "Back",
            forward: "Forward",
            language: "Language",
            language_hint: "Language for menus and Settings. Web content follows Kimi Code Web.",
            lang_en: "English",
            lang_zh: "Chinese (Simplified)",
            launch_probing: "Looking for the local Kimi server…",
            launch_starting: "Starting the local Kimi server…",
            launch_waiting: "Still waiting for the Kimi server — try running `kimi` in your terminal.",
            launch_no_kimi: "Kimi CLI not found. Install it and this page will connect automatically:",
            launch_invalid_url: "Only local (loopback) addresses are allowed.",
            launch_manual: "Connect manually",
            launch_manual_hint: "Paste the URL printed by `kimi web`.",
            launch_connect: "Connect",
        }
    }

    pub const fn zh() -> Self {
        Self {
            about: "关于 Kimini",
            settings: "设置…",
            settings_title: "Kimini 设置",
            edit: "编辑",
            navigate: "导航",
            window: "窗口",
            reload: "重新加载",
            back: "后退",
            forward: "前进",
            language: "语言",
            language_hint: "用于菜单与设置窗口。网页内容仍由 Kimi Code Web 决定。",
            lang_en: "English",
            lang_zh: "简体中文",
            launch_probing: "正在查找本地 Kimi 服务…",
            launch_starting: "正在启动本地 Kimi 服务…",
            launch_waiting: "仍在等待 Kimi 服务就绪，可尝试在终端运行 `kimi`。",
            launch_no_kimi: "未找到 Kimi CLI。安装后本页会自动连接：",
            launch_invalid_url: "仅支持本机 (localhost) 地址。",
            launch_manual: "手动连接",
            launch_manual_hint: "粘贴 `kimi web` 输出的地址。",
            launch_connect: "连接",
        }
    }
}

fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = env::var_os("HOME")?;
        Some(
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Kimini"),
        )
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg).join("kimini"));
        }
        let home = env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".config").join("kimini"))
    }
}

fn preference_path() -> Option<PathBuf> {
    Some(config_dir()?.join("lang"))
}

pub fn load_preference() -> Option<Lang> {
    let path = preference_path()?;
    let raw = fs::read_to_string(path).ok()?;
    Lang::parse_tag(raw.trim())
}

pub fn save_preference(lang: Lang) -> std::io::Result<()> {
    let dir = config_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory"))?;
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("lang"), lang.code())
}

#[cfg(target_os = "macos")]
fn apple_locale() -> Option<Lang> {
    let output = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleLocale"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout);
    Lang::parse_tag(s.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_english_tags() {
        assert_eq!(Lang::parse_tag("en"), Some(Lang::En));
        assert_eq!(Lang::parse_tag("en_US"), Some(Lang::En));
        assert_eq!(Lang::parse_tag("en-US.UTF-8"), Some(Lang::En));
    }

    #[test]
    fn parse_chinese_tags() {
        assert_eq!(Lang::parse_tag("zh"), Some(Lang::Zh));
        assert_eq!(Lang::parse_tag("zh_CN"), Some(Lang::Zh));
        assert_eq!(Lang::parse_tag("zh-Hans"), Some(Lang::Zh));
    }

    #[test]
    fn parse_unknown_is_none() {
        assert_eq!(Lang::parse_tag(""), None);
        assert_eq!(Lang::parse_tag("ja"), None);
    }

    #[test]
    fn zh_table_has_cjk_en_table_does_not() {
        let en = Strings::en().reload;
        let zh = Strings::zh().reload;
        assert!(en.is_ascii());
        assert!(!zh.is_ascii());
    }
}
