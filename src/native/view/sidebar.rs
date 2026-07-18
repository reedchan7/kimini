use gpui::{Context, IntoElement, Role, div, prelude::*, px, rgb};

use super::super::app::Shell;
use super::super::theme::*;

impl Shell {
    pub(super) fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let sessions = self.model.sessions();
        let active = self
            .model
            .active_session()
            .map(|session| session.id.as_str());
        div()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .p_3()
            .gap_1()
            .border_r_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SIDEBAR))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_2()
                    .py_1()
                    .text_xs()
                    .text_color(rgb(TEXT_MUTED))
                    .child(
                        div()
                            .id("sessions-heading")
                            .role(Role::Heading)
                            .aria_level(2)
                            .aria_label("Sessions")
                            .child("SESSIONS"),
                    )
                    .child(
                        div()
                            .id("new-session")
                            .focusable()
                            .tab_stop(true)
                            .role(Role::Button)
                            .aria_label("New session")
                            .cursor_pointer()
                            .rounded_md()
                            .px_2()
                            .py_1()
                            .hover(|item| item.bg(rgb(SURFACE_ACTIVE)))
                            .on_click(cx.listener(|this, _, _, cx| this.create_session(cx)))
                            .child("+ NEW"),
                    ),
            )
            .child(
                div()
                    .id("session-list")
                    .role(Role::List)
                    .aria_label("Kimi Code sessions")
                    .flex()
                    .flex_col()
                    .children(sessions.iter().enumerate().map(|(index, session)| {
                        let session_id = session.id.clone();
                        let selected = active == Some(session.id.as_str());
                        div()
                            .id(("session", index))
                            .focusable()
                            .tab_stop(true)
                            .role(Role::ListItem)
                            .aria_label(format!("{} — {}", session.title, session.metadata.cwd))
                            .aria_selected(selected)
                            .aria_position_in_set(index + 1)
                            .aria_size_of_set(sessions.len())
                            .cursor_pointer()
                            .rounded_md()
                            .px_2()
                            .py_2()
                            .when(selected, |item| item.bg(rgb(SURFACE_ACTIVE)))
                            .hover(|item| item.bg(rgb(SURFACE_ACTIVE)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.select_session(session_id.clone(), cx)
                            }))
                            .child(div().text_sm().line_clamp(1).child(session.title.clone()))
                            .child(
                                div()
                                    .mt_1()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .line_clamp(1)
                                    .child(session.metadata.cwd.clone()),
                            )
                    })),
            )
    }
}
