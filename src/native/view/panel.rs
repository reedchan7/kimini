use gpui::{ElementId, Role, div, prelude::*, rgb};

use super::super::theme::SURFACE_ACTIVE;

pub(super) fn panel_button(
    label: &'static str,
    id: impl Into<ElementId>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(label)
        .cursor_pointer()
        .rounded_md()
        .px_2()
        .py_1()
        .text_xs()
        .hover(|button| button.bg(rgb(SURFACE_ACTIVE)))
        .child(label)
}
