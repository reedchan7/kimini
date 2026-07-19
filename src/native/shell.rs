use std::borrow::Cow;

use gpui::{
    App, AppContext, AssetSource, Bounds, KeyBinding, SharedString, TitlebarOptions, WindowBounds,
    WindowOptions, px, size,
};
use gpui_component::Root;
use gpui_platform::application;

use super::app::{NativePreferences, Shell};
use super::theme;
use super::{
    CloseSessionSearch, FocusNext, FocusPrevious, FocusSessionSearch, NewSession,
    SessionSearchNext, SessionSearchPrevious, SteerPrompt, ToggleBrowser, ToggleFiles,
    ToggleSidebar, ToggleSkills, ToggleTasks, ToggleTerminal,
};

const WINDOW_WIDTH: f32 = 1440.0;
const WINDOW_HEIGHT: f32 = 900.0;
pub(super) const APP_ICON_PATH: &str = "images/kimini-app-icon.png";
pub(super) const ATTACHMENT_ICON_PATH: &str = "icons/kimini-paperclip.svg";

struct KiminiAssets;

impl AssetSource for KiminiAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        if path == APP_ICON_PATH {
            return Ok(Some(Cow::Borrowed(include_bytes!(
                "../../docs/brand/exports/app-icon-256.png"
            ))));
        }
        if path == ATTACHMENT_ICON_PATH {
            return Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/paperclip.svg"
            ))));
        }
        gpui_component_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let mut assets = gpui_component_assets::Assets.list(path)?;
        if APP_ICON_PATH.starts_with(path) {
            assets.push(APP_ICON_PATH.into());
        }
        if ATTACHMENT_ICON_PATH.starts_with(path) {
            assets.push(ATTACHMENT_ICON_PATH.into());
        }
        Ok(assets)
    }
}

pub(super) fn run() {
    application().with_assets(KiminiAssets).run(|cx: &mut App| {
        gpui_component::init(cx);
        let window_appearance = cx.window_appearance();
        theme::apply(&NativePreferences::load(), window_appearance, cx);
        cx.bind_keys([
            KeyBinding::new("tab", FocusNext, None),
            KeyBinding::new("shift-tab", FocusPrevious, None),
            KeyBinding::new("cmd-k", FocusSessionSearch, None),
            KeyBinding::new("cmd-n", NewSession, None),
            KeyBinding::new("ctrl-s", SteerPrompt, None),
            KeyBinding::new("cmd-shift-e", ToggleFiles, None),
            KeyBinding::new("cmd-shift-k", ToggleSkills, None),
            KeyBinding::new("cmd-j", ToggleTerminal, None),
            KeyBinding::new("cmd-shift-t", ToggleTasks, None),
            KeyBinding::new("cmd-shift-b", ToggleBrowser, None),
            KeyBinding::new("cmd-b", ToggleSidebar, None),
            KeyBinding::new("down", SessionSearchNext, Some("SessionSearch")),
            KeyBinding::new("up", SessionSearchPrevious, Some("SessionSearch")),
            KeyBinding::new("escape", CloseSessionSearch, Some("SessionSearch")),
        ]);
        let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Kimini".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let shell = cx.new(|cx| Shell::new(window, cx));
                cx.new(|cx| Root::new(shell, window, cx))
            },
        )
        .expect("open native Kimini window");
        cx.activate(true);
    });
}
