use gpui::{App, AppContext, Bounds, KeyBinding, WindowBounds, WindowOptions, px, size};
use gpui_component::Root;
use gpui_platform::application;

use super::app::Shell;
use super::{
    FocusNext, FocusPrevious, SteerPrompt, ToggleFiles, ToggleSkills, ToggleTasks, ToggleTerminal,
};

const WINDOW_WIDTH: f32 = 1440.0;
const WINDOW_HEIGHT: f32 = 900.0;
pub(super) fn run() {
    application()
        .with_assets(gpui_component_assets::Assets)
        .run(|cx: &mut App| {
            gpui_component::init(cx);
            cx.bind_keys([
                KeyBinding::new("tab", FocusNext, None),
                KeyBinding::new("shift-tab", FocusPrevious, None),
                KeyBinding::new("ctrl-s", SteerPrompt, None),
                KeyBinding::new("cmd-shift-e", ToggleFiles, None),
                KeyBinding::new("cmd-shift-k", ToggleSkills, None),
                KeyBinding::new("cmd-j", ToggleTerminal, None),
                KeyBinding::new("cmd-shift-t", ToggleTasks, None),
            ]);
            let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
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
