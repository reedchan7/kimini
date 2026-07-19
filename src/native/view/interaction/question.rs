use gpui::{AnyElement, Context, Role, div, prelude::*, px};
use gpui_component::{StyledExt, input::Input};

use crate::protocol::QuestionRequest;

use super::super::super::app::Shell;
use super::super::super::theme::*;
use super::super::accessible_input::accessible_input;
use super::action_button;

pub(super) fn render(
    shell: &Shell,
    cx: &mut Context<Shell>,
    requests: Vec<QuestionRequest>,
) -> Vec<AnyElement> {
    requests
        .into_iter()
        .enumerate()
        .map(|(request_index, request)| render_request(shell, cx, request_index, request))
        .collect()
}

fn render_request(
    shell: &Shell,
    cx: &mut Context<Shell>,
    request_index: usize,
    request: QuestionRequest,
) -> AnyElement {
    let can_submit = shell.question_drafts.answers(&request).is_some();
    let question_id = request.question_id.clone();
    let dismiss_id = request.question_id.clone();
    let items = request
        .questions
        .iter()
        .enumerate()
        .map(|(item_index, item)| {
            let options = item
                .options
                .iter()
                .enumerate()
                .map(|(option_index, option)| {
                    let selected = shell.question_drafts.is_selected(
                        &request.question_id,
                        &item.id,
                        &option.id,
                    );
                    let request_id = request.question_id.clone();
                    let item_id = item.id.clone();
                    let option_id = option.id.clone();
                    let multi = item.multi_select;
                    div()
                        .id((
                            "question-option",
                            request_index * 10_000 + item_index * 100 + option_index,
                        ))
                        .focusable()
                        .tab_stop(true)
                        .role(if multi {
                            Role::CheckBox
                        } else {
                            Role::RadioButton
                        })
                        .aria_label(option.label.clone())
                        .cursor_pointer()
                        .rounded_md()
                        .border_1()
                        .border_color(theme_rgb(if selected { ACCENT } else { BORDER }))
                        .bg(theme_rgb(if selected { SURFACE_ACTIVE } else { SURFACE }))
                        .px_3()
                        .py_2()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.toggle_question_option(
                                request_id.clone(),
                                item_id.clone(),
                                option_id.clone(),
                                multi,
                                cx,
                            )
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .text_size(font_px(13.0))
                                .font_semibold()
                                .child(if selected { "✓" } else { "○" })
                                .child(option.label.clone())
                                .when(option.recommended, |label| {
                                    label.child(
                                        div()
                                            .text_size(font_px(12.0))
                                            .text_color(theme_rgb(TEXT_MUTED))
                                            .child(shell.strings.native.recommended),
                                    )
                                }),
                        )
                        .children(option.description.clone().map(|description| {
                            div()
                                .mt_1()
                                .text_size(font_px(12.0))
                                .text_color(theme_rgb(TEXT_MUTED))
                                .child(description)
                        }))
                })
                .collect::<Vec<_>>();
            let other = item.allow_other.then(|| {
                let selected = shell
                    .question_drafts
                    .is_other_selected(&request.question_id, &item.id);
                let label = item
                    .other_label
                    .clone()
                    .unwrap_or_else(|| shell.strings.native.other_answer.into());
                let input = shell
                    .question_drafts
                    .other_input(&request.question_id, &item.id);
                div()
                    .id(("question-other", request_index * 100 + item_index))
                    .role(if item.multi_select {
                        Role::CheckBox
                    } else {
                        Role::RadioButton
                    })
                    .aria_label(label.clone())
                    .rounded_md()
                    .border_1()
                    .border_color(theme_rgb(if selected { ACCENT } else { BORDER }))
                    .bg(theme_rgb(if selected { SURFACE_ACTIVE } else { SURFACE }))
                    .px_3()
                    .py_2()
                    .child(
                        div()
                            .mb_2()
                            .text_size(font_px(13.0))
                            .font_semibold()
                            .child(if selected { "✓" } else { "○" })
                            .child(format!(" {label}")),
                    )
                    .children(item.other_description.clone().map(|description| {
                        div()
                            .mb_2()
                            .text_size(font_px(12.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(description)
                    }))
                    .children(input.map(|input| {
                        accessible_input(
                            ("question-other-input", request_index * 100 + item_index),
                            &input,
                            Role::TextInput,
                            label.clone(),
                            label,
                            Input::new(&input),
                            cx,
                        )
                        .h(px(34.0))
                    }))
            });
            div()
                .mb_3()
                .children(item.header.clone().map(|header| {
                    div()
                        .mb_1()
                        .text_size(font_px(12.0))
                        .text_color(theme_rgb(TEXT_MUTED))
                        .child(header)
                }))
                .child(
                    div()
                        .text_size(font_px(13.0))
                        .font_semibold()
                        .child(item.question.clone()),
                )
                .children(item.body.clone().map(|body| {
                    div()
                        .mt_1()
                        .text_size(font_px(13.0))
                        .text_color(theme_rgb(TEXT_MUTED))
                        .child(body)
                }))
                .child(
                    div()
                        .mt_2()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(options)
                        .children(other),
                )
        })
        .collect::<Vec<_>>();

    div()
        .id(("question", request_index))
        .role(Role::Group)
        .aria_label(shell.strings.native.question_required)
        .mb_2()
        .rounded_lg()
        .border_1()
        .border_color(theme_rgb(BORDER))
        .bg(theme_rgb(SURFACE))
        .p_3()
        .children(items)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    action_button(
                        shell.strings.native.dismiss_question,
                        ("dismiss-question", request_index),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.dismiss_question(dismiss_id.clone(), cx)
                    })),
                )
                .child(
                    action_button(
                        shell.strings.native.submit_answers,
                        ("answer", request_index),
                    )
                    .when(can_submit, |button| {
                        button
                            .bg(theme_rgb(ACCENT))
                            .text_color(theme_rgb(SURFACE))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.submit_question(question_id.clone(), cx)
                            }))
                    })
                    .when(!can_submit, |button| {
                        button.cursor_default().text_color(theme_rgb(TEXT_MUTED))
                    }),
                ),
        )
        .into_any_element()
}
