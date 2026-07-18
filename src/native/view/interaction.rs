use gpui::{AnyElement, Context, IntoElement, Role, div, prelude::*, rgb};
use gpui_component::StyledExt;

use super::super::app::Shell;
use super::super::theme::*;

impl Shell {
    pub(super) fn interactions(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let (approvals, questions) = self
            .model
            .active_conversation()
            .map(|conversation| {
                (
                    conversation.approvals.clone(),
                    conversation.questions.clone(),
                )
            })
            .unwrap_or_default();
        let mut cards = Vec::<AnyElement>::new();

        for (index, approval) in approvals.into_iter().enumerate() {
            let approve_id = approval.approval_id.clone();
            let reject_id = approval.approval_id;
            cards.push(
                div()
                    .id(("approval", index))
                    .role(Role::Group)
                    .aria_label(format!(
                        "Approval required: {} — {}",
                        approval.tool_name, approval.action
                    ))
                    .mb_2()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(BORDER))
                    .bg(rgb(SURFACE))
                    .p_3()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .child(format!("{} · {}", approval.tool_name, approval.action)),
                    )
                    .child(
                        div()
                            .mt_3()
                            .flex()
                            .gap_2()
                            .child(action_button("Reject", ("reject", index)).on_click(
                                cx.listener(move |this, _, _, cx| {
                                    this.resolve_approval(reject_id.clone(), false, cx)
                                }),
                            ))
                            .child(
                                action_button("Approve", ("approve", index))
                                    .bg(rgb(ACCENT))
                                    .text_color(rgb(SURFACE))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.resolve_approval(approve_id.clone(), true, cx)
                                    })),
                            ),
                    )
                    .into_any_element(),
            );
        }

        for (request_index, request) in questions.into_iter().enumerate() {
            for (item_index, item) in request.questions.into_iter().enumerate() {
                let mut options = Vec::<AnyElement>::new();
                for (option_index, option) in item.options.into_iter().enumerate() {
                    let question_id = request.question_id.clone();
                    let item_id = item.id.clone();
                    let option_id = option.id;
                    options.push(
                        action_button(
                            option.label,
                            (
                                "option",
                                request_index * 1000 + item_index * 100 + option_index,
                            ),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.resolve_question(
                                question_id.clone(),
                                item_id.clone(),
                                option_id.clone(),
                                cx,
                            )
                        }))
                        .into_any_element(),
                    );
                }
                cards.push(
                    div()
                        .id(("question", request_index * 100 + item_index))
                        .role(Role::Group)
                        .aria_label(format!("Question: {}", item.question))
                        .mb_2()
                        .rounded_lg()
                        .border_1()
                        .border_color(rgb(BORDER))
                        .bg(rgb(SURFACE))
                        .p_3()
                        .child(div().text_sm().font_semibold().child(item.question))
                        .child(div().mt_2().flex().flex_wrap().gap_2().children(options))
                        .into_any_element(),
                );
            }
        }

        div().flex_none().px_3().children(cards)
    }
}

fn action_button(
    label: impl Into<gpui::SharedString>,
    id: impl Into<gpui::ElementId>,
) -> gpui::Stateful<gpui::Div> {
    let label = label.into();
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(label.clone())
        .cursor_pointer()
        .rounded_md()
        .border_1()
        .border_color(rgb(BORDER))
        .px_3()
        .py_1()
        .text_sm()
        .child(label)
}
