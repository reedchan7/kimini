use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::Duration;

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
    // --- Web design-token additions (stage 0). Pure additive: existing
    // surfaces keep their values; these fill gaps the reference CSS exposes. ---
    /// `--color-surface-raised`: pure white (light) / #1c2128 (dark). Used by
    /// elevated cards, popovers, code-block headers.
    SurfaceRaised,
    /// `--color-surface-sunken`: #f3f5f8 (light) / #0d1117 (dark). Used by
    /// inline code, code-block bodies, secondary inset surfaces.
    SurfaceSunken,
    /// `--color-text-faint`: #9aa3af (light) / #6b7280 (dark). One step below
    /// `TextMuted` for timestamps, chevrons, separators.
    TextFaint,
    /// `--color-accent-hover`: #0f6fe0 (light) / #79b8ff (dark).
    AccentHover,
    /// `--color-accent-bd`: #cfe6ff (light) / rgba(88,166,255,.28) (dark, 28%).
    /// Used for accent-tinted borders (user bubble, accent chip, focused card).
    AccentBorder,
    /// `--color-selected`: 8% black (light) / 8% white (dark). Selection wash
    /// for sidebar rows, list items.
    Selected,
    /// `--color-hover`: 5% black (light) / 5% white (dark). Hover wash for
    /// sidebar rows, buttons, list items.
    Hover,
    /// `--color-done`: #8250df (light) / #a371f7 (dark). PR-merged, completed
    /// non-success states.
    Done,
    /// `--color-text-on-accent`: foreground on top of `Accent` fills.
    TextOnAccent,
    /// `--color-success-soft`: #e7f6ee (light) / 14% green (dark).
    SuccessSoft,
    /// `--color-success-bd`: #bfe3cc (light) / 28% green (dark).
    SuccessBorder,
    /// `--color-warning-soft`: #fbf1e0 (light) / 14% amber (dark).
    WarningSoft,
    /// `--color-warning-bd`: #f0d9b8 (light) / 28% amber (dark).
    WarningBorder,
    /// `--color-danger-bd`: #f0cccc (light) / 28% red (dark). Pairs with the
    /// existing `ErrorSoft` to complete the danger trio.
    DangerBorder,
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
pub(super) const SURFACE_RAISED: ColorToken = ColorToken::SurfaceRaised;
pub(super) const SURFACE_SUNKEN: ColorToken = ColorToken::SurfaceSunken;
pub(super) const TEXT_FAINT: ColorToken = ColorToken::TextFaint;
pub(super) const ACCENT_HOVER: ColorToken = ColorToken::AccentHover;
pub(super) const ACCENT_BORDER: ColorToken = ColorToken::AccentBorder;
pub(super) const SELECTED: ColorToken = ColorToken::Selected;
pub(super) const HOVER: ColorToken = ColorToken::Hover;
pub(super) const DONE: ColorToken = ColorToken::Done;
pub(super) const TEXT_ON_ACCENT: ColorToken = ColorToken::TextOnAccent;
pub(super) const SUCCESS_SOFT: ColorToken = ColorToken::SuccessSoft;
pub(super) const SUCCESS_BORDER: ColorToken = ColorToken::SuccessBorder;
pub(super) const WARNING_SOFT: ColorToken = ColorToken::WarningSoft;
pub(super) const WARNING_BORDER: ColorToken = ColorToken::WarningBorder;
pub(super) const DANGER_BORDER: ColorToken = ColorToken::DangerBorder;

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

/// Resolve a token to a full `0xRRGGBBAA` value (alpha in the low byte).
///
/// Use for tokens whose reference values carry alpha — `AccentBorder`,
/// `Selected`, `Hover`, and the dark-mode soft/-bd pairs. For opaque tokens
/// `theme_rgb` remains the right call.
pub(super) fn theme_rgba(color: ColorToken) -> Rgba {
    gpui::rgba(resolve_color_with_alpha(color))
}

/// `rgba(r, g, b, alpha)` where alpha is 0..=255. Packs to `0xRRGGBBAA`.
const fn rgba_u32(rgb: u32, alpha: u8) -> u32 {
    (rgb << 8) | (alpha as u32)
}

fn resolve_color_with_alpha(color: ColorToken) -> u32 {
    let dark = is_dark();
    let mono = MONO_ACCENT.load(Ordering::Relaxed);
    match color {
        ColorToken::AccentBorder => match (dark, mono) {
            (false, _) => rgba_u32(0xcfe6ff, 0xff),
            (true, false) => rgba_u32(0x58a6ff, 0x47), // 28%
            (true, true) => rgba_u32(0x3d444d, 0xff),
        },
        ColorToken::Selected => {
            let alpha: u8 = 0x14; // 8% — #00000014 (light) / #ffffff14 (dark)
            if dark {
                rgba_u32(0xffffff, alpha)
            } else {
                rgba_u32(0x000000, alpha)
            }
        }
        ColorToken::Hover => {
            let alpha: u8 = 0x0d; // 5% — #0000000d (light) / #ffffff0d (dark)
            if dark {
                rgba_u32(0xffffff, alpha)
            } else {
                rgba_u32(0x000000, alpha)
            }
        }
        ColorToken::SuccessSoft => match dark {
            false => rgba_u32(0xe7f6ee, 0xff),
            true => rgba_u32(0x3fb950, 0x24), // 14%
        },
        ColorToken::SuccessBorder => match dark {
            false => rgba_u32(0xbfe3cc, 0xff),
            true => rgba_u32(0x3fb950, 0x47), // 28%
        },
        ColorToken::WarningSoft => match dark {
            false => rgba_u32(0xfbf1e0, 0xff),
            true => rgba_u32(0xd29922, 0x24),
        },
        ColorToken::WarningBorder => match dark {
            false => rgba_u32(0xf0d9b8, 0xff),
            true => rgba_u32(0xd29922, 0x47),
        },
        ColorToken::DangerBorder => match dark {
            false => rgba_u32(0xf0cccc, 0xff),
            true => rgba_u32(0xf85149, 0x47),
        },
        _ => {
            // Fallback: opaque tokens use the 0RGB path; lift alpha to 0xff.
            resolve_color_for(color, dark, mono) << 8 | 0xff
        }
    }
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
        (false, CANVAS | SURFACE | SURFACE_RAISED) => 0xffffff,
        (false, SIDEBAR) => 0xfbfaf9,
        (false, SURFACE_SUBTLE | ASSISTANT) => 0xfafbfc,
        (false, SURFACE_ACTIVE) => 0xebebeb,
        (false, SURFACE_SUNKEN) => 0xf3f5f8,
        (false, BORDER) => 0xe7eaee,
        (false, BORDER_STRONG) => 0xd4d9e0,
        (false, TEXT) => 0x1a1a1a,
        (false, TEXT_SECONDARY) => 0x6b7280,
        (false, TEXT_MUTED) => 0x9aa3af,
        (false, TEXT_FAINT) => 0x9aa3af,
        (false, ACCENT_HOVER) => 0x0f6fe0,
        (false, DONE) => 0x8250df,
        (false, TEXT_ON_ACCENT) => 0xffffff,
        (false, SUCCESS) => 0x249366,
        (false, WARNING) => 0xc28b00,
        (false, ERROR) => 0xb42318,
        (false, ERROR_SOFT) => 0xffeeee,
        (false, ERROR_SOFT_ACTIVE) => 0xffe3e3,
        (true, CANVAS) => 0x0d1117,
        (true, SIDEBAR) => 0x181817,
        (true, SURFACE | SURFACE_RAISED) => 0x1c2128,
        (true, SURFACE_SUBTLE | ASSISTANT) => 0x161b22,
        (true, SURFACE_ACTIVE) => 0x282d33,
        (true, SURFACE_SUNKEN) => 0x0d1117,
        (true, BORDER) => 0x2d333b,
        (true, BORDER_STRONG) => 0x3d444d,
        (true, TEXT) => 0xc9cdd4,
        (true, TEXT_SECONDARY) => 0x9aa0a8,
        (true, TEXT_MUTED) => 0x6b7280,
        (true, TEXT_FAINT) => 0x6b7280,
        (true, ACCENT_HOVER) => 0x79b8ff,
        (true, DONE) => 0xa371f7,
        (true, TEXT_ON_ACCENT) => 0xffffff,
        (true, SUCCESS) => 0x3fb950,
        (true, WARNING) => 0xd29922,
        (true, ERROR) => 0xf85149,
        (true, ERROR_SOFT) => 0x382022,
        (true, ERROR_SOFT_ACTIVE) => 0x4a2424,
        (_, ACCENT | ACCENT_SOFT) => unreachable!("accent tokens return above"),
        (
            _,
            ColorToken::AccentBorder
            | ColorToken::Selected
            | ColorToken::Hover
            | ColorToken::SuccessSoft
            | ColorToken::SuccessBorder
            | ColorToken::WarningSoft
            | ColorToken::WarningBorder
            | ColorToken::DangerBorder,
        ) => unreachable!(
            "alpha tokens return from resolve_color_with_alpha; use theme_rgba"
        ),
    }
}

pub(super) const SIDEBAR_WIDTH: f32 = 270.0;
pub(super) const TASK_PANEL_WIDTH: f32 = 340.0;
pub(super) const SIDE_CHAT_PANEL_WIDTH: f32 = 400.0;
pub(super) const TERMINAL_PANEL_WIDTH: f32 = 520.0;
pub(super) const FILE_PANEL_WIDTH: f32 = 440.0;
pub(super) const CONTENT_WIDTH: f32 = 760.0;
pub(super) const HEADER_HEIGHT: f32 = 48.0;

// --- Reference: Kimi Code Web `--text-xs/sm/base/lg/xl/2xl` ladder. ---
// Stage 0 introduces these named helpers so later stages can swap off the
// ad-hoc `font_px(12.0)` / `ui_font_px()` calls without changing behaviour
// today. `body_font_px` / `ui_font_px` / `caption_font_px` stay as-is.
pub(super) const TEXT_XS_BASE: f32 = 12.0;
pub(super) const TEXT_SM_BASE: f32 = 13.0;
pub(super) const TEXT_BASE_BASE: f32 = 14.0;
pub(super) const TEXT_LG_BASE: f32 = 16.0;
pub(super) const TEXT_XL_BASE: f32 = 18.0;
pub(super) const TEXT_2XL_BASE: f32 = 22.0;

/// `--text-xs` (12px). Captions, kbd chips, code-header labels.
pub(super) fn text_xs_font_px() -> Pixels {
    font_px(TEXT_XS_BASE)
}
/// `--text-sm` (13px). Secondary chrome, sidebar rows, toolbar sub-line.
pub(super) fn text_sm_font_px() -> Pixels {
    font_px(TEXT_SM_BASE)
}
/// `--text-base` (14px). Primary chrome.
pub(super) fn text_base_font_px() -> Pixels {
    font_px(TEXT_BASE_BASE)
}
/// `--text-lg` (16px). Assistant body on small viewports, user-bubble body
/// minimum (Web clamps to `max(16px, ui-font-size-xl)`).
pub(super) fn text_lg_font_px() -> Pixels {
    font_px(TEXT_LG_BASE)
}
/// `--text-xl` (18px). Empty-state headings.
pub(super) fn text_xl_font_px() -> Pixels {
    font_px(TEXT_XL_BASE)
}

// --- Reference: chat spacing tokens (`--chat-turn/block/section-gap`). ---
/// `--chat-turn-gap: 16px` — gap between user-turn and assistant-turn roots.
pub(super) const CHAT_TURN_GAP: f32 = 16.0;
/// `--chat-block-gap: 10px` — gap between thinking / tool-group / msg inside
/// one assistant turn.
pub(super) const CHAT_BLOCK_GAP: f32 = 10.0;
/// `--chat-section-gap: 18px` — gap between major conversation sections.
pub(super) const CHAT_SECTION_GAP: f32 = 18.0;
/// `.chat` outer padding (16px top / 14px inline / 20px bottom).
pub(super) const CHAT_PADDING_TOP: f32 = 16.0;
pub(super) const CHAT_PADDING_BOTTOM: f32 = 20.0;

// --- Reference: Web transition durations (`--duration-fast/base/slow`). ---
pub(super) const DURATION_FAST: Duration = Duration::from_millis(120);
pub(super) const DURATION_BASE: Duration = Duration::from_millis(160);
pub(super) const DURATION_SLOW: Duration = Duration::from_millis(260);

/// Standard 60fps UI coalesce tick for streaming / progressive updates.
/// Mirrors `gpui-component`'s `AutoScroll` cadence.
pub(super) const FRAME_TICK_60FPS: Duration = Duration::from_millis(16);

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

    #[test]
    fn additive_tokens_match_reference_light_palette() {
        // Light `--color-*` from the reference CSS.
        assert_eq!(resolve_color_for(SURFACE_RAISED, false, false), 0xffffff);
        assert_eq!(resolve_color_for(SURFACE_SUNKEN, false, false), 0xf3f5f8);
        assert_eq!(resolve_color_for(TEXT_FAINT, false, false), 0x9aa3af);
        assert_eq!(resolve_color_for(ACCENT_HOVER, false, false), 0x0f6fe0);
        assert_eq!(resolve_color_for(DONE, false, false), 0x8250df);
        assert_eq!(resolve_color_for(TEXT_ON_ACCENT, false, false), 0xffffff);
    }

    #[test]
    fn additive_tokens_match_reference_dark_palette() {
        // Dark `--color-*` from the reference CSS.
        assert_eq!(resolve_color_for(SURFACE_RAISED, true, false), 0x1c2128);
        assert_eq!(resolve_color_for(SURFACE_SUNKEN, true, false), 0x0d1117);
        assert_eq!(resolve_color_for(TEXT_FAINT, true, false), 0x6b7280);
        assert_eq!(resolve_color_for(ACCENT_HOVER, true, false), 0x79b8ff);
        assert_eq!(resolve_color_for(DONE, true, false), 0xa371f7);
    }

    #[test]
    fn alpha_tokens_pack_reference_alpha_into_low_byte() {
        // Light mode accent-bd is opaque #cfe6ff.
        assert_eq!(
            resolve_color_with_alpha(ColorToken::AccentBorder),
            rgba_u32(0xcfe6ff, 0xff)
        );
        // Dark mode accent-bd is rgba(88,166,255,.28) → 0x47.
        DARK_MODE.store(true, Ordering::Relaxed);
        MONO_ACCENT.store(false, Ordering::Relaxed);
        assert_eq!(
            resolve_color_with_alpha(ColorToken::AccentBorder),
            rgba_u32(0x58a6ff, 0x47)
        );

        // Selected / Hover are 8% / 5% overlays of the base bg.
        assert_eq!(
            resolve_color_with_alpha(ColorToken::Selected),
            rgba_u32(0xffffff, 0x14)
        );
        assert_eq!(
            resolve_color_with_alpha(ColorToken::Hover),
            rgba_u32(0xffffff, 0x0d)
        );

        // Restore light mode.
        DARK_MODE.store(false, Ordering::Relaxed);
        assert_eq!(
            resolve_color_with_alpha(ColorToken::Selected),
            rgba_u32(0x000000, 0x14)
        );
        assert_eq!(
            resolve_color_with_alpha(ColorToken::Hover),
            rgba_u32(0x000000, 0x0d)
        );
    }

    #[test]
    fn text_ladder_and_chat_spacing_pin_reference_values() {
        assert_eq!(text_xs_font_px(), font_px(12.0));
        assert_eq!(text_sm_font_px(), font_px(13.0));
        assert_eq!(text_base_font_px(), font_px(14.0));
        assert_eq!(text_lg_font_px(), font_px(16.0));
        assert_eq!(text_xl_font_px(), font_px(18.0));
        assert_eq!(CHAT_TURN_GAP, 16.0);
        assert_eq!(CHAT_BLOCK_GAP, 10.0);
        assert_eq!(CHAT_SECTION_GAP, 18.0);
    }
}
