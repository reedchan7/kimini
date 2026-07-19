use std::borrow::Cow;

use gpui::{
    App, AppContext, AssetSource, Bounds, KeyBinding, Menu, MenuItem, OsAction, SharedString,
    SystemMenuType, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};
use gpui_component::{
    Root,
    input::{Copy, Cut, Paste, Redo, SelectAll, Undo},
};
use gpui_platform::application;

use crate::i18n::{Lang, Strings};

use super::app::{NativePreferences, Shell};
use super::theme;
use super::{
    About, ArchiveSession, CheckForUpdates, CloseSessionSearch, CloseWindow, CompactSession,
    ExportSession, FocusNext, FocusPrevious, FocusSessionSearch, ForkSession, Hide, HideOthers,
    Minimize, NewSession, OpenSettings, Quit, RenameSession, SessionSearchNext,
    SessionSearchPrevious, ShowAll, SteerPrompt, ToggleBrowser, ToggleFiles, ToggleSidebar,
    ToggleSkills, ToggleTasks, ToggleTerminal, UndoSession, Zoom,
};

const WINDOW_WIDTH: f32 = 1440.0;
const WINDOW_HEIGHT: f32 = 900.0;
pub(super) const APP_ICON_PATH: &str = "images/kimini-app-icon.png";
pub(super) const ATTACHMENT_ICON_PATH: &str = "icons/kimini-paperclip.svg";
pub(super) const SESSION_RENAME_ICON_PATH: &str = "icons/kimini-pencil.svg";
pub(super) const SESSION_FORK_ICON_PATH: &str = "icons/kimini-git-fork.svg";
pub(super) const SESSION_EXPORT_ICON_PATH: &str = "icons/kimini-download.svg";
pub(super) const SESSION_ARCHIVE_ICON_PATH: &str = "icons/kimini-archive.svg";
#[cfg(target_os = "macos")]
pub(super) const PRIMARY_MODIFIER_LABEL: &str = "⌘";
#[cfg(not(target_os = "macos"))]
pub(super) const PRIMARY_MODIFIER_LABEL: &str = "Ctrl";

fn app_menus(strings: &Strings) -> Vec<Menu> {
    let native = strings.native;
    let mut app_items = vec![
        MenuItem::action(strings.about, About),
        MenuItem::separator(),
        MenuItem::action(strings.settings, OpenSettings),
        MenuItem::action(strings.check_for_updates, CheckForUpdates),
    ];
    #[cfg(target_os = "macos")]
    app_items.extend([
        MenuItem::separator(),
        MenuItem::os_submenu(strings.services, SystemMenuType::Services),
        MenuItem::separator(),
        MenuItem::action(strings.hide, Hide),
        MenuItem::action(strings.hide_others, HideOthers),
        MenuItem::action(strings.show_all, ShowAll),
    ]);
    app_items.extend([MenuItem::separator(), MenuItem::action(strings.quit, Quit)]);

    vec![
        Menu::new("Kimini").items(app_items),
        Menu::new(strings.file).items([
            MenuItem::action(native.new_session, NewSession),
            MenuItem::action(native.search_sessions, FocusSessionSearch),
            MenuItem::separator(),
            MenuItem::action(strings.close_window, CloseWindow),
        ]),
        Menu::new(strings.edit).items([
            MenuItem::os_action(strings.undo_edit, Undo, OsAction::Undo),
            MenuItem::os_action(strings.redo, Redo, OsAction::Redo),
            MenuItem::separator(),
            MenuItem::os_action(strings.cut, Cut, OsAction::Cut),
            MenuItem::os_action(strings.copy, Copy, OsAction::Copy),
            MenuItem::os_action(strings.paste, Paste, OsAction::Paste),
            MenuItem::separator(),
            MenuItem::os_action(strings.select_all, SelectAll, OsAction::SelectAll),
        ]),
        Menu::new(strings.view).items([
            MenuItem::action(strings.toggle_sidebar, ToggleSidebar),
            MenuItem::separator(),
            MenuItem::action(native.tasks, ToggleTasks),
            MenuItem::action(native.files, ToggleFiles),
            MenuItem::action(native.skills, ToggleSkills),
            MenuItem::action(native.terminal, ToggleTerminal),
            MenuItem::action(native.browser, ToggleBrowser),
        ]),
        Menu::new(strings.session).items([
            MenuItem::action(native.rename_session, RenameSession),
            MenuItem::action(native.fork, ForkSession),
            MenuItem::action(native.compact, CompactSession),
            MenuItem::action(native.undo, UndoSession),
            MenuItem::separator(),
            MenuItem::action(native.export_session, ExportSession),
            MenuItem::action(native.archive, ArchiveSession),
        ]),
        Menu::new(strings.window).items([
            MenuItem::action(strings.minimize, Minimize),
            MenuItem::action(strings.zoom, Zoom),
        ]),
    ]
}

pub(super) fn install_app_menus(strings: &Strings, cx: &App) {
    cx.set_menus(app_menus(strings));
}

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
        if path == SESSION_RENAME_ICON_PATH {
            return Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/pencil.svg"
            ))));
        }
        if path == SESSION_FORK_ICON_PATH {
            return Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/git-fork.svg"
            ))));
        }
        if path == SESSION_EXPORT_ICON_PATH {
            return Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/download.svg"
            ))));
        }
        if path == SESSION_ARCHIVE_ICON_PATH {
            return Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/archive.svg"
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
        for icon in [
            SESSION_RENAME_ICON_PATH,
            SESSION_FORK_ICON_PATH,
            SESSION_EXPORT_ICON_PATH,
            SESSION_ARCHIVE_ICON_PATH,
        ] {
            if icon.starts_with(path) {
                assets.push(icon.into());
            }
        }
        Ok(assets)
    }
}

pub(super) fn run() {
    application().with_assets(KiminiAssets).run(|cx: &mut App| {
        gpui_component::init(cx);
        cx.on_action(|_: &Quit, cx| cx.quit());
        #[cfg(target_os = "macos")]
        {
            cx.on_action(|_: &Hide, cx| cx.hide());
            cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
            cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
        }
        let window_appearance = cx.window_appearance();
        theme::apply(&NativePreferences::load(), window_appearance, cx);
        cx.bind_keys([
            KeyBinding::new("tab", FocusNext, None),
            KeyBinding::new("shift-tab", FocusPrevious, None),
            KeyBinding::new("secondary-k", FocusSessionSearch, None),
            KeyBinding::new("secondary-n", NewSession, None),
            KeyBinding::new("secondary-,", OpenSettings, None),
            KeyBinding::new("secondary-q", Quit, None),
            KeyBinding::new("secondary-w", CloseWindow, None),
            KeyBinding::new("secondary-m", Minimize, None),
            KeyBinding::new("ctrl-s", SteerPrompt, None),
            KeyBinding::new("secondary-shift-e", ToggleFiles, None),
            KeyBinding::new("secondary-shift-k", ToggleSkills, None),
            KeyBinding::new("secondary-j", ToggleTerminal, None),
            KeyBinding::new("secondary-shift-t", ToggleTasks, None),
            KeyBinding::new("secondary-shift-b", ToggleBrowser, None),
            KeyBinding::new("secondary-b", ToggleSidebar, None),
            KeyBinding::new("down", SessionSearchNext, Some("SessionSearch")),
            KeyBinding::new("up", SessionSearchPrevious, Some("SessionSearch")),
            KeyBinding::new("escape", CloseSessionSearch, Some("SessionSearch")),
        ]);
        install_app_menus(&Lang::resolve().strings(), cx);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_menu_exposes_standard_app_and_edit_commands() {
        let menus = app_menus(&Strings::en());
        let names = menus
            .iter()
            .map(|menu| menu.name.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            ["Kimini", "File", "Edit", "View", "Session", "Window"]
        );
        assert!(menus[0].items.iter().any(|item| matches!(
            item,
            MenuItem::Action { name, .. } if name.as_ref() == "Quit Kimini"
        )));
        assert_eq!(
            menus[2]
                .items
                .iter()
                .filter(|item| matches!(
                    item,
                    MenuItem::Action {
                        os_action: Some(_),
                        ..
                    }
                ))
                .count(),
            6
        );
    }
}
