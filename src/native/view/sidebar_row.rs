use gpui::{AnyElement, Context, Role, div, prelude::*, px, rgb};
use gpui_component::{Icon, IconName, Sizable as _, StyledExt};

use super::super::app::Shell;
use super::super::session_list::{SessionListRow, SidebarSession};
use super::super::theme::*;

impl Shell {
    pub(super) fn sidebar_list_row(
        &self,
        index: usize,
        visible_count: usize,
        active: Option<&str>,
        archived: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.session_list.row(index).cloned() {
            Some(SessionListRow::Workspace {
                key,
                label,
                collapsed,
            }) => self.workspace_row(index, key, label, collapsed, cx),
            Some(SessionListRow::Session(session)) => {
                self.sidebar_session_row(session, visible_count, active, archived, cx)
            }
            Some(SessionListRow::ShowMore {
                workspace_key,
                remaining,
                expanded,
            }) => self.sidebar_show_more_row(index, workspace_key, remaining, expanded, cx),
            None => div().into_any_element(),
        }
    }

    fn sidebar_session_row(
        &self,
        session: SidebarSession,
        visible_count: usize,
        active: Option<&str>,
        archived: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let session_id = session.id.clone();
        let selected = active == Some(session.id.as_str());
        let title = if session.title.is_empty() {
            self.strings.native.untitled_session.to_owned()
        } else {
            session.title
        };
        let row = div()
            .id(("session", session.position))
            .focusable()
            .tab_stop(true)
            .role(Role::ListItem)
            .aria_label(format!("{} — {}", title, session.cwd))
            .aria_selected(selected)
            .aria_position_in_set(session.position + 1)
            .aria_size_of_set(visible_count)
            .rounded_md()
            .px_2()
            .py_1()
            .when(selected, |item| item.bg(rgb(SURFACE_ACTIVE)))
            .hover(|item| item.bg(rgb(SURFACE_ACTIVE)));

        if archived {
            row.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(div().min_w_0().text_sm().line_clamp(1).child(title))
                    .child(
                        div()
                            .id(("restore-session", session.position))
                            .focusable()
                            .tab_stop(true)
                            .role(Role::Button)
                            .aria_label(self.strings.native.restore_session)
                            .cursor_pointer()
                            .rounded_md()
                            .px_2()
                            .py_1()
                            .text_xs()
                            .text_color(rgb(ACCENT))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.restore_archived_session(session_id.clone(), cx)
                            }))
                            .child(self.strings.native.restore_session),
                    ),
            )
            .into_any_element()
        } else {
            row.cursor_pointer()
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.select_session(session_id.clone(), window, cx)
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .size(px(5.0))
                                .flex_none()
                                .rounded_full()
                                .bg(rgb(if session.busy { ACCENT } else { TEXT_MUTED })),
                        )
                        .child(
                            div()
                                .min_w_0()
                                .flex_1()
                                .text_size(px(12.0))
                                .font_weight(if selected {
                                    gpui::FontWeight::MEDIUM
                                } else {
                                    gpui::FontWeight::NORMAL
                                })
                                .text_color(rgb(if selected { TEXT } else { TEXT_SECONDARY }))
                                .line_clamp(1)
                                .child(title),
                        ),
                )
                .when(session.busy, |item| {
                    item.child(
                        div()
                            .pl(px(13.0))
                            .text_size(px(10.0))
                            .text_color(rgb(ACCENT))
                            .child(self.strings.native.working),
                    )
                })
                .into_any_element()
        }
    }

    fn workspace_row(
        &self,
        index: usize,
        workspace_key: String,
        label: String,
        collapsed: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .id(("workspace-heading", index))
            .focusable()
            .tab_stop(true)
            .role(Role::Button)
            .aria_label(label.clone())
            .aria_expanded(!collapsed)
            .cursor_pointer()
            .h(px(36.0))
            .px_2()
            .flex()
            .items_center()
            .gap_2()
            .rounded_md()
            .text_size(px(11.0))
            .font_semibold()
            .text_color(rgb(TEXT_SECONDARY))
            .hover(|row| row.bg(rgb(SURFACE_ACTIVE)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.session_list.toggle_workspace(&workspace_key);
                cx.notify();
            }))
            .child(
                Icon::new(if collapsed {
                    IconName::FolderClosed
                } else {
                    IconName::FolderOpen
                })
                .xsmall()
                .text_color(rgb(TEXT_MUTED)),
            )
            .child(div().min_w_0().line_clamp(1).child(label))
            .into_any_element()
    }

    fn sidebar_show_more_row(
        &self,
        index: usize,
        workspace_key: String,
        remaining: usize,
        expanded: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let label = if expanded {
            self.strings.native.show_less_conversations.to_owned()
        } else {
            self.strings
                .native
                .show_more_conversations
                .replace("{count}", &remaining.to_string())
        };
        div()
            .id(("show-more-workspace-sessions", index))
            .focusable()
            .tab_stop(true)
            .role(Role::Button)
            .aria_label(label.clone())
            .aria_expanded(expanded)
            .cursor_pointer()
            .h(px(28.0))
            .pl(px(22.0))
            .pr_2()
            .flex()
            .items_center()
            .rounded_md()
            .text_size(px(10.0))
            .text_color(rgb(TEXT_MUTED))
            .hover(|row| row.bg(rgb(SURFACE_ACTIVE)).text_color(rgb(TEXT_SECONDARY)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.session_list.toggle_expanded(&workspace_key);
                cx.notify();
            }))
            .child(label)
            .into_any_element()
    }
}
