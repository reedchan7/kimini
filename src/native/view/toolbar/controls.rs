use gpui::{Context, Div, Role, SharedString, Stateful, div, prelude::*, rgb};
use gpui_component::{
    IconName, Sizable as _,
    button::{Button, ButtonVariants},
};

use crate::native::{
    app::{LoadState, Shell},
    theme::*,
};

impl Shell {
    pub(super) fn browser_button(&self, cx: &mut Context<Self>) -> Button {
        let label = if self.browser.is_some() {
            self.strings.native.close_browser
        } else {
            self.strings.native.browser
        };
        Button::new("browser-toggle")
            .xsmall()
            .ghost()
            .icon(IconName::Globe)
            .tooltip(label)
            .on_click(cx.listener(|this, _, window, cx| this.toggle_browser(window, cx)))
    }

    pub(super) fn status_text(&self) -> String {
        match &self.state {
            LoadState::Connecting => self.strings.native.connecting.into(),
            LoadState::Ready => self.strings.native.connected.into(),
            LoadState::Working(message) | LoadState::Failed(message) => message.clone(),
        }
    }

    pub(super) fn connection_status_color(&self) -> u32 {
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
        .text_xs()
        .hover(|item| item.bg(rgb(SURFACE_ACTIVE)))
        .child(label)
}
