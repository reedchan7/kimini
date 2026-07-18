use gpui::{AnyElement, Context, div, prelude::*};
use gpui_component::{
    Disableable, Selectable, Sizable as _,
    button::{Button, ButtonVariants},
    menu::DropdownMenu,
};

use super::super::app::{Shell, UtilityPanel};
use super::super::{SetModel, SetPermission, SetThinking};

impl Shell {
    pub(super) fn runtime_controls(&self, cx: &mut Context<Self>) -> AnyElement {
        let runtime = self.model.active_runtime();
        let resolved = self
            .active_prompt_runtime()
            .map(|runtime| runtime.options(None));
        let current_model = runtime
            .and_then(|runtime| runtime.model.clone())
            .filter(|model| !model.is_empty())
            .or_else(|| resolved.as_ref().and_then(|options| options.model.clone()));
        let model_label = current_model
            .as_deref()
            .and_then(|model| {
                self.models
                    .iter()
                    .find(|item| item.model == model)
                    .map(|item| item.label())
            })
            .or(current_model.as_deref())
            .unwrap_or(self.strings.native.model)
            .to_owned();
        let models = self.models.clone();
        let selected_model = current_model.clone();
        let current_thinking = runtime
            .map(|runtime| runtime.thinking_level.clone())
            .filter(|thinking| !thinking.is_empty())
            .or_else(|| {
                resolved
                    .as_ref()
                    .and_then(|options| options.thinking.clone())
            })
            .unwrap_or_else(|| self.strings.native.thinking.into());
        let mut efforts = current_model
            .as_deref()
            .and_then(|model| self.models.iter().find(|item| item.model == model))
            .map(|model| model.support_efforts.clone())
            .unwrap_or_default();
        if efforts.is_empty() {
            efforts = vec!["off".into(), "on".into()];
        }
        if !efforts.iter().any(|effort| effort == "off") {
            efforts.insert(0, "off".into());
        }
        let selected_thinking = current_thinking.clone();
        let current_permission = runtime
            .map(|runtime| runtime.permission.clone())
            .filter(|permission| !permission.is_empty())
            .or_else(|| {
                resolved
                    .as_ref()
                    .and_then(|options| options.permission_mode.clone())
            })
            .unwrap_or_else(|| self.strings.native.permission.into());
        let selected_permission = current_permission.clone();
        let plan_enabled = runtime.is_some_and(|runtime| runtime.plan_mode);
        let swarm_enabled = runtime.is_some_and(|runtime| runtime.swarm_mode);
        let goal_armed = self
            .model
            .active_session()
            .is_some_and(|session| self.goals.is_armed(&session.id));
        let has_goal = self.model.active_goal().is_some();

        div()
            .min_w_0()
            .flex()
            .items_center()
            .gap_1()
            .flex_wrap()
            .child(
                Button::new("model-control-button")
                    .xsmall()
                    .ghost()
                    .disabled(models.is_empty())
                    .label(model_label)
                    .dropdown_menu(move |menu, _, _| {
                        models.iter().fold(menu, |menu, model| {
                            menu.menu_with_check(
                                model.label(),
                                selected_model.as_deref() == Some(model.model.as_str()),
                                Box::new(SetModel {
                                    model: model.model.clone(),
                                }),
                            )
                        })
                    }),
            )
            .child(
                Button::new("thinking-control-button")
                    .xsmall()
                    .ghost()
                    .label(current_thinking)
                    .dropdown_menu(move |menu, _, _| {
                        efforts.iter().fold(menu, |menu, effort| {
                            menu.menu_with_check(
                                effort.clone(),
                                selected_thinking == *effort,
                                Box::new(SetThinking {
                                    effort: effort.clone(),
                                }),
                            )
                        })
                    }),
            )
            .child(
                Button::new("permission-control-button")
                    .xsmall()
                    .ghost()
                    .label(current_permission)
                    .dropdown_menu(move |menu, _, _| {
                        ["manual", "auto", "yolo"]
                            .into_iter()
                            .fold(menu, |menu, mode| {
                                menu.menu_with_check(
                                    mode,
                                    selected_permission == mode,
                                    Box::new(SetPermission { mode: mode.into() }),
                                )
                            })
                    }),
            )
            .child(
                Button::new("plan-control")
                    .xsmall()
                    .ghost()
                    .label(if plan_enabled {
                        self.strings.native.plan_on
                    } else {
                        self.strings.native.plan_off
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_plan_mode(cx))),
            )
            .child(
                Button::new("swarm-control")
                    .xsmall()
                    .ghost()
                    .label(if swarm_enabled {
                        self.strings.native.swarm_on
                    } else {
                        self.strings.native.swarm_off
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_swarm_mode(cx))),
            )
            .child(
                Button::new("goal-mode-control")
                    .xsmall()
                    .ghost()
                    .disabled(has_goal)
                    .label(if goal_armed {
                        self.strings.native.goal_mode_on
                    } else {
                        self.strings.native.goal_mode_off
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_goal_mode(cx))),
            )
            .child(
                Button::new("side-chat-control")
                    .xsmall()
                    .ghost()
                    .label(self.strings.native.side_chat)
                    .selected(self.utility_panel == Some(UtilityPanel::SideChat))
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_side_chat(window, cx))),
            )
            .into_any_element()
    }
}
