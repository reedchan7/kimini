use gpui::{Context, IntoElement, Role, div, list, prelude::*, px, rgb};
use gpui_component::{StyledExt, scroll::ScrollableElement};

use super::super::app::Shell;
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
        div()
            .id("conversation-document")
            .role(Role::Document)
            .aria_label(self.strings.native.conversation)
            .w_full()
            .max_w(px(CONTENT_WIDTH))
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .pt_4()
            .when(can_load_older, |conversation| {
                conversation.child(
                    div().flex_none().flex().justify_center().pb_3().child(
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
                            .text_xs()
                            .text_color(rgb(TEXT_MUTED))
                            .hover(|item| item.bg(rgb(SURFACE_ACTIVE)))
                            .on_click(cx.listener(|this, _, _, cx| this.load_older_messages(cx)))
                            .child(if self.history_loading {
                                self.strings.native.loading_earlier
                            } else {
                                self.strings.native.load_earlier
                            }),
                    ),
                )
            })
            .child(content)
            .child(self.prompt_queue(cx))
            .child(self.interactions(cx))
    }
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
                .text_xl()
                .font_semibold()
                .child(strings.start_session),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(TEXT_MUTED))
                .child(strings.start_session_hint),
        )
}
