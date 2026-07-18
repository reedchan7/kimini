use gpui::{AnyElement, Context, IntoElement, Role, div, prelude::*, px, rgb};
use gpui_component::{StyledExt, v_virtual_list};

use crate::protocol::MessageRole;

use super::super::app::Shell;
use super::super::theme::*;

impl Shell {
    pub(super) fn conversation(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.item_count();
        let content = if count == 0 {
            div()
                .w_full()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .child(
                    div()
                        .id("empty-conversation-heading")
                        .role(Role::Heading)
                        .aria_level(2)
                        .aria_label("Start a Kimi Code session")
                        .text_xl()
                        .font_semibold()
                        .child("Start a Kimi Code session"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(TEXT_MUTED))
                        .child("Select a session or send a prompt to begin."),
                )
                .into_any_element()
        } else {
            let sizes = self.transcript.sizes.clone();
            v_virtual_list(cx.entity(), "conversation", sizes, |this, range, _, _| {
                range
                    .map(|index| this.message_item(index))
                    .collect::<Vec<_>>()
            })
            .size_full()
            .into_any_element()
        };
        div()
            .id("conversation-document")
            .role(Role::Document)
            .aria_label("Conversation")
            .w_full()
            .max_w(px(CONTENT_WIDTH))
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .pt_4()
            .child(content)
            .child(self.interactions(cx))
    }

    fn item_count(&self) -> usize {
        self.transcript.rows.len()
    }

    fn message_item(&self, index: usize) -> AnyElement {
        let Some((role, text)) = self.item_text(index) else {
            return div().into_any_element();
        };
        let user = role == MessageRole::User;
        let speaker = if user { "You" } else { "Kimi" };
        div()
            .id(("message", index))
            .role(Role::Article)
            .aria_label(format!("{speaker}: {text}"))
            .w_full()
            .px_3()
            .pb_3()
            .child(
                div()
                    .w_full()
                    .rounded_lg()
                    .p_3()
                    .bg(rgb(if user { SURFACE } else { ASSISTANT }))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .mb_2()
                            .text_xs()
                            .font_semibold()
                            .text_color(rgb(TEXT_MUTED))
                            .child(speaker),
                    )
                    .child(div().text_sm().whitespace_normal().child(text)),
            )
            .into_any_element()
    }

    fn item_text(&self, index: usize) -> Option<(MessageRole, String)> {
        let row = self.transcript.rows.get(index)?;
        Some((row.role, row.text.clone()))
    }
}
