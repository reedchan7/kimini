use crate::i18n::Strings;

pub(super) fn install(strings: &Strings) -> muda::Menu {
    use muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};

    let menu = Menu::new();
    let app = Submenu::with_id_and_items(
        "app",
        "Kimini Web",
        true,
        &[
            &PredefinedMenuItem::about(Some(strings.about), None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(
                "settings",
                strings.settings,
                true,
                "CmdOrCtrl+,".parse().ok(),
            ),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ],
    );
    let edit = Submenu::with_id_and_items(
        "edit",
        strings.edit,
        true,
        &[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::select_all(None),
        ],
    );
    let navigate = Submenu::with_id_and_items(
        "navigate",
        strings.navigate,
        true,
        &[
            &MenuItem::with_id("reload", strings.reload, true, "CmdOrCtrl+R".parse().ok()),
            &MenuItem::with_id("back", strings.back, true, "CmdOrCtrl+[".parse().ok()),
            &MenuItem::with_id("forward", strings.forward, true, "CmdOrCtrl+]".parse().ok()),
        ],
    );
    let window = Submenu::with_id_and_items(
        "window",
        strings.window,
        true,
        &[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::close_window(None),
        ],
    );

    for submenu in [app, edit, navigate, window] {
        menu.append(&submenu.expect("valid menu item"))
            .expect("append submenu");
    }
    menu.init_for_nsapp();
    menu
}
