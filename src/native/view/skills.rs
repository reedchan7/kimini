use gpui::{Context, IntoElement, Role, div, prelude::*, px, rgb};
use gpui_component::{StyledExt, scroll::ScrollableElement};

use crate::native::{app::Shell, theme::*};

use super::panel::panel_button;

impl Shell {
    pub(super) fn skill_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("skill-panel")
            .role(Role::Group)
            .aria_label(self.strings.native.skills_panel)
            .w(px(TASK_PANEL_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SURFACE))
            .child(self.skill_panel_header(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .p_3()
                    .when_some(self.skills.error.clone(), |panel, error| {
                        panel.child(
                            div()
                                .mb_3()
                                .rounded_md()
                                .border_1()
                                .border_color(rgb(ERROR))
                                .p_2()
                                .text_xs()
                                .text_color(rgb(ERROR))
                                .child(error),
                        )
                    })
                    .when_some(self.skills.activated.clone(), |panel, name| {
                        panel.child(
                            div()
                                .mb_3()
                                .rounded_md()
                                .bg(rgb(SURFACE_ACTIVE))
                                .p_2()
                                .text_xs()
                                .child(format!("{}: {name}", self.strings.native.skill_activated)),
                        )
                    })
                    .when(self.skills.items.is_empty(), |panel| {
                        panel.child(
                            div()
                                .py_8()
                                .text_center()
                                .text_sm()
                                .text_color(rgb(TEXT_MUTED))
                                .child(if self.skills.loading {
                                    self.strings.native.skills_loading
                                } else {
                                    self.strings.native.no_skills
                                }),
                        )
                    })
                    .children(self.skills.items.iter().enumerate().map(|(index, skill)| {
                        let name = skill.name.clone();
                        let activating = self.skills.activating.as_deref() == Some(&skill.name);
                        div()
                            .id(("skill-card", index))
                            .role(Role::Article)
                            .aria_label(format!("{}: {}", skill.name, skill.description))
                            .mb_2()
                            .rounded_lg()
                            .border_1()
                            .border_color(rgb(BORDER))
                            .bg(rgb(CANVAS))
                            .p_3()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap_2()
                                    .child(
                                        div()
                                            .min_w_0()
                                            .flex_1()
                                            .text_sm()
                                            .font_semibold()
                                            .line_clamp(1)
                                            .child(skill.name.clone()),
                                    )
                                    .child(
                                        panel_button(
                                            if activating {
                                                self.strings.native.activating_skill
                                            } else {
                                                self.strings.native.activate_skill
                                            },
                                            ("activate-skill", index),
                                        )
                                        .when(
                                            !activating,
                                            |button| {
                                                button.on_click(cx.listener(
                                                    move |this, _, _, cx| {
                                                        this.activate_skill(name.clone(), None, cx)
                                                    },
                                                ))
                                            },
                                        ),
                                    ),
                            )
                            .when(!skill.description.is_empty(), |card| {
                                card.child(
                                    div()
                                        .mt_2()
                                        .text_xs()
                                        .text_color(rgb(TEXT_MUTED))
                                        .child(skill.description.clone()),
                                )
                            })
                            .child(
                                div()
                                    .mt_2()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child(if skill.source.is_empty() {
                                        "—".to_owned()
                                    } else {
                                        skill.source.clone()
                                    })
                                    .when(skill.disable_model_invocation, |meta| {
                                        meta.child(self.strings.native.slash_only)
                                    }),
                            )
                    })),
            )
    }

    fn skill_panel_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(48.0))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_sm()
                    .font_semibold()
                    .child(self.strings.native.skills)
                    .child(
                        div()
                            .text_xs()
                            .font_normal()
                            .text_color(rgb(TEXT_MUTED))
                            .child(self.skills.items.len().to_string()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        panel_button(self.strings.native.refresh_skills, "refresh-skills")
                            .on_click(cx.listener(|this, _, _, cx| this.refresh_skills(cx))),
                    )
                    .child(
                        panel_button(self.strings.native.close_skills, "close-skills")
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_skill_panel(cx))),
                    ),
            )
    }
}
