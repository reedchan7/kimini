use gpui::{AnyElement, Context, Role, div, prelude::*};
use gpui_component::{StyledExt, text::TextView};

use crate::protocol::ApprovalRequest;

use super::super::super::app::Shell;
use super::super::super::theme::*;
use super::action_button;

pub(super) fn render(
    shell: &Shell,
    cx: &mut Context<Shell>,
    approvals: Vec<ApprovalRequest>,
) -> Vec<AnyElement> {
    approvals
        .into_iter()
        .enumerate()
        .map(|(index, approval)| {
            let reject_id = approval.approval_id.clone();
            let once_id = approval.approval_id.clone();
            let session_id = approval.approval_id.clone();
            let input = (!approval.tool_input_display.is_null())
                .then(|| display_value(&approval.tool_input_display));
            div()
                .id(("approval", index))
                .role(Role::Group)
                .aria_label(format!(
                    "{}: {} — {}",
                    shell.strings.native.approval_required, approval.tool_name, approval.action
                ))
                .mb_2()
                .rounded_lg()
                .border_1()
                .border_color(theme_rgb(BORDER))
                .bg(theme_rgb(SURFACE))
                .p_3()
                .child(
                    div()
                        .text_size(font_px(13.0))
                        .font_semibold()
                        .child(format!("{} · {}", approval.tool_name, approval.action)),
                )
                .children(input.map(|input| {
                    TextView::markdown(("approval-input", index), input)
                        .selectable(true)
                        .text_size(font_px(13.0))
                }))
                .child(
                    div()
                        .mt_3()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .child(
                            action_button(shell.strings.native.reject, ("reject", index)).on_click(
                                cx.listener(move |this, _, _, cx| {
                                    this.resolve_approval(reject_id.clone(), false, false, cx)
                                }),
                            ),
                        )
                        .child(
                            action_button(
                                shell.strings.native.approve_once,
                                ("approve-once", index),
                            )
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    this.resolve_approval(once_id.clone(), true, false, cx)
                                },
                            )),
                        )
                        .child(
                            action_button(
                                shell.strings.native.approve_session,
                                ("approve-session", index),
                            )
                            .bg(theme_rgb(ACCENT))
                            .text_color(theme_rgb(SURFACE))
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    this.resolve_approval(session_id.clone(), true, true, cx)
                                },
                            )),
                        ),
                )
                .into_any_element()
        })
        .collect()
}

fn display_value(value: &serde_json::Value) -> String {
    value.as_str().map(str::to_owned).unwrap_or_else(|| {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    })
}
