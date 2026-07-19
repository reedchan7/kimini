use gpui::{AnyElement, Context, Role, div, prelude::*};
use gpui_component::{StyledExt, text::TextView};

use crate::protocol::MessageRole;

use super::super::app::Shell;
use super::super::presentation::{AttachmentKind, TranscriptBlock};
use super::super::theme::*;

impl Shell {
    pub(super) fn message_item(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(row) = self.transcript.rows.get(index) else {
            return div().into_any_element();
        };
        let speaker = speaker(row.role, self.strings.native);
        let blocks = self.render_blocks(index, &row.blocks, cx);
        let is_user = row.role == MessageRole::User;
        div()
            .id(("message", index))
            .role(Role::Article)
            .aria_label(format!("{speaker}: {}", row.accessible_text()))
            .w_full()
            .px_3()
            .pb_5()
            .child(
                div()
                    .w_full()
                    .flex()
                    .when(is_user, |container| container.justify_end())
                    .child(
                        div()
                            .when(is_user, |body| {
                                body.max_w(gpui::relative(0.82))
                                    .rounded_lg()
                                    .bg(theme_rgb(SURFACE_SUBTLE))
                                    .px_3()
                                    .py_2()
                            })
                            .when(!is_user, |body| body.w_full())
                            .when(row.role == MessageRole::System, |body| {
                                body.border_l_2()
                                    .border_color(theme_rgb(BORDER_STRONG))
                                    .pl_3()
                            })
                            .when(row.role == MessageRole::System || row.streaming, |body| {
                                body.child(
                                    div()
                                        .mb_2()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_size(gpui::px(11.0))
                                        .font_semibold()
                                        .text_color(theme_rgb(TEXT_MUTED))
                                        .child(speaker)
                                        .when(row.streaming, |header| {
                                            header
                                                .child(format!("· {}", self.strings.native.working))
                                        }),
                                )
                            })
                            .children(blocks),
                    ),
            )
            .into_any_element()
    }

    fn render_blocks(
        &self,
        message_index: usize,
        blocks: &[TranscriptBlock],
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut rendered = Vec::new();
        let mut index = 0;
        while index < blocks.len() {
            if matches!(blocks[index], TranscriptBlock::Tool(_)) {
                let start = index;
                while index < blocks.len() && matches!(blocks[index], TranscriptBlock::Tool(_)) {
                    index += 1;
                }
                let tools = blocks[start..index]
                    .iter()
                    .filter_map(|block| match block {
                        TranscriptBlock::Tool(tool) => Some(tool),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                rendered.push(self.tool_group(
                    message_index.saturating_mul(10_000).saturating_add(start),
                    &tools,
                    cx,
                ));
                continue;
            }
            rendered.push(self.render_block(message_index, index, &blocks[index], cx));
            index += 1;
        }
        rendered
    }

    fn render_block(
        &self,
        index: usize,
        block_index: usize,
        block: &TranscriptBlock,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let key = index.saturating_mul(10_000).saturating_add(block_index);
        let strings = self.strings.native;
        match block {
            TranscriptBlock::Text(text) => {
                TextView::markdown(("message-markdown", key), text.clone())
                    .selectable(true)
                    .text_size(font_px(13.0))
                    .into_any_element()
            }
            TranscriptBlock::Thinking(text) => self.thinking_block(key, text, cx),
            TranscriptBlock::Tool(tool) => self.tool_card(key, tool, cx),
            TranscriptBlock::Attachment { kind, name, detail } => attachment_block(
                match kind {
                    AttachmentKind::Image => strings.attachment_image,
                    AttachmentKind::Video => strings.attachment_video,
                    AttachmentKind::File => strings.attachment_file,
                },
                name,
                detail,
            ),
            TranscriptBlock::Unknown { kind, value } => semantic_block(kind, false, key, value),
        }
    }

    fn thinking_block(&self, key: usize, body: &str, cx: &mut Context<Self>) -> AnyElement {
        let text = body.to_owned();
        div()
            .id(("thinking-trace", key))
            .mt_2()
            .focusable()
            .tab_stop(true)
            .role(Role::Button)
            .aria_label(self.strings.native.preview_thinking)
            .cursor_pointer()
            .border_l_2()
            .border_color(theme_rgb(BORDER_STRONG))
            .pl_3()
            .py_1()
            .hover(|item| item.border_color(theme_rgb(ACCENT)))
            .on_click(
                cx.listener(move |this, _, _, cx| this.open_thinking_preview(text.clone(), cx)),
            )
            .child(
                div()
                    .mb_1()
                    .flex()
                    .items_center()
                    .justify_between()
                    .text_size(gpui::px(11.0))
                    .font_semibold()
                    .text_color(theme_rgb(TEXT_MUTED))
                    .child(self.strings.native.thinking)
                    .child(self.strings.native.preview_thinking),
            )
            .child(
                div().max_h(gpui::px(64.0)).overflow_hidden().child(
                    TextView::markdown(("thinking-markdown", key), body.to_owned())
                        .selectable(true)
                        .text_size(font_px(13.0))
                        .text_color(theme_rgb(TEXT_SECONDARY)),
                ),
            )
            .into_any_element()
    }
}

fn attachment_block(label: &str, name: &str, detail: &str) -> AnyElement {
    div()
        .mt_2()
        .rounded_md()
        .border_1()
        .border_color(theme_rgb(BORDER))
        .bg(theme_rgb(SURFACE))
        .px_3()
        .py_2()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_size(font_px(12.0))
                .font_semibold()
                .child(label.to_owned()),
        )
        .child(
            div()
                .text_size(font_px(13.0))
                .line_clamp(1)
                .child(name.to_owned()),
        )
        .child(
            div()
                .text_size(font_px(12.0))
                .text_color(theme_rgb(TEXT_MUTED))
                .line_clamp(1)
                .child(detail.to_owned()),
        )
        .into_any_element()
}

fn semantic_block(label: &str, error: bool, key: usize, body: &str) -> AnyElement {
    div()
        .mt_2()
        .rounded_md()
        .border_1()
        .border_color(theme_rgb(if error { ERROR } else { BORDER }))
        .bg(theme_rgb(SURFACE))
        .p_3()
        .child(
            div()
                .mb_2()
                .text_size(font_px(12.0))
                .font_semibold()
                .text_color(theme_rgb(if error { ERROR } else { TEXT_MUTED }))
                .child(label.to_owned()),
        )
        .child(
            TextView::markdown(("semantic-markdown", key), body.to_owned())
                .selectable(true)
                .text_size(font_px(13.0)),
        )
        .into_any_element()
}

fn speaker(role: MessageRole, strings: crate::i18n::NativeStrings) -> &'static str {
    match role {
        MessageRole::User => strings.you,
        MessageRole::Assistant => strings.kimi,
        MessageRole::Tool => strings.tool,
        MessageRole::System => strings.system,
    }
}
