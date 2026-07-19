use gpui::{IntoElement, Role, SharedString, div, prelude::*, px};
use gpui_component::{Icon, IconName, Sizable as _, StyledExt};

use super::super::theme::*;

pub(super) fn settings_section(label: &'static str) -> gpui::Div {
    div().flex().flex_col().child(
        div()
            .mb_3()
            .text_size(font_px(10.0))
            .font_semibold()
            .text_color(theme_rgb(TEXT_MUTED))
            .child(label.to_uppercase()),
    )
}

pub(super) fn settings_config_error(error: impl Into<SharedString>) -> gpui::Stateful<gpui::Div> {
    div()
        .id("settings-config-error")
        .mb_3()
        .role(Role::Status)
        .rounded_md()
        .border_1()
        .border_color(theme_rgb(ERROR))
        .p_2()
        .text_size(font_px(12.0))
        .text_color(theme_rgb(ERROR))
        .child(error.into())
}

pub(super) fn next_model_id(
    models: &[crate::protocol::ModelCatalogItem],
    current: Option<&str>,
) -> Option<String> {
    let first = models.first()?;
    let next = models
        .iter()
        .position(|item| Some(item.model.as_str()) == current)
        .map(|index| &models[(index + 1) % models.len()])
        .unwrap_or(first);
    Some(next.model.clone())
}

pub(super) fn disabled_settings_control(
    control: gpui::Stateful<gpui::Div>,
) -> gpui::Stateful<gpui::Div> {
    control.cursor_default().opacity(0.55)
}

pub(super) fn settings_row(
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
) -> gpui::Div {
    div()
        .min_h(px(40.0))
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .border_b_1()
        .border_color(theme_rgb(BORDER))
        .text_size(font_px(13.0))
        .child(div().min_w_0().child(label.into()))
        .child(
            div()
                .flex_none()
                .text_size(font_px(12.0))
                .text_color(theme_rgb(TEXT_SECONDARY))
                .child(value.into()),
        )
}

pub(super) fn settings_action_row(
    label: impl Into<SharedString>,
    action: impl IntoElement,
) -> gpui::Div {
    div()
        .min_h(px(40.0))
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .border_b_1()
        .border_color(theme_rgb(BORDER))
        .text_size(font_px(13.0))
        .child(div().min_w_0().flex_1().child(label.into()))
        .child(div().flex_none().child(action))
}

pub(super) fn settings_labeled_action_row(
    label: impl Into<SharedString>,
    description: Option<&'static str>,
    action: impl IntoElement,
) -> gpui::Div {
    let row_height = if description.is_some() { 64.0 } else { 40.0 };
    settings_labeled_action_row_with_height(label, description, action, row_height)
}

pub(super) fn settings_agent_action_row(
    label: impl Into<SharedString>,
    description: Option<&'static str>,
    action: impl IntoElement,
) -> gpui::Div {
    settings_labeled_action_row_with_height(label, description, action, 52.0)
}

fn settings_labeled_action_row_with_height(
    label: impl Into<SharedString>,
    description: Option<&'static str>,
    action: impl IntoElement,
    row_height: f32,
) -> gpui::Div {
    let label = label.into();
    div()
        .min_h(px(row_height))
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .border_b_1()
        .border_color(theme_rgb(BORDER))
        .child(
            div()
                .min_w_0()
                .flex_1()
                .flex()
                .flex_col()
                .text_size(font_px(13.0))
                .child(label)
                .when_some(description, |item, description| {
                    item.child(
                        div()
                            .mt_0p5()
                            .text_size(font_px(11.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(description),
                    )
                }),
        )
        .child(div().flex_none().child(action))
}

pub(super) fn settings_segment(
    id: &'static str,
    label: &'static str,
    selected: bool,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(label)
        .aria_selected(selected)
        .cursor_pointer()
        .flex_1()
        .h(px(30.0))
        .px_3()
        .flex()
        .items_center()
        .justify_center()
        .border_1()
        .border_color(theme_rgb(BORDER_STRONG))
        .bg(theme_rgb(if selected { SURFACE } else { SURFACE_SUBTLE }))
        .text_size(font_px(11.0))
        .text_color(theme_rgb(if selected { TEXT } else { TEXT_SECONDARY }))
        .when(selected, |item| item.shadow_sm())
        .hover(|item| {
            item.bg(theme_rgb(SURFACE_ACTIVE))
                .text_color(theme_rgb(TEXT))
        })
        .child(label)
}

pub(super) fn settings_stepper_button(
    id: &'static str,
    label: &'static str,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(label)
        .cursor_pointer()
        .flex_none()
        .w(px(28.0))
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .text_size(font_px(13.0))
        .text_color(theme_rgb(TEXT_SECONDARY))
        .hover(|item| {
            item.bg(theme_rgb(SURFACE_ACTIVE))
                .text_color(theme_rgb(TEXT))
        })
        .child(label)
}

pub(super) fn settings_select(
    id: &'static str,
    label: impl Into<SharedString>,
) -> gpui::Stateful<gpui::Div> {
    let label = label.into();
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(label.clone())
        .cursor_pointer()
        .h(px(30.0))
        .min_w(px(222.0))
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .rounded_md()
        .border_1()
        .border_color(theme_rgb(BORDER_STRONG))
        .bg(theme_rgb(SURFACE))
        .px_3()
        .text_size(font_px(11.0))
        .text_color(theme_rgb(TEXT_SECONDARY))
        .hover(|item| {
            item.bg(theme_rgb(SURFACE_ACTIVE))
                .text_color(theme_rgb(TEXT))
        })
        .child(label)
        .child(Icon::new(IconName::ChevronDown).xsmall())
}

pub(super) fn settings_toggle(
    id: &'static str,
    label: &'static str,
    active: bool,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .role(Role::Switch)
        .aria_label(label)
        .aria_selected(active)
        .cursor_pointer()
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
