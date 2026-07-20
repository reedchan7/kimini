use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, Context, Corners, FontWeight, Role, div, prelude::*, px,
    relative,
};
use gpui_component::{StyledExt, text::TextView};

use crate::protocol::{MessageRole, PromptPart};

use super::super::app::Shell;
use super::super::presentation::{AttachmentKind, TranscriptBlock, TranscriptRow};
use super::super::streaming::streaming_text_view;
use super::super::theme::*;

impl Shell {
    pub(super) fn message_item(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(row) = self.transcript.rows.get(index) else {
            return div().into_any_element();
        };
        self.render_message_row(index, row, cx)
    }

    pub(super) fn pending_prompt_preview(
        &self,
        parts: &[PromptPart],
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let user = TranscriptRow::from_prompt_parts("new-session-user", parts);
        let pending = TranscriptRow::from_stream("new-session", None, Some(""))
            .expect("a pending stream always has a row");
        div()
            .w_full()
            .child(self.render_message_row(0, &user, cx))
            .child(self.render_message_row(1, &pending, cx))
            .into_any_element()
    }

    fn render_message_row(
        &self,
        index: usize,
        row: &TranscriptRow,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let speaker = speaker(row.role, self.strings.native);
        let blocks = self.render_blocks(index, &row.blocks, row.streaming, cx);
        let is_user = row.role == MessageRole::User;
        let waiting = row.streaming && row.blocks.is_empty();
        // Each row owns a centered CONTENT_WIDTH column. The scroll container
        // is now full pane width (see `conversation`), so wheel events in the
        // gutters still reach the list while the message column stays aligned
        // with the composer and 760px reading measure.
        div()
            .id(("message", index))
            .role(Role::Article)
            .aria_label(if waiting {
                self.strings.native.waiting_for_response.into()
            } else {
                format!("{speaker}: {}", row.accessible_text())
            })
            .w_full()
            .flex()
            .justify_center()
            .px_3()
            // Web `--chat-turn-gap` between turns. Use the gap token directly
            // so changing the design constant later ripples here.
            .pb(px(CHAT_TURN_GAP))
            .child(
                div()
                    .w_full()
                    .max_w(px(CONTENT_WIDTH))
                    .flex()
                    .when(is_user, |container| container.justify_end())
                    .child(
                        div()
                            .when(is_user, |body| {
                                // Web `.u-bub`: max-width 78%, accent-tinted fill,
                                // accent border, asymmetric "tail" radius
                                // (16/16/6/16), 11×15 padding, subtle shadow.
                                body.max_w(relative(0.78))
                                    .bg(theme_rgb(ACCENT_SOFT))
                                    .border_1()
                                    .border_color(theme_rgba(ACCENT_BORDER))
                                    .corner_radii(Corners {
                                        top_left: px(16.0),
                                        top_right: px(16.0),
                                        // Tail corner at bottom-right.
                                        bottom_right: px(6.0),
                                        bottom_left: px(16.0),
                                    })
                                    .px(px(15.0))
                                    .py(px(11.0))
                                    .shadow_xs()
                                    .text_size(text_lg_font_px())
                            })
                            .when(!is_user, |body| {
                                // Web `.a-msg`: 94% column, medium-weight body.
                                body.max_w(relative(0.94))
                                    .w(relative(0.94))
                                    .font_weight(FontWeight::MEDIUM)
                            })
                            .when(row.role == MessageRole::System, |body| {
                                body.border_l_2()
                                    .border_color(theme_rgb(BORDER_STRONG))
                                    .pl_3()
                            })
                            .when(waiting, |body| {
                                body.child(self.kimi_waiting_indicator(index, cx))
                            })
                            .when(
                                !waiting && (row.role == MessageRole::System || row.streaming),
                                |body| {
                                    body.child(
                                        div()
                                            .mb_2()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .text_size(gpui::px(11.0))
                                            .font_semibold()
                                            .text_color(theme_rgb(TEXT_MUTED))
                                            .child(speaker),
                                    )
                                },
                            )
                            .children(blocks),
                    ),
            )
            .into_any_element()
    }

    /// Blinking caret that mirrors the Web `typewriter-cursor-blink` animation
    /// while the assistant row is streaming. Rendered as a thin accent bar
    /// anchored to the baseline of the streaming markdown body. Static when
    /// the user has reduced motion enabled.
    fn streaming_caret(&self, key: usize, cx: &Context<Self>) -> AnyElement {
        const PERIOD: Duration = Duration::from_millis(850);
        // 1px wide × ~1.15em tall, matching the Web `|` caret.
        let bar = div()
            .id(("streaming-caret", key))
            .role(Role::Image)
            .aria_label(self.strings.native.waiting_for_response)
            .w(px(2.0))
            .h(body_font_px() * 1.15)
            .rounded_sm()
            .bg(theme_rgb(ACCENT));
        if cx.reduce_motion() {
            return bar.into_any_element();
        }
        // Web cursor cycle: 0-49% visible, 50-100% hidden. `delta` runs
        // 0..1 each period; flip at the midpoint.
        bar.with_animation(
            ("streaming-caret-blink", key),
            Animation::new(PERIOD).repeat(),
            |bar, delta| bar.opacity(if delta < 0.5 { 1.0 } else { 0.0 }),
        )
        .into_any_element()
    }

    fn kimi_waiting_indicator(&self, key: usize, cx: &Context<Self>) -> AnyElement {
        const FRAMES: [&str; 8] = ["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"];
        let moon = div()
            .id(("kimi-waiting-indicator", key))
            .role(Role::Image)
            .aria_label(self.strings.native.waiting_for_response)
            .size(px(18.0))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(18.0));
        if cx.reduce_motion() {
            return moon.child(FRAMES[3]).into_any_element();
        }
        moon.with_animation(
            ("kimi-moon-frame", key),
            Animation::new(Duration::from_millis(960)).repeat(),
            move |moon, delta| {
                let frame = (delta * FRAMES.len() as f32).floor() as usize % FRAMES.len();
                moon.child(FRAMES[frame])
            },
        )
        .into_any_element()
    }

    fn render_blocks(
        &self,
        message_index: usize,
        blocks: &[TranscriptBlock],
        streaming: bool,
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
            rendered.push(self.render_block(message_index, index, &blocks[index], streaming, cx));
            index += 1;
        }
        rendered
    }

    fn render_block(
        &self,
        index: usize,
        block_index: usize,
        block: &TranscriptBlock,
        streaming: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let key = index.saturating_mul(10_000).saturating_add(block_index);
        let strings = self.strings.native;
        match block {
            TranscriptBlock::Text(text) => {
                // Streaming rows render into a long-lived TextViewState so the
                // parsed markdown AST survives across deltas and the parse
                // pipeline can append new suffixes off-thread. Non-streaming
                // rows keep the stateless `TextView::markdown` form.
                if streaming
                    && let Some(entity) = self
                        .streaming
                        .as_ref()
                        .map(|state| state.assistant_entity().clone())
                {
                    div()
                        .flex()
                        .items_end()
                        .gap_1()
                        .child(
                            streaming_text_view(&entity)
                                .text_size(content_font_px())
                                .line_height(relative(1.6)),
                        )
                        .child(self.streaming_caret(key, cx))
                        .into_any_element()
                } else {
                    TextView::markdown(("message-markdown", key), text.clone())
                        .selectable(true)
                        .text_size(content_font_px())
                        .line_height(relative(1.6))
                        .into_any_element()
                }
            }
            TranscriptBlock::Thinking(text) => self.thinking_block(key, text, streaming, cx),
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

    fn thinking_block(
        &self,
        key: usize,
        body: &str,
        streaming: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let text = body.to_owned();
        let trace_view = if streaming
            && let Some(entity) = self
                .streaming
                .as_ref()
                .map(|state| state.thinking_entity().clone())
        {
            streaming_text_view(&entity)
                .text_size(body_font_px())
                .line_height(relative(1.5))
                .text_color(theme_rgb(TEXT_SECONDARY))
                .into_any_element()
        } else {
            TextView::markdown(("thinking-markdown", key), body.to_owned())
                .selectable(true)
                .text_size(body_font_px())
                .line_height(relative(1.5))
                .text_color(theme_rgb(TEXT_SECONDARY))
                .into_any_element()
        };
        div()
            .id(("thinking-trace", key))
            .mt(px(CHAT_BLOCK_GAP))
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
            .child(div().max_h(gpui::px(72.0)).overflow_hidden().child(trace_view))
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
                .text_size(body_font_px())
                .line_height(relative(1.55)),
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
