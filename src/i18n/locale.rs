use std::{env, fs, path::PathBuf};

use super::Lang;

#[cfg(test)]
use super::Strings;

impl Lang {
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh",
        }
    }

    /// Parse a BCP-47 / locale-ish tag into a supported language.
    pub fn parse_tag(raw: &str) -> Option<Self> {
        let value = raw.trim();
        if value.is_empty() {
            return None;
        }
        let primary = value
            .split(['_', '-', '.'])
            .next()
            .unwrap_or(value)
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
            if let Ok(value) = env::var(key)
                && let Some(lang) = Self::parse_tag(&value)
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
}

pub(crate) fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    const DIRECTORY_NAME: &str = "kimini";
    #[cfg(not(target_os = "linux"))]
    const DIRECTORY_NAME: &str = "Kimini";

    Some(dirs::config_dir()?.join(DIRECTORY_NAME))
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
    let value = String::from_utf8_lossy(&output.stdout);
    Lang::parse_tag(value.trim())
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
    fn language_tables_keep_their_expected_scripts() {
        let en = Strings::en().reload;
        let zh = Strings::zh().reload;
        assert!(en.is_ascii());
        assert!(!zh.is_ascii());
    }
}
