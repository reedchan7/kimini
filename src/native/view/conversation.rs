use gpui::{Context, IntoElement, Role, div, list, prelude::*, px};
use gpui_component::{StyledExt, scroll::ScrollableElement};

use crate::protocol::MessageRole;

use super::super::app::Shell;
use super::super::presentation::TranscriptRow;
use super::super::theme::*;

impl Shell {
    pub(super) fn conversation(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let can_load_older = self
            .model
            .active_conversation()
            .is_some_and(|conversation| conversation.has_more_messages);
        let content = if self.transcript.rows.is_empty() {
            empty_conversation(self.strings.native).into_any_element()
        } else {
            let shell = cx.entity();
            let list_state = self.transcript.list.clone();
            div()
                .size_full()
                .relative()
                .child(
                    list(list_state.clone(), move |index, _, cx| {
                        shell.update(cx, |this, cx| this.message_item(index, cx))
                    })
                    .size_full(),
                )
                .vertical_scrollbar(&list_state)
                .into_any_element()
        };
        let outline = self
            .preferences
            .conversation_outline
            .then(|| conversation_outline_items(&self.transcript.rows))
            .filter(|items| items.len() > 1);
        // The scroll container takes the full chat pane width so wheel events
        // landing in the side gutters (outside the 760px content column) still
        // scroll the list. Per-row content is capped to CONTENT_WIDTH inside
        // `render_message_row`, so the visual layout is unchanged.
        div()
            .id("conversation-document")
            .role(Role::Document)
            .aria_label(self.strings.native.conversation)
            .w_full()
            .relative()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            // Web `.chat` padding-top is 22px; the inline + bottom padding
            // is applied per-row so the centered CONTENT_WIDTH column lines
            // up with the composer.
            .pt(px(22.0))
            .when(can_load_older, |conversation| {
                conversation.child(
                    div()
                        .flex_none()
                        .flex()
                        .justify_center()
                        .pb_3()
                        .child(
                            div()
                                .id("load-older-messages")
                                .focusable()
                                .tab_stop(true)
                                .role(Role::Button)
                                .aria_label(self.strings.native.load_earlier)
                                .cursor_pointer()
                                .rounded_md()
                                .px_3()
                                .py_1()
                                .text_size(font_px(12.0))
                                .text_color(theme_rgb(TEXT_MUTED))
                                .hover(|item| item.bg(theme_rgb(SURFACE_ACTIVE)))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.load_older_messages(cx)
                                }))
                                .child(if self.history_loading {
                                    self.strings.native.loading_earlier
                                } else {
                                    self.strings.native.load_earlier
                                }),
                        ),
                )
            })
            .child(content)
            .when_some(outline, |conversation, items| {
                conversation.child(self.conversation_outline(items, cx))
            })
            .child(self.prompt_queue(cx))
            .child(self.interactions(cx))
    }

    fn conversation_outline(
        &self,
        items: Vec<(usize, String)>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let list = self.transcript.list.clone();
        div()
            .id("conversation-outline")
            .role(Role::Navigation)
            .aria_label(self.strings.native.settings_show_outline)
            .absolute()
            .top(px(16.0))
            .right(px(-154.0))
            .w(px(142.0))
            .max_h(px(420.0))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_1()
            .children(
                items
                    .into_iter()
                    .enumerate()
                    .map(|(position, (index, label))| {
                        let list = list.clone();
                        div()
                            .id(("conversation-outline-item", position))
                            .role(Role::Link)
                            .aria_label(label.clone())
                            .focusable()
                            .tab_stop(true)
                            .cursor_pointer()
                            .rounded_md()
                            .px_2()
                            .py_1()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .text_size(font_px(11.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .hover(|item| {
                                item.bg(theme_rgb(SURFACE_ACTIVE))
                                    .text_color(theme_rgb(TEXT))
                            })
                            .on_click(cx.listener(move |_, _, _, cx| {
                                list.scroll_to_reveal_item(index);
                                cx.notify();
                            }))
                            .child(label)
                    }),
            )
    }
}

fn conversation_outline_items(rows: &[TranscriptRow]) -> Vec<(usize, String)> {
    rows.iter()
        .enumerate()
        .filter(|(_, row)| row.role == MessageRole::User)
        .filter_map(|(index, row)| {
            let text = row.accessible_text();
            if text.trim_start().starts_with("<system-reminder>") {
                return None;
            }
            let label = text
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(72)
                .collect::<String>();
            Some((index, label))
        })
        .filter(|(_, label)| !label.is_empty())
        .collect()
}

fn empty_conversation(strings: crate::i18n::NativeStrings) -> impl IntoElement {
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
                .aria_label(strings.start_session)
                .text_size(font_px(18.0))
                .font_semibold()
                .child(strings.start_session),
        )
        .child(
            div()
                .text_size(body_font_px())
                .text_color(theme_rgb(TEXT_MUTED))
                .child(strings.start_session_hint),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::presentation::TranscriptBlock;

    fn row(role: MessageRole, text: &str) -> TranscriptRow {
        TranscriptRow {
            id: text.into(),
            role,
            blocks: vec![TranscriptBlock::Text(text.into())],
            streaming: false,
        }
    }

    #[test]
    fn outline_contains_only_compact_user_queries() {
        let rows = vec![
            row(MessageRole::User, "  first\nquestion  "),
            row(MessageRole::Assistant, "answer"),
            row(
                MessageRole::User,
                "  <system-reminder>internal context</system-reminder>",
            ),
            row(MessageRole::User, "second question"),
        ];

        assert_eq!(
            conversation_outline_items(&rows),
            vec![(0, "first question".into()), (3, "second question".into())]
        );
    }
}
