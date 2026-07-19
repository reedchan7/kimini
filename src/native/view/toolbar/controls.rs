use gpui::{Div, Role, SharedString, Stateful, div, prelude::*};

use crate::native::{
    app::{LoadState, Shell},
    theme::*,
};

impl Shell {
    pub(in crate::native) fn status_text(&self) -> String {
        match &self.state {
            LoadState::Connecting => self.strings.native.connecting.into(),
            LoadState::Ready => self.strings.native.connected.into(),
            LoadState::Working(message) | LoadState::Failed(message) => message.clone(),
        }
    }

    pub(super) fn connection_status_color(&self) -> ColorToken {
        match self.state {
            LoadState::Ready => SUCCESS,
            LoadState::Failed(_) => ERROR,
            LoadState::Connecting | LoadState::Working(_) => TEXT_MUTED,
        }
    }
}

pub(super) fn toolbar_button(label: impl Into<SharedString>, id: &'static str) -> Stateful<Div> {
    let label = label.into();
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(label.clone())
        .cursor_pointer()
        .rounded_md()
        .px_2()
        .py_1()
        .text_size(font_px(12.0))
        .hover(|item| item.bg(theme_rgb(SURFACE_ACTIVE)))
        .child(label)
}
