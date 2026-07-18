use gpui::{Context, IntoElement, Role, div, prelude::*, px, rgb};
use gpui_component::{StyledExt, text::TextView};

use super::super::app::Shell;
use super::super::theme::*;
use super::panel::panel_button;

impl Shell {
    pub(super) fn auth_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let summary = self.auth.summary.clone();
        let can_logout = summary
            .as_ref()
            .and_then(|summary| summary.managed_provider.as_ref())
            .is_some();
        let pending_code = self.auth.pending().map(|(_, code, _)| code.to_owned());
        let has_pending = pending_code.is_some();
        div()
            .id("auth-panel")
            .role(Role::Group)
            .aria_label(self.strings.native.auth_panel)
            .w(px(TASK_PANEL_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SURFACE))
            .child(self.auth_panel_header(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .p_4()
                    .when_some(self.auth.error.clone(), |panel, error| {
                        panel.child(
                            div()
                                .id("auth-error")
                                .mb_3()
                                .role(Role::Status)
                                .rounded_md()
                                .border_1()
                                .border_color(rgb(ERROR))
                                .p_2()
                                .text_xs()
                                .text_color(rgb(ERROR))
                                .child(error),
                        )
                    })
                    .when_some(summary, |panel, summary| {
                        panel.child(self.auth_summary_card(summary))
                    })
                    .when_some(pending_code, |panel, code| {
                        panel.child(self.oauth_handoff(code, cx))
                    })
                    .when(!has_pending && can_logout, |panel| {
                        panel.child(
                            div().mt_3().child(
                                panel_button(self.strings.native.sign_out, "oauth-logout")
                                    .on_click(cx.listener(|this, _, _, cx| this.logout_oauth(cx))),
                            ),
                        )
                    })
                    .when(!has_pending && !self.auth.ready(), |panel| {
                        panel.child(self.sign_in_action(cx))
                    }),
            )
    }

    fn auth_panel_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(48.0))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .child(self.strings.native.authentication),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        panel_button(self.strings.native.refresh_auth, "refresh-auth")
                            .on_click(cx.listener(|this, _, _, cx| this.refresh_auth(cx))),
                    )
                    .child(
                        panel_button(self.strings.native.close_auth, "close-auth")
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_auth_panel(cx))),
                    ),
            )
    }

    fn auth_summary_card(&self, summary: crate::protocol::AuthSummary) -> impl IntoElement {
        let status = if summary.ready {
            self.strings.native.auth_ready
        } else {
            self.strings.native.auth_required
        };
        div()
            .rounded_lg()
            .border_1()
            .border_color(rgb(BORDER))
            .bg(rgb(CANVAS))
            .p_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .child(self.strings.native.auth_status),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(if summary.ready { ACCENT } else { ERROR }))
                            .child(status),
                    ),
            )
            .child(
                div()
                    .mt_2()
                    .text_xs()
                    .text_color(rgb(TEXT_MUTED))
                    .child(format!(
                        "{}: {}",
                        self.strings.native.providers, summary.providers_count
                    )),
            )
            .when_some(summary.default_model, |card, model| {
                card.child(
                    div()
                        .mt_1()
                        .text_xs()
                        .text_color(rgb(TEXT_MUTED))
                        .child(format!("{}: {model}", self.strings.native.default_model)),
                )
            })
            .when_some(summary.managed_provider, |card, provider| {
                card.child(
                    div()
                        .mt_1()
                        .text_xs()
                        .text_color(rgb(TEXT_MUTED))
                        .child(format!("{} · {}", provider.name, provider.status)),
                )
            })
    }

    fn sign_in_action(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .mt_3()
            .child(
                div()
                    .mb_2()
                    .text_sm()
                    .text_color(rgb(TEXT_MUTED))
                    .child(self.strings.native.sign_in_hint),
            )
            .child(
                panel_button(
                    if self.auth.loading {
                        self.strings.native.auth_working
                    } else {
                        self.strings.native.sign_in
                    },
                    "oauth-login",
                )
                .when(!self.auth.loading, |button| {
                    button.on_click(cx.listener(|this, _, _, cx| this.start_oauth_login(cx)))
                }),
            )
    }

    fn oauth_handoff(&self, code: String, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .mt_3()
            .rounded_lg()
            .border_1()
            .border_color(rgb(BORDER))
            .p_3()
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .child(self.strings.native.finish_sign_in),
            )
            .child(
                div()
                    .mt_2()
                    .text_xs()
                    .text_color(rgb(TEXT_MUTED))
                    .child(self.strings.native.device_code),
            )
            .child(
                div().mt_1().rounded_md().bg(rgb(CANVAS)).p_2().child(
                    TextView::markdown("oauth-user-code", format!("`{code}`"))
                        .selectable(true)
                        .text_sm(),
                ),
            )
            .child(
                div()
                    .mt_3()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        panel_button(self.strings.native.open_sign_in, "open-oauth-page").on_click(
                            cx.listener(|this, _, window, cx| this.open_oauth_page(window, cx)),
                        ),
                    )
                    .child(
                        panel_button(self.strings.native.cancel_sign_in, "cancel-oauth")
                            .on_click(cx.listener(|this, _, _, cx| this.cancel_oauth_login(cx))),
                    ),
            )
    }
}
