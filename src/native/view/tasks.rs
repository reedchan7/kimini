use gpui::{AnyElement, Context, IntoElement, Role, div, prelude::*, px};
use gpui_component::{StyledExt, scroll::ScrollableElement, text::TextView};

use crate::protocol::{Task, TaskKind, TaskStatus};

use super::super::app::Shell;
use super::super::theme::*;
use super::panel::panel_button;

impl Shell {
    pub(super) fn task_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let session_id = self
            .model
            .active_session()
            .map(|session| session.id.as_str())
            .unwrap_or_default();
        let tasks = self.tasks.for_session(session_id).to_vec();
        let running = tasks.iter().filter(|task| task.is_running()).count();
        div()
            .id("task-panel")
            .role(Role::Group)
            .aria_label(self.strings.native.tasks_panel)
            .w(px(TASK_PANEL_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(theme_rgb(BORDER))
            .bg(theme_rgb(SURFACE))
            .child(self.task_panel_header(tasks.len(), running, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .p_3()
                    .when_some(self.task_error.clone(), |panel, error| {
                        panel.child(
                            div()
                                .mb_3()
                                .rounded_md()
                                .border_1()
                                .border_color(theme_rgb(BORDER_STRONG))
                                .bg(theme_rgb(ERROR_SOFT))
                                .p_2()
                                .text_size(font_px(12.0))
                                .text_color(theme_rgb(ERROR))
                                .child(error),
                        )
                    })
                    .when(tasks.is_empty(), |panel| {
                        panel.child(
                            div()
                                .py_8()
                                .text_center()
                                .text_size(font_px(13.0))
                                .text_color(theme_rgb(TEXT_MUTED))
                                .child(if self.tasks_loading {
                                    self.strings.native.tasks_loading
                                } else {
                                    self.strings.native.no_tasks
                                }),
                        )
                    })
                    .children(
                        tasks
                            .iter()
                            .enumerate()
                            .map(|(index, task)| self.task_card(index, task, cx)),
                    ),
            )
    }

    fn task_panel_header(
        &self,
        total: usize,
        running: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .h(px(48.0))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .border_b_1()
            .border_color(theme_rgb(BORDER))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(font_px(13.0))
                    .font_semibold()
                    .child(self.strings.native.tasks)
                    .child(
                        div()
                            .text_size(font_px(12.0))
                            .font_normal()
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(format!("{running}/{total}")),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        panel_button(self.strings.native.refresh_tasks, "refresh-tasks")
                            .on_click(cx.listener(|this, _, _, cx| this.refresh_tasks(cx))),
                    )
                    .child(
                        panel_button(self.strings.native.close_tasks, "close-tasks")
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_task_panel(cx))),
                    ),
            )
    }

    fn task_card(&self, index: usize, task: &Task, cx: &mut Context<Self>) -> AnyElement {
        let status = task_status_label(task.status, self.strings.native);
        let kind = task_kind_label(task.kind, self.strings.native);
        let task_id = task.id.clone();
        div()
            .id(("task-card", index))
            .role(Role::Article)
            .aria_label(format!("{kind}: {} · {status}", task.description))
            .mb_3()
            .rounded_lg()
            .border_1()
            .border_color(theme_rgb(if task.status == TaskStatus::Failed {
                ERROR
            } else {
                BORDER
            }))
            .bg(theme_rgb(CANVAS))
            .p_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .text_size(font_px(12.0))
                            .font_semibold()
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(kind),
                    )
                    .child(
                        div()
                            .text_size(font_px(12.0))
                            .text_color(theme_rgb(status_color(task.status)))
                            .child(status),
                    ),
            )
            .child(
                div()
                    .mt_2()
                    .text_size(font_px(13.0))
                    .font_semibold()
                    .child(task.description.clone()),
            )
            .when_some(task.command.clone(), |card, command| {
                card.child(
                    div()
                        .mt_2()
                        .rounded_md()
                        .bg(theme_rgb(SURFACE))
                        .p_2()
                        .child(
                            TextView::markdown(
                                ("task-command", index),
                                format!("```\n{command}\n```"),
                            )
                            .selectable(true)
                            .text_size(font_px(12.0)),
                        ),
                )
            })
            .when_some(task.output_preview.clone(), |card, output| {
                card.child(
                    div()
                        .mt_2()
                        .max_h(px(180.0))
                        .overflow_y_scrollbar()
                        .rounded_md()
                        .bg(theme_rgb(SURFACE))
                        .p_2()
                        .child(
                            TextView::markdown(("task-output", index), output)
                                .selectable(true)
                                .text_size(font_px(12.0)),
                        ),
                )
            })
            .when(task.is_running(), |card| {
                card.child(div().mt_2().flex().justify_end().child(
                    panel_button(self.strings.native.cancel_task, ("cancel-task", index)).on_click(
                        cx.listener(move |this, _, _, cx| {
                            this.cancel_background_task(task_id.clone(), cx)
                        }),
                    ),
                ))
            })
            .into_any_element()
    }
}

fn task_kind_label(kind: TaskKind, strings: crate::i18n::NativeStrings) -> &'static str {
    match kind {
        TaskKind::Subagent => strings.subagent_task,
        TaskKind::Bash => strings.shell_task,
        TaskKind::Tool => strings.tool_task,
    }
}

fn task_status_label(status: TaskStatus, strings: crate::i18n::NativeStrings) -> &'static str {
    match status {
        TaskStatus::Running => strings.task_running,
        TaskStatus::Completed => strings.task_completed,
        TaskStatus::Failed => strings.task_failed,
        TaskStatus::Cancelled => strings.task_cancelled,
    }
}

fn status_color(status: TaskStatus) -> ColorToken {
    match status {
        TaskStatus::Failed => ERROR,
        TaskStatus::Running => ACCENT,
        TaskStatus::Completed | TaskStatus::Cancelled => TEXT_MUTED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_colors_keep_failures_visually_distinct() {
        assert_eq!(status_color(TaskStatus::Failed), ERROR);
        assert_ne!(status_color(TaskStatus::Running), ERROR);
    }
}
