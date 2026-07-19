use gpui::{ElementId, Role, div, prelude::*};

use super::super::theme::{SURFACE_ACTIVE, font_px, theme_rgb};

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
        .text_size(font_px(12.0))
        .hover(|button| button.bg(theme_rgb(SURFACE_ACTIVE)))
        .child(label)
}
