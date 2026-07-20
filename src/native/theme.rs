use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use gpui::{App, Pixels, Rgba, WindowAppearance, px};
use gpui_component::{Theme, ThemeMode};

use super::app::{AccentMode, AppearanceMode, NativePreferences};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ColorToken {
    Canvas,
    Sidebar,
    Surface,
    SurfaceSubtle,
    SurfaceActive,
    Border,
    BorderStrong,
    Text,
    TextSecondary,
    TextMuted,
    Accent,
    AccentSoft,
    Success,
    Warning,
    Assistant,
    Error,
    ErrorSoft,
    ErrorSoftActive,
}

pub(super) const CANVAS: ColorToken = ColorToken::Canvas;
pub(super) const SIDEBAR: ColorToken = ColorToken::Sidebar;
pub(super) const SURFACE: ColorToken = ColorToken::Surface;
pub(super) const SURFACE_SUBTLE: ColorToken = ColorToken::SurfaceSubtle;
pub(super) const SURFACE_ACTIVE: ColorToken = ColorToken::SurfaceActive;
pub(super) const BORDER: ColorToken = ColorToken::Border;
pub(super) const BORDER_STRONG: ColorToken = ColorToken::BorderStrong;
pub(super) const TEXT: ColorToken = ColorToken::Text;
pub(super) const TEXT_SECONDARY: ColorToken = ColorToken::TextSecondary;
pub(super) const TEXT_MUTED: ColorToken = ColorToken::TextMuted;
pub(super) const ACCENT: ColorToken = ColorToken::Accent;
pub(super) const ACCENT_SOFT: ColorToken = ColorToken::AccentSoft;
pub(super) const SUCCESS: ColorToken = ColorToken::Success;
pub(super) const WARNING: ColorToken = ColorToken::Warning;
pub(super) const ASSISTANT: ColorToken = ColorToken::Assistant;
pub(super) const ERROR: ColorToken = ColorToken::Error;
pub(super) const ERROR_SOFT: ColorToken = ColorToken::ErrorSoft;
pub(super) const ERROR_SOFT_ACTIVE: ColorToken = ColorToken::ErrorSoftActive;

static DARK_MODE: AtomicBool = AtomicBool::new(false);
static MONO_ACCENT: AtomicBool = AtomicBool::new(false);
static FONT_SIZE: AtomicU8 = AtomicU8::new(14);

pub(super) fn apply(
    preferences: &NativePreferences,
    window_appearance: WindowAppearance,
    cx: &mut App,
) {
    let dark = resolve_dark_mode(preferences, window_appearance);
    DARK_MODE.store(dark, Ordering::Relaxed);
    MONO_ACCENT.store(preferences.accent == AccentMode::Black, Ordering::Relaxed);
    FONT_SIZE.store(preferences.font_size.clamp(12, 20), Ordering::Relaxed);

    let mode = if dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    Theme::change(mode, None, cx);
    let component_theme = Theme::global_mut(cx);
    component_theme.font_size = px(f32::from(current_font_size()));
    component_theme.mono_font_size = px(f32::from(current_font_size().saturating_sub(1)));
    cx.refresh_windows();
}

fn resolve_dark_mode(preferences: &NativePreferences, window_appearance: WindowAppearance) -> bool {
    match preferences.appearance {
        AppearanceMode::MoonBright => false,
        AppearanceMode::MoonDark => true,
        AppearanceMode::System => matches!(
            window_appearance,
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        ),
    }
}

pub(super) fn is_dark() -> bool {
    DARK_MODE.load(Ordering::Relaxed)
}

pub(super) fn current_font_size() -> u8 {
    FONT_SIZE.load(Ordering::Relaxed)
}

pub(super) trait ThemeColorValue {
    fn resolved(self) -> u32;
}

impl ThemeColorValue for ColorToken {
    fn resolved(self) -> u32 {
        resolve_color(self)
    }
}

impl ThemeColorValue for u32 {
    fn resolved(self) -> u32 {
        self
    }
}

pub(super) fn theme_rgb(color: impl ThemeColorValue) -> Rgba {
    gpui::rgb(color.resolved())
}

/// Scale a design-token size against Settings → Font size (default base 14).
///
/// `base` is the size at the default 14px setting. Raising Settings to 16
/// adds +2 to every token so hierarchy stays proportional.
pub(super) fn font_px(base: f32) -> Pixels {
    let offset = f32::from(current_font_size()) - 14.0;
    px((base + offset).max(8.0))
}

/// Primary reading size: chat markdown, side-chat answers, empty-state body.
/// Matches Settings "Font size" (Codex / Kimi Web UI base of 14px).
pub(super) fn body_font_px() -> Pixels {
    font_px(14.0)
}

/// Default chrome labels: sidebar rows, toolbar title, settings rows.
pub(super) fn ui_font_px() -> Pixels {
    font_px(13.0)
}

/// Secondary chrome: workspace headings, control captions.
pub(super) fn caption_font_px() -> Pixels {
    font_px(12.0)
}

fn resolve_color(color: ColorToken) -> u32 {
    let dark = is_dark();
    let mono = MONO_ACCENT.load(Ordering::Relaxed);
    resolve_color_for(color, dark, mono)
}

fn resolve_color_for(color: ColorToken, dark: bool, mono: bool) -> u32 {
    if color == ACCENT {
        return match (dark, mono) {
            (false, false) => 0x1783ff,
            (false, true) => 0x171717,
            (true, false) => 0x58a6ff,
            (true, true) => 0xe8eaed,
        };
    }
    if color == ACCENT_SOFT {
        return match (dark, mono) {
            (false, false) => 0xe8f3ff,
            (false, true) => 0xf1f1f2,
            (true, false) => 0x1c2a3a,
            (true, true) => 0x21262d,
        };
    }
    match (dark, color) {
        (false, CANVAS | SURFACE) => 0xffffff,
        (false, SIDEBAR) => 0xfbfaf9,
        (false, SURFACE_SUBTLE | ASSISTANT) => 0xfafbfc,
        (false, SURFACE_ACTIVE) => 0xebebeb,
        (false, BORDER) => 0xe7eaee,
        (false, BORDER_STRONG) => 0xd4d9e0,
        (false, TEXT) => 0x1a1a1a,
        (false, TEXT_SECONDARY) => 0x6b7280,
        (false, TEXT_MUTED) => 0x9aa3af,
        (false, SUCCESS) => 0x249366,
        (false, WARNING) => 0xc28b00,
        (false, ERROR) => 0xb42318,
        (false, ERROR_SOFT) => 0xffeeee,
        (false, ERROR_SOFT_ACTIVE) => 0xffe3e3,
        (true, CANVAS) => 0x0d1117,
        (true, SIDEBAR) => 0x181817,
        (true, SURFACE) => 0x1c2128,
        (true, SURFACE_SUBTLE | ASSISTANT) => 0x161b22,
        (true, SURFACE_ACTIVE) => 0x282d33,
        (true, BORDER) => 0x2d333b,
        (true, BORDER_STRONG) => 0x3d444d,
        (true, TEXT) => 0xc9cdd4,
        (true, TEXT_SECONDARY) => 0x9aa0a8,
        (true, TEXT_MUTED) => 0x6b7280,
        (true, SUCCESS) => 0x3fb950,
        (true, WARNING) => 0xd29922,
        (true, ERROR) => 0xf85149,
        (true, ERROR_SOFT) => 0x382022,
        (true, ERROR_SOFT_ACTIVE) => 0x4a2424,
        (_, ACCENT | ACCENT_SOFT) => unreachable!("accent tokens return above"),
    }
}

pub(super) const SIDEBAR_WIDTH: f32 = 270.0;
pub(super) const TASK_PANEL_WIDTH: f32 = 340.0;
pub(super) const SIDE_CHAT_PANEL_WIDTH: f32 = 400.0;
pub(super) const TERMINAL_PANEL_WIDTH: f32 = 520.0;
pub(super) const FILE_PANEL_WIDTH: f32 = 440.0;
pub(super) const CONTENT_WIDTH: f32 = 760.0;
pub(super) const HEADER_HEIGHT: f32 = 48.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_palette_tracks_the_web_design_tokens() {
        assert_eq!(resolve_color_for(CANVAS, true, false), 0x0d1117);
        assert_eq!(resolve_color_for(SURFACE, true, false), 0x1c2128);
        assert_eq!(resolve_color_for(TEXT, true, false), 0xc9cdd4);
        assert_eq!(resolve_color_for(ACCENT, true, false), 0x58a6ff);
    }

    #[test]
    fn mono_accent_is_scheme_aware() {
        assert_eq!(resolve_color_for(ACCENT, false, true), 0x171717);
        assert_eq!(resolve_color_for(ACCENT, true, true), 0xe8eaed);
    }

    #[test]
    fn system_theme_uses_the_gpui_window_appearance() {
        let preferences = NativePreferences::default();

        assert!(!resolve_dark_mode(
            &preferences,
            gpui::WindowAppearance::Light
        ));
        assert!(resolve_dark_mode(
            &preferences,
            gpui::WindowAppearance::VibrantDark
        ));
    }
}
