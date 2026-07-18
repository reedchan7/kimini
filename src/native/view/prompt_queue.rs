use gpui::{AnyElement, Context, Role, div, prelude::*, rgb};
use gpui_component::StyledExt;

use super::super::app::Shell;
use super::super::theme::*;

impl Shell {
    pub(super) fn prompt_queue(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(session) = self.model.active_session() else {
            return div().into_any_element();
        };
        let prompts = self.prompt_queues.queued(&session.id);
        if prompts.is_empty() {
            return div().into_any_element();
        }

        div()
            .id("queued-prompts")
            .role(Role::Group)
            .aria_label(self.strings.native.queued_prompts)
            .mx_3()
            .mb_3()
            .rounded_lg()
            .border_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SURFACE))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .text_xs()
                            .font_semibold()
                            .text_color(rgb(TEXT_MUTED))
                            .child(format!(
                                "{} · {}",
                                self.strings.native.queued_prompts,
                                prompts.len()
                            )),
                    )
                    .child(
                        queue_button(self.strings.native.steer, "steer-queued-prompts")
                            .on_click(cx.listener(|this, _, _, cx| this.steer_queued_prompts(cx))),
                    ),
            )
            .children(prompts.into_iter().enumerate().map(|(index, prompt)| {
                let prompt_id = prompt.id.clone();
                let attachment_text = (prompt.attachment_count > 0).then(|| {
                    format!(
                        "{} {}",
                        prompt.attachment_count, self.strings.native.queued_attachments
                    )
                });
                let accessible_text = if prompt.text.is_empty() {
                    attachment_text.clone().unwrap_or_default()
                } else {
                    prompt.text.clone()
                };
                div()
                    .id(("queued-prompt", index))
                    .role(Role::Article)
                    .aria_label(format!(
                        "{}: {accessible_text}",
                        self.strings.native.queued_prompts
                    ))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .when(index > 0, |row| row.border_t_1().border_color(rgb(BORDER)))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_sm()
                            .line_clamp(1)
                            .child(accessible_text),
                    )
                    .children(
                        attachment_text
                            .map(|text| div().text_xs().text_color(rgb(TEXT_MUTED)).child(text)),
                    )
                    .child(
                        queue_button(
                            self.strings.native.remove_from_queue,
                            ("remove-queued-prompt", index),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.remove_queued_prompt(prompt_id.clone(), cx)
                        })),
                    )
            }))
            .into_any_element()
    }
}

fn queue_button(label: &'static str, id: impl Into<gpui::ElementId>) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(label)
        .cursor_pointer()
        .rounded_md()
        .px_2()
        .py_1()
        .text_xs()
        .hover(|button| button.bg(rgb(SURFACE_ACTIVE)))
        .child(label)
}
