use gpui::{Context, IntoElement, Role, Window, div, prelude::*, px, rgb};
use gpui_component::input::Input;

use super::super::app::Shell;
use super::super::theme::*;

impl Shell {
    pub(super) fn composer(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("message-composer")
            .role(Role::Group)
            .aria_label("Message composer")
            .w_full()
            .max_w(px(CONTENT_WIDTH))
            .p_4()
            .child(
                div()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(BORDER))
                    .bg(rgb(SURFACE))
                    .p_2()
                    .child(
                        Input::new(&self.composer)
                            .role(Role::MultilineTextInput)
                            .h(px(84.0)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_2()
                            .when(
                                self.model
                                    .active_session()
                                    .is_some_and(|session| session.busy),
                                |row| {
                                    row.child(
                                        div()
                                            .id("abort-session")
                                            .focusable()
                                            .tab_stop(true)
                                            .role(Role::Button)
                                            .aria_label("Stop current session")
                                            .cursor_pointer()
                                            .rounded_md()
                                            .border_1()
                                            .border_color(rgb(BORDER))
                                            .text_sm()
                                            .px_3()
                                            .py_1()
                                            .child("Stop")
                                            .on_click(cx.listener(|this, _, _, cx| this.abort(cx))),
                                    )
                                },
                            )
                            .child(
                                div()
                                    .id("send-prompt")
                                    .focusable()
                                    .tab_stop(true)
                                    .role(Role::Button)
                                    .aria_label("Send prompt")
                                    .cursor_pointer()
                                    .rounded_md()
                                    .bg(rgb(ACCENT))
                                    .text_color(rgb(SURFACE))
                                    .text_sm()
                                    .px_3()
                                    .py_1()
                                    .child("Send")
                                    .on_click(
                                        cx.listener(|this, _, window, cx| this.submit(window, cx)),
                                    ),
                            ),
                    ),
            )
    }
}
