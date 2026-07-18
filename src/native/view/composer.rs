use gpui::{Context, ExternalPaths, IntoElement, Role, Window, div, prelude::*, px, rgb};
use gpui_component::{Icon, IconName, Sizable as _, input::Input};

use super::super::app::Shell;
use super::super::theme::*;
use super::accessible_input::accessible_input;

impl Shell {
    pub(super) fn composer(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let session_id = self
            .model
            .active_session()
            .map(|session| session.id.as_str())
            .unwrap_or_default();
        let uploads_pending = self.attachments.has_uploads(session_id);
        let session_busy = self
            .model
            .active_session()
            .is_some_and(|session| session.busy);

        div()
            .id("message-composer")
            .role(Role::Group)
            .aria_label(self.strings.native.message_composer)
            .w_full()
            .max_w(px(CONTENT_WIDTH))
            .px_3()
            .pt_2()
            .pb_4()
            .child(
                div()
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb(BORDER_STRONG))
                    .bg(rgb(SURFACE))
                    .shadow_sm()
                    .p_2()
                    .drag_over::<ExternalPaths>(|style, _, _, _| {
                        style.border_color(rgb(ACCENT)).bg(rgb(ACCENT_SOFT))
                    })
                    .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                        this.add_attachment_paths(paths.paths().to_vec(), cx)
                    }))
                    .child(self.attachment_strip(cx))
                    .children(self.slash_suggestions(cx))
                    .child(
                        accessible_input(
                            "composer-input",
                            &self.composer,
                            Role::MultilineTextInput,
                            self.strings.native.message_composer,
                            self.strings.native.ask_placeholder,
                            Input::new(&self.composer)
                                .appearance(false)
                                .bordered(false)
                                .focus_bordered(false),
                            cx,
                        )
                        .w_full(),
                    )
                    .child(
                        div()
                            .mt_2()
                            .flex()
                            .items_end()
                            .justify_between()
                            .gap_2()
                            .child(
                                div()
                                    .min_w_0()
                                    .flex_1()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(self.attach_button(cx))
                                    .child(self.runtime_controls(cx)),
                            )
                            .child(self.composer_actions(session_busy, uploads_pending, cx)),
                    ),
            )
    }

    fn attach_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("attach-files")
            .focusable()
            .tab_stop(true)
            .role(Role::Button)
            .aria_label(self.strings.native.attach_file)
            .cursor_pointer()
            .size(px(28.0))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .text_color(rgb(TEXT_SECONDARY))
            .hover(|item| item.bg(rgb(SURFACE_ACTIVE)).text_color(rgb(TEXT)))
            .on_click(cx.listener(|this, _, _, cx| this.choose_attachments(cx)))
            .child(Icon::new(IconName::Plus).small())
    }

    fn composer_actions(
        &self,
        session_busy: bool,
        uploads_pending: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1()
            .when(session_busy, |actions| {
                actions.child(
                    div()
                        .id("abort-session")
                        .focusable()
                        .tab_stop(true)
                        .role(Role::Button)
                        .aria_label(self.strings.native.stop)
                        .cursor_pointer()
                        .size(px(30.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .bg(rgb(ERROR_SOFT))
                        .text_color(rgb(ERROR))
                        .hover(|item| item.bg(rgb(ERROR_SOFT_ACTIVE)))
                        .on_click(cx.listener(|this, _, _, cx| this.abort(cx)))
                        .child(div().size(px(8.0)).rounded_sm().bg(rgb(ERROR))),
                )
            })
            .when(session_busy, |actions| {
                actions.child(
                    div()
                        .id("steer-prompt")
                        .focusable()
                        .tab_stop(true)
                        .role(Role::Button)
                        .aria_label(self.strings.native.steer)
                        .rounded_full()
                        .px_2()
                        .h(px(30.0))
                        .flex()
                        .items_center()
                        .text_size(px(11.0))
                        .text_color(rgb(TEXT_SECONDARY))
                        .hover(|item| item.bg(rgb(SURFACE_ACTIVE)))
                        .when(!uploads_pending, |button| {
                            button.cursor_pointer().on_click(
                                cx.listener(|this, _, window, cx| this.steer_prompt(window, cx)),
                            )
                        })
                        .child(self.strings.native.steer),
                )
            })
            .child(
                div()
                    .id("send-prompt")
                    .focusable()
                    .tab_stop(true)
                    .role(Role::Button)
                    .aria_label(if session_busy {
                        self.strings.native.queue
                    } else {
                        self.strings.native.send
                    })
                    .size(px(30.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .bg(rgb(if uploads_pending {
                        BORDER_STRONG
                    } else {
                        ACCENT
                    }))
                    .text_color(rgb(SURFACE))
                    .when(!uploads_pending, |button| {
                        button
                            .cursor_pointer()
                            .hover(|item| item.opacity(0.9))
                            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx)))
                    })
                    .child(Icon::new(IconName::ArrowUp).small()),
            )
    }
}
