use gpui::{Anchor, AnyElement, Context, Role, div, prelude::*, px};
use gpui_component::{
    Disableable, Icon, IconName, Sizable as _, StyledExt,
    button::{Button, ButtonVariants},
    popover::Popover,
};

use crate::protocol::{PromptOptions, SessionStatus};

use super::super::app::{ComposerMenu, Shell};
use super::super::prompt_runtime::thinking_segments;
use super::super::theme::*;

impl Shell {
    pub(super) fn runtime_controls(&self, cx: &mut Context<Self>) -> (AnyElement, AnyElement) {
        let runtime = self
            .new_session_draft
            .is_none()
            .then(|| self.model.active_runtime())
            .flatten();
        let resolved = self
            .active_prompt_runtime()
            .map(|runtime| runtime.options(None));
        let current_model = runtime
            .and_then(|runtime| runtime.model.clone())
            .filter(|model| !model.is_empty())
            .or_else(|| resolved.as_ref().and_then(|options| options.model.clone()));
        let current_model_item = current_model
            .as_deref()
            .and_then(|model| self.models.iter().find(|item| item.model == model));
        let model_label = current_model_item
            .map(|item| item.label())
            .or(current_model.as_deref())
            .unwrap_or(self.strings.native.model)
            .to_owned();

        let current_thinking = runtime
            .map(|runtime| runtime.thinking_level.clone())
            .filter(|thinking| !thinking.is_empty())
            .or_else(|| {
                resolved
                    .as_ref()
                    .and_then(|options| options.thinking.clone())
            })
            .unwrap_or_else(|| self.strings.native.thinking.into());
        let efforts = thinking_segments(current_model_item);

        let current_permission = runtime
            .map(|runtime| runtime.permission.clone())
            .filter(|permission| !permission.is_empty())
            .or_else(|| {
                resolved
                    .as_ref()
                    .and_then(|options| options.permission_mode.clone())
            })
            .unwrap_or_else(|| "manual".into());
        let permission_label = match current_permission.as_str() {
            "manual" => self.strings.native.permission_manual,
            "auto" => self.strings.native.permission_auto,
            "yolo" => self.strings.native.permission_yolo,
            _ => self.strings.native.permission,
        };

        let (plan_enabled, swarm_enabled) = displayed_modes(runtime, resolved.as_ref());
        let goal_enabled = self.new_session_draft.is_none()
            && self
                .model
                .active_session()
                .is_some_and(|session| self.goals.is_armed(&session.id));
        let context_percent = runtime
            .map(|runtime| runtime.context_percent())
            .unwrap_or(0);

        let permission_open = self.composer_menu == Some(ComposerMenu::Permission);
        let modes_open = self.composer_menu == Some(ComposerMenu::Modes);
        let model_open = matches!(
            self.composer_menu,
            Some(ComposerMenu::Model | ComposerMenu::AllModels)
        );
        let all_models_open = self.composer_menu == Some(ComposerMenu::AllModels);

        let permission_trigger = Button::new("permission-control-button")
            .xsmall()
            .ghost()
            .text_color(theme_rgb(permission_color(&current_permission)))
            .label(permission_label);
        let permission = Popover::new("permission-control-popover")
            .anchor(Anchor::BottomLeft)
            .open(permission_open)
            .on_open_change(cx.listener(|this, open, _, cx| {
                this.set_composer_menu_open(ComposerMenu::Permission, *open, cx)
            }))
            .trigger(permission_trigger)
            .w(px(390.0))
            .p(px(5.0))
            .rounded_lg()
            .border_1()
            .border_color(theme_rgb(BORDER))
            .bg(theme_rgb(SURFACE))
            .shadow_sm()
            .child(self.permission_menu(&current_permission, cx));

        let modes_trigger = Button::new("modes-control-button")
            .xsmall()
            .ghost()
            .label(self.strings.native.modes)
            .when(plan_enabled, |button| {
                button.child(mode_tag(self.strings.native.mode_plan))
            })
            .when(swarm_enabled, |button| {
                button.child(mode_tag(self.strings.native.mode_swarm))
            })
            .when(goal_enabled, |button| {
                button.child(mode_tag(self.strings.native.mode_goal))
            });
        let modes = Popover::new("modes-control-popover")
            .anchor(Anchor::BottomLeft)
            .open(modes_open)
            .on_open_change(cx.listener(|this, open, _, cx| {
                this.set_composer_menu_open(ComposerMenu::Modes, *open, cx)
            }))
            .trigger(modes_trigger)
            .w(px(390.0))
            .p(px(5.0))
            .rounded_lg()
            .border_1()
            .border_color(theme_rgb(BORDER))
            .bg(theme_rgb(SURFACE))
            .shadow_sm()
            .child(self.modes_menu(plan_enabled, swarm_enabled, goal_enabled, cx));

        let left = div()
            .min_w_0()
            .flex()
            .items_center()
            .gap_1()
            .child(permission)
            .child(modes)
            .into_any_element();

        let models_empty = self.models.is_empty();
        let show_thinking = !matches!(current_thinking.as_str(), "off" | "");
        let model_trigger = Button::new("model-control-button")
            .xsmall()
            .ghost()
            .disabled(models_empty)
            .label(model_label)
            .child(
                div()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap_1()
                    .text_size(font_px(12.0))
                    .font_medium()
                    .when(show_thinking, |label| {
                        label.child(
                            div()
                                .text_color(theme_rgb(ACCENT))
                                .child(format!("· {current_thinking}")),
                        )
                    })
                    .child(
                        Icon::new(IconName::ChevronDown)
                            .xsmall()
                            .text_color(theme_rgb(TEXT_MUTED)),
                    ),
            );
        let model = Popover::new("model-control-popover")
            .anchor(Anchor::BottomRight)
            .open(model_open)
            .on_open_change(cx.listener(|this, open, _, cx| {
                if *open {
                    if !matches!(
                        this.composer_menu,
                        Some(ComposerMenu::Model | ComposerMenu::AllModels)
                    ) {
                        this.composer_menu = Some(ComposerMenu::Model);
                    }
                } else if matches!(
                    this.composer_menu,
                    Some(ComposerMenu::Model | ComposerMenu::AllModels)
                ) {
                    this.composer_menu = None;
                }
                cx.notify();
            }))
            .trigger(model_trigger)
            .w(px(if all_models_open { 360.0 } else { 220.0 }))
            .max_h(px(440.0))
            .p(px(5.0))
            .rounded_lg()
            .border_1()
            .border_color(theme_rgb(BORDER))
            .bg(theme_rgb(SURFACE))
            .shadow_sm()
            .child(self.model_menu(
                current_model.as_deref(),
                &current_thinking,
                &efforts,
                all_models_open,
                cx,
            ));

        let right = div()
            .min_w_0()
            .flex()
            .items_center()
            .gap_1()
            .when(self.new_session_draft.is_none(), |right| {
                right.child(
                    div()
                        .id("composer-context-meter")
                        .role(Role::Image)
                        .aria_label(format!(
                            "{}% {}",
                            context_percent, self.strings.native.context_label
                        ))
                        .size(px(12.0))
                        .flex_none()
                        .rounded_full()
                        .border_1()
                        .border_color(theme_rgb(if context_percent > 0 {
                            ACCENT
                        } else {
                            BORDER_STRONG
                        }))
                        .child(
                            div()
                                .m(px(3.0))
                                .size(px(4.0))
                                .rounded_full()
                                .bg(theme_rgb(SURFACE)),
                        ),
                )
            })
            .child(model)
            .into_any_element();

        (left, right)
    }

    fn set_composer_menu_open(&mut self, menu: ComposerMenu, open: bool, cx: &mut Context<Self>) {
        if open {
            self.composer_menu = Some(menu);
        } else if self.composer_menu == Some(menu) {
            self.composer_menu = None;
        }
        cx.notify();
    }

    fn permission_menu(&self, selected: &str, cx: &mut Context<Self>) -> impl IntoElement {
        let items = [
            (
                "manual",
                self.strings.native.permission_manual,
                self.strings.native.permission_manual_desc,
            ),
            (
                "yolo",
                self.strings.native.permission_yolo,
                self.strings.native.permission_yolo_desc,
            ),
            (
                "auto",
                self.strings.native.permission_auto,
                self.strings.native.permission_auto_desc,
            ),
        ];
        div()
            .id("permission-menu")
            .role(Role::Menu)
            .children(items.into_iter().enumerate().map(|(index, item)| {
                let (mode, label, description) = item;
                let active = selected == mode;
                let mode = mode.to_owned();
                let mode_for_click = mode.clone();
                div()
                    .id(("permission-option", index))
                    .focusable()
                    .tab_stop(true)
                    .role(Role::MenuItem)
                    .aria_label(format!("{label}. {description}"))
                    .cursor_pointer()
                    .w_full()
                    .flex()
                    .items_start()
                    .gap_2()
                    .rounded_md()
                    .px_2()
                    .py_2()
                    .when(active, |row| row.bg(theme_rgb(ACCENT_SOFT)))
                    .hover(|row| row.bg(theme_rgb(SURFACE_ACTIVE)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.composer_menu = None;
                        this.set_permission(mode_for_click.clone(), cx);
                    }))
                    .child(
                        div()
                            .w(px(14.0))
                            .h(px(18.0))
                            .flex_none()
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(active, |slot| {
                                slot.child(
                                    Icon::new(IconName::Check)
                                        .xsmall()
                                        .text_color(theme_rgb(ACCENT)),
                                )
                            }),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(font_px(12.0))
                                    .font_medium()
                                    .text_color(theme_rgb(permission_color(&mode)))
                                    .child(label),
                            )
                            .child(
                                div()
                                    .text_size(font_px(10.0))
                                    .text_color(theme_rgb(TEXT_MUTED))
                                    .child(description),
                            ),
                    )
            }))
    }

    fn modes_menu(
        &self,
        plan_enabled: bool,
        swarm_enabled: bool,
        goal_enabled: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("modes-menu")
            .role(Role::Menu)
            .child(self.mode_menu_row(
                0,
                IconName::File,
                self.strings.native.mode_plan,
                self.strings.native.mode_plan_desc,
                plan_enabled,
                cx.listener(|this, _, _, cx| this.toggle_plan_mode(cx)),
            ))
            .child(self.mode_menu_row(
                1,
                IconName::Asterisk,
                self.strings.native.mode_swarm,
                self.strings.native.mode_swarm_desc,
                swarm_enabled,
                cx.listener(|this, _, _, cx| this.toggle_swarm_mode(cx)),
            ))
            .child(self.mode_menu_row(
                2,
                IconName::CircleCheck,
                self.strings.native.mode_goal,
                self.strings.native.mode_goal_desc,
                goal_enabled,
                cx.listener(|this, _, _, cx| this.toggle_goal_mode(cx)),
            ))
    }

    fn mode_menu_row(
        &self,
        index: usize,
        icon: IconName,
        label: &'static str,
        description: &'static str,
        active: bool,
        listener: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    ) -> impl IntoElement {
        div()
            .id(("mode-option", index))
            .focusable()
            .tab_stop(true)
            .role(Role::MenuItem)
            .aria_label(format!("{label}. {description}"))
            .cursor_pointer()
            .w_full()
            .flex()
            .items_start()
            .gap_2()
            .rounded_md()
            .px_2()
            .py_2()
            .when(active, |row| row.bg(theme_rgb(ACCENT_SOFT)))
            .hover(|row| row.bg(theme_rgb(SURFACE_ACTIVE)))
            .on_click(listener)
            .child(
                div()
                    .w(px(14.0))
                    .h(px(18.0))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(icon).xsmall().text_color(theme_rgb(TEXT_MUTED))),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_size(font_px(12.0)).font_medium().child(label))
                    .child(
                        div()
                            .text_size(font_px(10.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(description),
                    ),
            )
            .child(toggle_switch(active))
    }

    fn model_menu(
        &self,
        selected_model: Option<&str>,
        selected_thinking: &str,
        efforts: &[String],
        show_all: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let current_provider = selected_model
            .and_then(|selected| self.models.iter().find(|model| model.model == selected))
            .map(|model| model.provider.clone());
        let models = self
            .models
            .iter()
            .filter(|model| {
                show_all
                    || current_provider
                        .as_ref()
                        .is_none_or(|provider| model.provider == *provider)
            })
            .cloned()
            .collect::<Vec<_>>();
        let thinking = self.strings.native.thinking;
        div()
            .id("model-menu")
            .role(Role::Menu)
            .max_h(px(430.0))
            .overflow_y_scroll()
            .when(show_all, |menu| {
                menu.child(
                    div()
                        .px_2()
                        .pt_1()
                        .pb_2()
                        .text_size(font_px(10.0))
                        .font_semibold()
                        .text_color(theme_rgb(TEXT_MUTED))
                        .child(self.strings.native.more_models),
                )
            })
            .when(!show_all, |menu| {
                menu.when_some(current_provider.clone(), |menu, provider| {
                    menu.child(
                        div()
                            .px_2()
                            .pt_1()
                            .pb_1()
                            .text_size(font_px(10.0))
                            .font_semibold()
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(provider.to_uppercase()),
                    )
                })
            })
            .children(models.into_iter().enumerate().map(|(index, model)| {
                let active = selected_model == Some(model.model.as_str());
                let model_id = model.model.clone();
                div()
                    .id(("model-option", index))
                    .focusable()
                    .tab_stop(true)
                    .role(Role::MenuItem)
                    .aria_label(model.label())
                    .cursor_pointer()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded_md()
                    .px_2()
                    .py_2()
                    .when(active, |row| row.bg(theme_rgb(ACCENT_SOFT)))
                    .hover(|row| row.bg(theme_rgb(SURFACE_ACTIVE)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.composer_menu = None;
                        this.set_model(model_id.clone(), cx);
                    }))
                    .child(div().w(px(14.0)).flex_none().when(active, |slot| {
                        slot.child(
                            Icon::new(IconName::Check)
                                .xsmall()
                                .text_color(theme_rgb(ACCENT)),
                        )
                    }))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .text_size(font_px(12.0))
                            .font_medium()
                            .line_clamp(1)
                            .child(model.label().to_owned()),
                    )
                    .when(show_all, |row| {
                        row.child(
                            div()
                                .flex_none()
                                .text_size(font_px(10.0))
                                .text_color(theme_rgb(TEXT_MUTED))
                                .child(model.provider),
                        )
                    })
            }))
            .when(!show_all, |menu| {
                menu.child(div().my_1().border_t_1().border_color(theme_rgb(BORDER)))
                    .child(
                        div()
                            .px_2()
                            .py_2()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_3()
                            .child(div().text_size(font_px(12.0)).font_medium().child(thinking))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .rounded_md()
                                    .bg(theme_rgb(SURFACE_ACTIVE))
                                    .p(px(2.0))
                                    .children(efforts.iter().enumerate().map(|(index, effort)| {
                                        let effort_value = effort.clone();
                                        let active = selected_thinking == effort;
                                        div()
                                            .id(("thinking-option", index))
                                            .focusable()
                                            .tab_stop(true)
                                            .role(Role::MenuItem)
                                            .aria_label(format!("{thinking} {effort}"))
                                            .cursor_pointer()
                                            .rounded_sm()
                                            .px_2()
                                            .py_1()
                                            .text_size(font_px(10.0))
                                            .text_color(theme_rgb(if active {
                                                TEXT
                                            } else {
                                                TEXT_MUTED
                                            }))
                                            .when(active, |item| {
                                                item.bg(theme_rgb(SURFACE)).shadow_sm()
                                            })
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.set_thinking(effort_value.clone(), cx)
                                            }))
                                            .child(effort_label(effort))
                                    })),
                            ),
                    )
                    .child(div().my_1().border_t_1().border_color(theme_rgb(BORDER)))
                    .child(
                        div()
                            .id("more-models-option")
                            .focusable()
                            .tab_stop(true)
                            .role(Role::MenuItem)
                            .aria_label(self.strings.native.more_models)
                            .cursor_pointer()
                            .rounded_md()
                            .px_2()
                            .py_2()
                            .text_size(font_px(12.0))
                            .font_medium()
                            .text_color(theme_rgb(ACCENT))
                            .hover(|row| row.bg(theme_rgb(ACCENT_SOFT)))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.composer_menu = Some(ComposerMenu::AllModels);
                                cx.notify();
                            }))
                            .child(self.strings.native.more_models),
                    )
            })
    }
}

fn permission_color(mode: &str) -> ColorToken {
    match mode {
        "yolo" => WARNING,
        "auto" => ERROR,
        _ => TEXT_SECONDARY,
    }
}

fn displayed_modes(
    runtime: Option<&SessionStatus>,
    resolved: Option<&PromptOptions>,
) -> (bool, bool) {
    let plan = runtime
        .map(|runtime| runtime.plan_mode)
        .or_else(|| resolved.and_then(|options| options.plan_mode))
        .unwrap_or(false);
    let swarm = runtime
        .map(|runtime| runtime.swarm_mode)
        .or_else(|| resolved.and_then(|options| options.swarm_mode))
        .unwrap_or(false);
    (plan, swarm)
}

fn mode_tag(label: &'static str) -> impl IntoElement {
    div()
        .rounded_full()
        .border_1()
        .border_color(theme_rgb(ACCENT))
        .bg(theme_rgb(SURFACE))
        .px_1()
        .text_size(font_px(9.0))
        .text_color(theme_rgb(ACCENT))
        .child(label)
}

fn toggle_switch(active: bool) -> impl IntoElement {
    div()
        .mt(px(1.0))
        .w(px(30.0))
        .h(px(18.0))
        .flex_none()
        .flex()
        .items_center()
        .when(active, |switch| switch.justify_end().bg(theme_rgb(ACCENT)))
        .when(!active, |switch| {
            switch.justify_start().bg(theme_rgb(BORDER_STRONG))
        })
        .rounded_full()
        .p(px(2.0))
        .child(div().size(px(14.0)).rounded_full().bg(theme_rgb(SURFACE)))
}

fn effort_label(effort: &str) -> String {
    let mut chars = effort.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_modes_use_the_resolved_draft_values() {
        let options = PromptOptions {
            plan_mode: Some(true),
            swarm_mode: Some(true),
            ..PromptOptions::default()
        };

        assert_eq!(displayed_modes(None, Some(&options)), (true, true));
    }

    #[test]
    fn active_session_modes_keep_live_status_authoritative() {
        let status = SessionStatus {
            busy: false,
            model: None,
            thinking_level: "off".into(),
            permission: "manual".into(),
            plan_mode: false,
            swarm_mode: false,
            context_tokens: 0,
            max_context_tokens: 0,
            context_usage: 0.0,
        };
        let fallback = PromptOptions {
            plan_mode: Some(true),
            swarm_mode: Some(true),
            ..PromptOptions::default()
        };

        assert_eq!(
            displayed_modes(Some(&status), Some(&fallback)),
            (false, false)
        );
    }
}
