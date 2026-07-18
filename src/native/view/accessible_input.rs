use gpui::{
    AccessibleAction, App, ElementId, Entity, Role, SharedString, StatefulInteractiveElement, div,
    prelude::*,
};
use gpui_component::input::{Input, InputEvent, InputState};

pub(super) fn accessible_input(
    id: impl Into<ElementId>,
    state: &Entity<InputState>,
    role: Role,
    label: impl Into<SharedString>,
    placeholder: impl Into<SharedString>,
    input: Input,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    let value = state.read(cx).value();
    let state_for_value = state.clone();
    div()
        .id(id)
        .role(role)
        .aria_label(label)
        .aria_value(value)
        .aria_placeholder(placeholder)
        .on_a11y_action(AccessibleAction::SetValue, move |data, window, cx| {
            let Some(gpui::accesskit::ActionData::Value(value)) = data else {
                return;
            };
            state_for_value.update(cx, |state, cx| {
                state.set_value(value.as_ref(), window, cx);
                cx.emit(InputEvent::Change);
                state.focus(window, cx);
            });
        })
        .child(input.role(Role::Group))
}
