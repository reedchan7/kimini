use gpui::{Context, IntoElement, Role, div, prelude::*, rgb};

use super::super::app::{LoadState, Shell};
use super::super::theme::*;
use super::panel::panel_button;

impl Shell {
    pub(super) fn recovery_banner(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let LoadState::Failed(error) = &self.state else {
            return None;
        };
        Some(
            div()
                .id("recovery-banner")
                .role(Role::Alert)
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .border_b_1()
                .border_color(rgb(ERROR))
                .bg(rgb(0xffeeea))
                .px_4()
                .py_2()
                .child(
                    div()
                        .min_w_0()
                        .text_sm()
                        .text_color(rgb(ERROR))
                        .line_clamp(2)
                        .child(error.clone()),
                )
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            panel_button(self.strings.native.retry_connection, "retry-connection")
                                .on_click(cx.listener(|this, _, _, cx| this.reconnect(cx))),
                        )
                        .when(self.connection.is_some(), |actions| {
                            actions.child(
                                panel_button(
                                    self.strings.native.open_web_fallback,
                                    "open-web-fallback",
                                )
                                .on_click(cx.listener(|this, _, _, cx| this.open_web_fallback(cx))),
                            )
                        }),
                ),
        )
    }
}
