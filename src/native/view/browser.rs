use gpui::{Context, IntoElement, Role, div, prelude::*, px};
use gpui_component::input::{Input, InputContentType};

use super::super::app::Shell;
use super::super::theme::*;
use super::accessible_input::accessible_input;

impl Shell {
    pub(super) fn browser_surface(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let content = self.browser.clone();
        div()
            .id("browser-surface")
            .role(Role::Group)
            .aria_label(self.strings.native.browser)
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(48.0))
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .border_b_1()
                    .border_color(theme_rgb(BORDER))
                    .bg(theme_rgb(SURFACE))
                    .child(
                        browser_button(self.strings.back, "browser-back")
                            .on_click(cx.listener(|this, _, _, cx| this.browser_back(cx))),
                    )
                    .child(
                        accessible_input(
                            "browser-address-input",
                            &self.browser_address,
                            Role::UrlInput,
                            self.strings.native.browser_address,
                            self.strings.native.browser_address,
                            Input::new(&self.browser_address).content_type(InputContentType::Url),
                            cx,
                        )
                        .flex_1()
                        .min_w_0()
                        .h(px(32.0)),
                    )
                    .child(
                        browser_button(self.strings.native.close_browser, "browser-close")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.close_browser(window, cx)),
                            ),
                    ),
            )
            .when_some(self.browser_error.clone(), |surface, error| {
                surface.child(
                    div()
                        .id("browser-error")
                        .role(Role::Status)
                        .aria_label(error.clone())
                        .flex_none()
                        .px_3()
                        .py_2()
                        .text_size(font_px(12.0))
                        .text_color(theme_rgb(ERROR))
                        .child(error),
                )
            })
            .child(
                div()
                    .id("browser-content")
                    .role(Role::WebView)
                    .aria_label(self.strings.native.browser_content)
                    .flex_1()
                    .min_h_0()
                    .when_some(content, |container, browser| container.child(browser)),
            )
    }
}

fn browser_button(label: &'static str, id: &'static str) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(label)
        .cursor_pointer()
        .rounded_md()
        .border_1()
        .border_color(theme_rgb(BORDER))
        .px_3()
        .py_1()
        .text_size(font_px(13.0))
        .hover(|item| item.bg(theme_rgb(SURFACE_ACTIVE)))
        .child(label)
}
