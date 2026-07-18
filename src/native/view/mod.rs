mod browser;
mod composer;
mod conversation;
mod interaction;
mod sidebar;

use gpui::{Context, IntoElement, Render, Role, Window, div, prelude::*, px, rgb};
use gpui_component::StyledExt;

use super::app::Shell;
use super::theme::*;
use super::{FocusNext, FocusPrevious};

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("kimini-root")
            .role(Role::Application)
            .aria_label("Kimini")
            .on_action(cx.listener(|_, _: &FocusNext, window, cx| window.focus_next(cx)))
            .on_action(cx.listener(|_, _: &FocusPrevious, window, cx| window.focus_prev(cx)))
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(CANVAS))
            .text_color(rgb(TEXT))
            .child(self.toolbar(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.sidebar(cx))
                    .child(if self.browser.is_some() {
                        self.browser_surface(cx).into_any_element()
                    } else {
                        div()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .child(self.conversation(cx))
                            .child(self.composer(window, cx))
                            .into_any_element()
                    }),
            )
    }
}

impl Shell {
    fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(48.0))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .id("app-title")
                    .role(Role::Heading)
                    .aria_level(1)
                    .aria_label("Kimini")
                    .text_sm()
                    .font_semibold()
                    .child("Kimini"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .id("browser-toggle")
                            .focusable()
                            .tab_stop(true)
                            .role(Role::Button)
                            .aria_label(if self.browser.is_some() {
                                "Close browser"
                            } else {
                                "Open browser"
                            })
                            .cursor_pointer()
                            .rounded_md()
                            .px_2()
                            .py_1()
                            .text_xs()
                            .hover(|item| item.bg(rgb(SURFACE_ACTIVE)))
                            .on_click(
                                cx.listener(|this, _, window, cx| this.toggle_browser(window, cx)),
                            )
                            .child(if self.browser.is_some() {
                                "Close Browser"
                            } else {
                                "Browser"
                            }),
                    )
                    .child(
                        div()
                            .id("connection-status")
                            .role(Role::Status)
                            .aria_label(self.status_text())
                            .text_xs()
                            .text_color(rgb(TEXT_MUTED))
                            .child(self.status_text()),
                    ),
            )
    }

    fn status_text(&self) -> String {
        use super::app::LoadState;
        match &self.state {
            LoadState::Connecting => "Connecting to Kimi Code…".into(),
            LoadState::Ready => "Connected".into(),
            LoadState::Working(message) | LoadState::Failed(message) => message.clone(),
        }
    }
}
