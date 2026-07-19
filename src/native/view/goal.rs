use gpui::{AnyElement, Context, Role, div, prelude::*, px, relative};
use gpui_component::StyledExt;

use crate::native::app::Shell;
use crate::native::theme::*;
use crate::protocol::{GoalControl, GoalSnapshot};

impl Shell {
    pub(super) fn goal_strip(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let goal = self.model.active_goal()?.clone();
        let expanded = self.goals.is_expanded(&goal.goal_id);
        let goal_id = goal.goal_id.clone();
        let status_label = match goal.status.as_str() {
            "active" => self.strings.native.goal_active,
            "paused" => self.strings.native.goal_paused,
            "blocked" => self.strings.native.goal_blocked,
            _ => goal.status.as_str(),
        };
        let status_color = match goal.status.as_str() {
            "blocked" => ERROR,
            "paused" => TEXT_MUTED,
            _ => ACCENT,
        };
        let header = div()
            .id("goal-strip-toggle")
            .role(Role::Button)
            .aria_label(format!(
                "{}: {}",
                self.strings.native.goal_label, goal.objective
            ))
            .focusable()
            .tab_stop(true)
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.goals.toggle_expanded(&goal_id);
                cx.notify();
            }))
            .child(
                div()
                    .font_semibold()
                    .text_color(theme_rgb(status_color))
                    .child(self.strings.native.goal_label),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .line_clamp(1)
                    .child(goal.objective.clone()),
            )
            .child(
                div()
                    .rounded_full()
                    .bg(theme_rgb(SURFACE_ACTIVE))
                    .px_2()
                    .py_0p5()
                    .text_size(font_px(12.0))
                    .text_color(theme_rgb(status_color))
                    .child(status_label.to_owned()),
            )
            .children(goal.token_percent().map(|percent| {
                div()
                    .w(px(56.))
                    .h(px(4.))
                    .rounded_full()
                    .bg(theme_rgb(BORDER))
                    .child(
                        div()
                            .h_full()
                            .rounded_full()
                            .bg(theme_rgb(status_color))
                            .w(relative(f32::from(percent) / 100.)),
                    )
            }))
            .child(if expanded { "⌄" } else { "›" });

        let card = div()
            .id("goal-strip")
            .role(Role::Group)
            .aria_label(self.strings.native.goal_label)
            .w_full()
            .max_w(px(CONTENT_WIDTH - 32.))
            .rounded_lg()
            .border_1()
            .border_color(theme_rgb(BORDER))
            .bg(theme_rgb(SURFACE))
            .child(header)
            .when(expanded, |card| card.child(self.expanded_goal(&goal, cx)));

        Some(
            div()
                .w_full()
                .flex()
                .justify_center()
                .px_4()
                .pt_2()
                .child(card)
                .into_any_element(),
        )
    }

    fn expanded_goal(&self, goal: &GoalSnapshot, cx: &mut Context<Self>) -> AnyElement {
        let can_pause = goal.can_pause();
        let can_resume = goal.can_resume();
        div()
            .border_t_1()
            .border_color(theme_rgb(BORDER))
            .px_3()
            .py_3()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_size(font_px(13.0)).child(goal.objective.clone()))
            .children(goal.completion_criterion.as_ref().map(|criterion| {
                div()
                    .text_size(font_px(12.0))
                    .text_color(theme_rgb(TEXT_MUTED))
                    .child(format!("{} · {criterion}", self.strings.native.done_when))
            }))
            .children(goal.terminal_reason.as_ref().map(|reason| {
                div()
                    .text_size(font_px(12.0))
                    .text_color(theme_rgb(ERROR))
                    .child(reason.clone())
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(
                        div()
                            .text_size(font_px(12.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(format!(
                                "{} {} · {} {} · {}",
                                goal.turns_used,
                                self.strings.native.turns,
                                format_count(goal.tokens_used),
                                self.strings.native.tokens,
                                format_duration(goal.wall_clock_ms),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when(can_pause, |actions| {
                                actions.child(
                                    goal_action("pause-goal", self.strings.native.pause_goal)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.control_active_goal(GoalControl::Pause, cx)
                                        })),
                                )
                            })
                            .when(can_resume, |actions| {
                                actions.child(
                                    goal_action("resume-goal", self.strings.native.resume_goal)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.control_active_goal(GoalControl::Resume, cx)
                                        })),
                                )
                            })
                            .child(
                                goal_action("cancel-goal", self.strings.native.cancel_goal)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.confirm_cancel_goal(window, cx)
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }
}

fn goal_action(id: &'static str, label: &'static str) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .role(Role::Button)
        .aria_label(label)
        .focusable()
        .tab_stop(true)
        .cursor_pointer()
        .rounded_md()
        .border_1()
        .border_color(theme_rgb(BORDER))
        .px_2()
        .py_1()
        .text_size(font_px(12.0))
        .hover(|button| button.bg(theme_rgb(SURFACE_ACTIVE)))
        .child(label)
}

fn format_count(value: u64) -> String {
    if value >= 1_000_000 {
        format!("{:.1}m", value as f64 / 1_000_000.)
    } else if value >= 1_000 {
        format!("{:.1}k", value as f64 / 1_000.)
    } else {
        value.to_string()
    }
}

fn format_duration(milliseconds: u64) -> String {
    let seconds = milliseconds / 1_000;
    let minutes = seconds / 60;
    if minutes >= 60 {
        format!("{}h {}m", minutes / 60, minutes % 60)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds % 60)
    } else {
        format!("{seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_metrics_stay_compact() {
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(12_400), "12.4k");
        assert_eq!(format_duration(45_000), "45s");
        assert_eq!(format_duration(3_725_000), "1h 2m");
    }
}
