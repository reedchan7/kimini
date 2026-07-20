use gpui::{AnyElement, Context, IntoElement, Role, div, prelude::*, px};
use gpui_component::{StyledExt, input::Input, scroll::ScrollableElement, text::TextView};

use super::super::app::Shell;
use super::super::side_chat::SideChatTurn;
use super::super::theme::*;
use super::accessible_input::accessible_input;
use super::panel::panel_button;

impl Shell {
    pub(super) fn side_chat_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let session_id = self
            .model
            .active_session()
            .map(|session| session.id.as_str())
            .unwrap_or_default();
        let chat = self.side_chats.get(session_id);
        let turns = chat.map(|chat| chat.turns.clone()).unwrap_or_default();
        let running = chat.is_some_and(|chat| chat.running);
        let sending = chat.is_some_and(|chat| chat.sending);
        let error = chat
            .and_then(|chat| chat.error.clone())
            .or_else(|| self.side_chats.open_error(session_id).map(str::to_owned));
        let opening = self.side_chats.is_opening(session_id);

        div()
            .id("side-chat-panel")
            .role(Role::Complementary)
            .aria_label(self.strings.native.side_chat_panel)
            .w(px(SIDE_CHAT_PANEL_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(theme_rgb(BORDER))
            .bg(theme_rgb(SURFACE))
            .child(self.side_chat_header(cx))
            .child(
                div()
                    .id("side-chat-transcript")
                    .role(Role::Log)
                    .aria_label(self.strings.native.side_chat_panel)
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .p_3()
                    .when(opening, |panel| {
                        panel.child(
                            div()
                                .py_8()
                                .text_center()
                                .text_size(font_px(13.0))
                                .text_color(theme_rgb(TEXT_MUTED))
                                .child(self.strings.native.side_chat_opening),
                        )
                    })
                    .when(!opening && turns.is_empty(), |panel| {
                        panel.child(
                            div()
                                .py_8()
                                .text_center()
                                .text_size(font_px(13.0))
                                .text_color(theme_rgb(TEXT_MUTED))
                                .child(self.strings.native.side_chat_empty),
                        )
                    })
                    .children(
                        turns
                            .iter()
                            .enumerate()
                            .map(|(index, turn)| self.side_chat_turn(index, turn)),
                    )
                    .when(sending, |panel| {
                        panel.child(
                            div()
                                .text_size(font_px(12.0))
                                .text_color(theme_rgb(TEXT_MUTED))
                                .child(self.strings.native.side_chat_thinking),
                        )
                    })
                    .when_some(error, |panel, error| {
                        panel.child(
                            div()
                                .mt_3()
                                .rounded_md()
                                .border_1()
                                .border_color(theme_rgb(ERROR))
                                .p_2()
                                .text_size(font_px(12.0))
                                .text_color(theme_rgb(ERROR))
                                .child(error),
                        )
                    }),
            )
            .child(self.side_chat_composer(running, opening, chat.is_some(), cx))
    }

    fn side_chat_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(58.0))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_3()
            .border_b_1()
            .border_color(theme_rgb(BORDER))
            .child(
                div()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(font_px(13.0))
                            .font_semibold()
                            .child(self.strings.native.side_chat),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(font_px(12.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(self.strings.native.side_chat_subtitle),
                    ),
            )
            .child(
                panel_button(self.strings.native.side_chat_close, "close-side-chat")
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_side_chat(window, cx))),
            )
    }

    fn side_chat_turn(&self, index: usize, turn: &SideChatTurn) -> AnyElement {
        div()
            .id(("side-chat-turn", index))
            .role(Role::Article)
            .aria_label(format!("{}: {}", self.strings.native.you, turn.user))
            .mb_4()
            .child(
                div()
                    .ml_5()
                    .rounded_lg()
                    .bg(theme_rgb(ASSISTANT))
                    .p_3()
                    .child(
                        TextView::markdown(("side-chat-user", index), turn.user.clone())
                            .selectable(true)
                            .text_size(body_font_px()),
                    ),
            )
            .when(!turn.thinking.is_empty(), |item| {
                item.child(
                    div()
                        .mt_3()
                        .text_size(caption_font_px())
                        .text_color(theme_rgb(TEXT_MUTED))
                        .child(format!(
                            "{}\n{}",
                            self.strings.native.side_chat_thinking, turn.thinking
                        )),
                )
            })
            .when(!turn.assistant.is_empty(), |item| {
                item.child(
                    div().mt_3().child(
                        TextView::markdown(("side-chat-assistant", index), turn.assistant.clone())
                            .selectable(true)
                            .text_size(body_font_px()),
                    ),
                )
            })
            .into_any_element()
    }

    fn side_chat_composer(
        &self,
        running: bool,
        opening: bool,
        ready: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex_none()
            .border_t_1()
            .border_color(theme_rgb(BORDER))
            .p_3()
            .child(
                div()
                    .rounded_lg()
                    .border_1()
                    .border_color(theme_rgb(BORDER))
                    .p_2()
                    .child(
                        accessible_input(
                            "side-chat-input",
                            &self.side_chat_input,
                            Role::MultilineTextInput,
                            self.strings.native.side_chat_panel,
                            self.strings.native.side_chat_placeholder,
                            Input::new(&self.side_chat_input),
                            cx,
                        )
                        .w_full(),
                    )
                    .child(
                        div()
                            .mt_2()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .when(running, |actions| {
                                actions.child(
                                    panel_button(
                                        self.strings.native.side_chat_stop,
                                        "stop-side-chat",
                                    )
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.stop_side_chat(cx)),
                                    ),
                                )
                            })
                            .when(ready && !opening, |actions| {
                                actions.child(
                                    panel_button(
                                        self.strings.native.side_chat_send,
                                        "send-side-chat",
                                    )
                                    .on_click(cx.listener(
                                        |this, _, window, cx| {
                                            this.send_side_chat_prompt(window, cx)
                                        },
                                    )),
                                )
                            }),
                    ),
            )
    }
}
