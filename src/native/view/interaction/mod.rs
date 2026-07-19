mod approval;
mod question;

use gpui::{Context, IntoElement, Role, div, prelude::*};

use super::super::app::Shell;
use super::super::theme::*;

impl Shell {
    pub(super) fn interactions(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let (approvals, questions) = self
            .model
            .active_conversation()
            .map(|conversation| {
                (
                    conversation.approvals.clone(),
                    conversation.questions.clone(),
                )
            })
            .unwrap_or_default();
        let cards = approval::render(self, cx, approvals)
            .into_iter()
            .chain(question::render(self, cx, questions))
            .collect::<Vec<_>>();

        div().flex_none().px_3().children(cards)
    }
}

fn action_button(
    label: impl Into<gpui::SharedString>,
    id: impl Into<gpui::ElementId>,
) -> gpui::Stateful<gpui::Div> {
    let label = label.into();
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(label.clone())
        .cursor_pointer()
        .rounded_md()
        .border_1()
        .border_color(theme_rgb(BORDER))
        .px_3()
        .py_1()
        .text_size(font_px(13.0))
        .child(label)
}
