use gpui::{Context, IntoElement, div, prelude::*};
use gpui_component::{StyledExt, text::TextView};

use super::super::app::Shell;
use super::super::theme::*;
use super::panel::panel_button;

impl Shell {
    pub(super) fn auth_summary_card(
        &self,
        summary: crate::protocol::AuthSummary,
    ) -> impl IntoElement {
        let status = if summary.ready {
            self.strings.native.auth_ready
        } else {
            self.strings.native.auth_required
        };
        div()
            .rounded_lg()
            .border_1()
            .border_color(theme_rgb(BORDER))
            .bg(theme_rgb(CANVAS))
            .p_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(font_px(13.0))
                            .font_semibold()
                            .child(self.strings.native.auth_status),
                    )
                    .child(
                        div()
                            .text_size(font_px(12.0))
                            .text_color(theme_rgb(if summary.ready { ACCENT } else { ERROR }))
                            .child(status),
                    ),
            )
            .child(
                div()
                    .mt_2()
                    .text_size(font_px(12.0))
                    .text_color(theme_rgb(TEXT_MUTED))
                    .child(format!(
                        "{}: {}",
                        self.strings.native.providers, summary.providers_count
                    )),
            )
            .when_some(summary.default_model, |card, model| {
                card.child(
                    div()
                        .mt_1()
                        .text_size(font_px(12.0))
                        .text_color(theme_rgb(TEXT_MUTED))
                        .child(format!("{}: {model}", self.strings.native.default_model)),
                )
            })
            .when_some(summary.managed_provider, |card, provider| {
                card.child(
                    div()
                        .mt_1()
                        .text_size(font_px(12.0))
                        .text_color(theme_rgb(TEXT_MUTED))
                        .child(format!("{} · {}", provider.name, provider.status)),
                )
            })
    }

    pub(super) fn sign_in_action(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .mt_3()
            .child(
                div()
                    .mb_2()
                    .text_size(font_px(13.0))
                    .text_color(theme_rgb(TEXT_MUTED))
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

    pub(super) fn oauth_handoff(&self, code: String, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .mt_3()
            .rounded_lg()
            .border_1()
            .border_color(theme_rgb(BORDER))
            .p_3()
            .child(
                div()
                    .text_size(font_px(13.0))
                    .font_semibold()
                    .child(self.strings.native.finish_sign_in),
            )
            .child(
                div()
                    .mt_2()
                    .text_size(font_px(12.0))
                    .text_color(theme_rgb(TEXT_MUTED))
                    .child(self.strings.native.device_code),
            )
            .child(
                div().mt_1().rounded_md().bg(theme_rgb(CANVAS)).p_2().child(
                    TextView::markdown("oauth-user-code", format!("\x60{code}\x60"))
                        .selectable(true)
                        .text_size(font_px(13.0)),
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
