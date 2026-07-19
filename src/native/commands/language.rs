use gpui::{Context, Window};

use crate::i18n::{Lang, save_preference};

use super::super::app::Shell;
use super::super::shell::install_app_menus;

impl Shell {
    pub(in crate::native) fn toggle_language(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.lang = match self.lang {
            Lang::En => Lang::Zh,
            Lang::Zh => Lang::En,
        };
        self.strings = self.lang.strings();
        install_app_menus(&self.strings, cx);
        let strings = self.strings.native;
        self.composer.update(cx, |input, cx| {
            input.set_placeholder(strings.ask_placeholder, window, cx)
        });
        self.session_search.update(cx, |input, cx| {
            input.set_placeholder(strings.search_sessions, window, cx)
        });
        self.rename_editor.update(cx, |input, cx| {
            input.set_placeholder(strings.rename_session, window, cx)
        });
        self.browser_address.update(cx, |input, cx| {
            input.set_placeholder(strings.browser_address, window, cx)
        });
        let _ = save_preference(self.lang);
        cx.notify();
    }
}
