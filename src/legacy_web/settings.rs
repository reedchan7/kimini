use std::cell::RefCell;

use tao::event_loop::{EventLoopProxy, EventLoopWindowTarget};
use tao::window::{Window, WindowBuilder};
use wry::{WebView, WebViewBuilder};

use crate::i18n::Lang;

use super::app::UserEvent;
use super::pages::settings_html;

pub(super) struct SettingsWindow {
    pub(super) window: Window,
    webview: WebView,
}

pub(super) fn open_or_focus(
    event_loop: &EventLoopWindowTarget<UserEvent>,
    proxy: &EventLoopProxy<UserEvent>,
    lang: Lang,
    slot: &RefCell<Option<SettingsWindow>>,
) {
    if let Some(existing) = slot.borrow().as_ref() {
        existing.window.set_focus();
        return;
    }

    let strings = lang.strings();
    let window = WindowBuilder::new()
        .with_title(strings.settings_title)
        .with_inner_size(tao::dpi::LogicalSize::new(420.0, 280.0))
        .with_resizable(false)
        .build(event_loop)
        .expect("create settings window");

    let proxy = proxy.clone();
    let webview = WebViewBuilder::new()
        .with_html(settings_html(lang))
        .with_ipc_handler(move |request| {
            let _ = proxy.send_event(UserEvent::PrefsMessage(request.body().to_string()));
        })
        .build(&window)
        .expect("create settings webview");

    *slot.borrow_mut() = Some(SettingsWindow { window, webview });
}

pub(super) fn refresh(slot: &RefCell<Option<SettingsWindow>>, lang: Lang) {
    let guard = slot.borrow();
    let Some(settings) = guard.as_ref() else {
        return;
    };
    settings.window.set_title(lang.strings().settings_title);
    let _ = settings.webview.load_html(&settings_html(lang));
}
