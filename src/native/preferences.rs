use std::{fs, io, path::PathBuf};

use gpui::{Context, Window};

use crate::i18n::config_dir;

use super::app::{LoadState, NativePreferences, Shell};
use super::theme;

const PREFERENCES_FILE: &str = "native-preferences.json";

impl NativePreferences {
    pub(super) fn load() -> Self {
        preferences_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .map(Self::normalized)
            .unwrap_or_default()
    }

    pub(super) fn save(self) -> io::Result<()> {
        let dir = config_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no config directory"))?;
        fs::create_dir_all(&dir)?;
        let destination = dir.join(PREFERENCES_FILE);
        let temporary = dir.join(format!("{PREFERENCES_FILE}.tmp"));
        let bytes = serde_json::to_vec_pretty(&self.normalized()).map_err(io::Error::other)?;
        fs::write(&temporary, bytes)?;
        fs::rename(temporary, destination)
    }

    fn normalized(mut self) -> Self {
        self.font_size = self.font_size.clamp(12, 20);
        self
    }
}

impl Shell {
    pub(super) fn update_preferences(
        &mut self,
        update: impl FnOnce(&mut NativePreferences),
        cx: &mut Context<Self>,
    ) {
        update(&mut self.preferences);
        if let Err(error) = self.preferences.save() {
            self.state = LoadState::Failed(format!("Could not save Kimini settings: {error}"));
        }
        cx.notify();
    }

    pub(super) fn update_theme_preferences(
        &mut self,
        update: impl FnOnce(&mut NativePreferences),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        update(&mut self.preferences);
        theme::apply(&self.preferences, window.appearance(), cx);
        if let Err(error) = self.preferences.save() {
            self.state = LoadState::Failed(format!("Could not save Kimini settings: {error}"));
        }
        cx.notify();
    }
}

fn preferences_path() -> Option<PathBuf> {
    Some(config_dir()?.join(PREFERENCES_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_saved_preferences_keep_new_defaults() {
        let preferences: NativePreferences =
            serde_json::from_str(r#"{"appearance":"moon_dark","composer_permission":"auto"}"#)
                .unwrap();

        assert_eq!(
            preferences.appearance,
            super::super::app::AppearanceMode::MoonDark
        );
        assert_eq!(
            preferences.composer_permission,
            super::super::app::DefaultPermission::Auto
        );
        assert_eq!(
            preferences.font_size,
            NativePreferences::default().font_size
        );
        assert!(preferences.conversation_outline);
    }

    #[test]
    fn loaded_font_size_is_bounded_to_supported_range() {
        let preferences: NativePreferences = serde_json::from_str(r#"{"font_size":255}"#).unwrap();

        assert_eq!(preferences.normalized().font_size, 20);
    }

    #[test]
    fn local_preferences_exclude_daemon_and_unimplemented_settings() {
        let saved = serde_json::to_value(NativePreferences::default()).unwrap();

        for key in [
            "default_model",
            "default_permission",
            "default_thinking",
            "default_plan_mode",
            "merge_skills",
            "telemetry",
            "notify_complete",
            "notify_question",
            "notify_approval",
            "sound",
        ] {
            assert!(saved.get(key).is_none(), "unexpected local setting: {key}");
        }
    }
}
