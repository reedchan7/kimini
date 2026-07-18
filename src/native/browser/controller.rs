use gpui::{AppContext, Context, Window};

use super::{BrowserPane, normalize_address};
use crate::native::app::Shell;

impl Shell {
    pub(in crate::native) fn toggle_browser(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.browser.is_some() {
            self.close_browser(window, cx);
        } else {
            self.open_browser(window, cx);
        }
    }

    pub(in crate::native) fn navigate_browser(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input = self.browser_address.read(cx).value().to_string();
        let url = match normalize_address(&input) {
            Ok(url) => url,
            Err(error) => {
                self.browser_error = Some(error.into());
                cx.notify();
                return;
            }
        };
        let Some(browser) = self.browser.as_ref() else {
            self.open_browser(window, cx);
            return;
        };
        match browser.read(cx).load_url(&url) {
            Ok(()) => {
                self.browser_error = None;
                self.browser_address
                    .update(cx, |input, cx| input.set_value(&url, window, cx));
            }
            Err(error) => self.browser_error = Some(error),
        }
        cx.notify();
    }

    pub(in crate::native) fn browser_back(&mut self, cx: &mut Context<Self>) {
        if let Some(browser) = &self.browser {
            self.browser_error = browser.read(cx).back().err();
            cx.notify();
        }
    }

    pub(in crate::native) fn close_browser(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(browser) = self.browser.take() {
            browser.update(cx, |browser, _| browser.hide());
        }
        self.browser_error = None;
        self.composer
            .update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    pub(in crate::native) fn open_browser(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.browser_address.read(cx).value().to_string();
        let url = match normalize_address(&input) {
            Ok(url) => url,
            Err(error) => {
                self.browser_error = Some(error.into());
                cx.notify();
                return;
            }
        };
        match wry::WebViewBuilder::new()
            .with_url(&url)
            .build_as_child(window)
        {
            Ok(webview) => {
                self.browser = Some(cx.new(|cx| BrowserPane::new(webview, cx)));
                self.browser_error = None;
                self.browser_address
                    .update(cx, |input, cx| input.focus(window, cx));
            }
            Err(error) => self.browser_error = Some(error.to_string()),
        }
        cx.notify();
    }

    pub(in crate::native) fn open_browser_url(
        &mut self,
        url: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.browser_address
            .update(cx, |input, cx| input.set_value(url, window, cx));
        if let Some(browser) = self.browser.as_ref() {
            match browser.read(cx).load_url(url) {
                Ok(()) => self.browser_error = None,
                Err(error) => self.browser_error = Some(error),
            }
            self.browser_address
                .update(cx, |input, cx| input.focus(window, cx));
            cx.notify();
        } else {
            self.open_browser(window, cx);
        }
    }
}
