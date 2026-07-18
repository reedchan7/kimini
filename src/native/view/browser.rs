use gpui::{Context, IntoElement, Role, div, prelude::*, px, rgb};
use gpui_component::input::{Input, InputContentType};

use super::super::app::Shell;
use super::super::theme::*;

impl Shell {
    pub(super) fn browser_surface(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let content = self.browser.clone();
        div()
            .id("browser-surface")
            .role(Role::Group)
            .aria_label("Human browser")
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
                    .border_color(rgb(BORDER))
                    .bg(rgb(SURFACE))
                    .child(
                        browser_button("Back", "browser-back")
                            .on_click(cx.listener(|this, _, _, cx| this.browser_back(cx))),
                    )
                    .child(
                        div().flex_1().min_w_0().child(
                            Input::new(&self.browser_address)
                                .role(Role::UrlInput)
                                .content_type(InputContentType::Url)
                                .h(px(32.0)),
                        ),
                    )
                    .child(
                        browser_button("Close", "browser-close")
                            .on_click(cx.listener(|this, _, _, cx| this.close_browser(cx))),
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
                        .text_xs()
                        .text_color(rgb(ERROR))
                        .child(error),
                )
            })
            .child(
                div()
                    .id("browser-content")
                    .role(Role::WebView)
                    .aria_label("Browser content")
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
        .border_color(rgb(BORDER))
        .px_3()
        .py_1()
        .text_sm()
        .hover(|item| item.bg(rgb(SURFACE_ACTIVE)))
        .child(label)
}
