use gpui::{AnyElement, ClipboardItem, Context, Role, div, prelude::*, px};
use gpui_component::{
    Icon, IconName, Sizable as _, StyledExt,
    button::{Button, ButtonVariants},
    input::Input,
    menu::{DropdownMenu, PopupMenuItem},
};

use super::super::app::Shell;
use super::super::session_list::{SessionListRow, SidebarSession, created_at_label, relative_time};
use super::super::shell::{
    SESSION_ARCHIVE_ICON_PATH, SESSION_EXPORT_ICON_PATH, SESSION_FORK_ICON_PATH,
    SESSION_RENAME_ICON_PATH,
};
use super::super::theme::*;
use super::accessible_input::accessible_input;

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
                root,
                collapsed,
            }) => self.workspace_row(index, key, label, root, collapsed, cx),
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
        let renaming = self.renaming_session_id.as_deref() == Some(session.id.as_str());
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
            .py_2()
            .when(selected, |item| item.bg(theme_rgb(SURFACE_ACTIVE)))
            .hover(|item| item.bg(theme_rgb(SURFACE_ACTIVE)));

        if archived {
            row.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .min_w_0()
                            .text_size(ui_font_px())
                            .line_clamp(1)
                            .child(title),
                    )
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
                            .text_size(font_px(12.0))
                            .text_color(theme_rgb(ACCENT))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.restore_archived_session(session_id.clone(), cx)
                            }))
                            .child(self.strings.native.restore_session),
                    ),
            )
            .into_any_element()
        } else {
            let shell = cx.entity();
            let menu_session_id = session.id.clone();
            let menu_title = title.clone();
            let created_at = created_at_label(&session.created_at);
            let strings = self.strings.native;
            let row_group = format!("session-row-{}", session.position);
            let actions_group = row_group.clone();
            let time_label = relative_time(
                &session.updated_at,
                self.strings.native.session_just_now,
            );
            row.w_full()
                .group(row_group)
                .cursor_pointer()
                .child(
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .id(("select-session", session.position))
                                .flex_1()
                                .min_w_0()
                                .focusable()
                                .tab_stop(true)
                                .role(Role::Button)
                                .aria_label(title.clone())
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .gap_1()
                                .when(!renaming, |row| {
                                    row.on_click(cx.listener(move |this, _, window, cx| {
                                        this.select_session(session_id.clone(), window, cx)
                                    }))
                                })
                                .child(
                                    div()
                                        .w(px(16.0))
                                        .h(px(12.0))
                                        .flex_none()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .when(session.busy, |slot| {
                                            slot.child(
                                                div()
                                                    .size(px(7.0))
                                                    .rounded_full()
                                                    .bg(theme_rgb(ACCENT)),
                                            )
                                        }),
                                )
                                .child(if renaming {
                                    accessible_input(
                                        ("rename-session-input", session.position),
                                        &self.rename_editor,
                                        Role::TextInput,
                                        self.strings.native.rename_session,
                                        self.strings.native.rename_session,
                                        Input::new(&self.rename_editor),
                                        cx,
                                    )
                                    .h(px(26.0))
                                    .min_w_0()
                                    .flex_1()
                                    .into_any_element()
                                } else {
                                    div()
                                        .min_w_0()
                                        .flex_1()
                                        .text_size(ui_font_px())
                                        .font_weight(if selected {
                                            gpui::FontWeight::MEDIUM
                                        } else {
                                            gpui::FontWeight::NORMAL
                                        })
                                        .text_color(theme_rgb(TEXT))
                                        .line_clamp(1)
                                        .child(title.clone())
                                        .into_any_element()
                                }),
                        )
                        .when(!renaming, |row| {
                            row.child(
                                // Shared trailing slot: relative time by default, row actions
                                // on hover / when selected. Keeps both on the far right.
                                div()
                                    .relative()
                                    .flex_none()
                                    .h(px(20.0))
                                    .min_w(px(28.0))
                                    .child(
                                        div()
                                            .absolute()
                                            .inset_0()
                                            .flex()
                                            .items_center()
                                            .justify_end()
                                            .text_size(font_px(10.0))
                                            .text_color(theme_rgb(TEXT_MUTED))
                                            .when(selected, |time| time.invisible())
                                            .when(!selected, |time| {
                                                time.group_hover(actions_group.clone(), |style| {
                                                    style.invisible()
                                                })
                                            })
                                            .child(time_label),
                                    )
                                    .child(
                                        div()
                                            .absolute()
                                            .inset_0()
                                            .flex()
                                            .items_center()
                                            .justify_end()
                                            .when(!selected, |actions| {
                                                actions.invisible().group_hover(
                                                    actions_group,
                                                    |style| style.visible(),
                                                )
                                            })
                                            .child(
                                                sidebar_icon_button(
                                                    ("session-actions", session.position),
                                                    IconName::Ellipsis,
                                                    self.strings.native.session_actions,
                                                )
                                                .dropdown_menu(move |menu, window, _| {
                                                    let copy_id = menu_session_id.clone();
                                                    let rename_id = menu_session_id.clone();
                                                    let rename_title = menu_title.clone();
                                                    let fork_id = menu_session_id.clone();
                                                    let export_id = menu_session_id.clone();
                                                    let archive_id = menu_session_id.clone();
                                                    let rename = window.listener_for(
                                                        &shell,
                                                        move |this, _, window, cx| {
                                                            this.begin_session_rename_for(
                                                                rename_id.clone(),
                                                                rename_title.clone(),
                                                                window,
                                                                cx,
                                                            )
                                                        },
                                                    );
                                                    let fork = window.listener_for(
                                                        &shell,
                                                        move |this, _, _, cx| {
                                                            this.fork_session(fork_id.clone(), cx)
                                                        },
                                                    );
                                                    let export = window.listener_for(
                                                        &shell,
                                                        move |this, _, _, cx| {
                                                            this.export_session(
                                                                export_id.clone(),
                                                                cx,
                                                            )
                                                        },
                                                    );
                                                    let archive = window.listener_for(
                                                        &shell,
                                                        move |this, _, window, cx| {
                                                            this.confirm_archive_session(
                                                                archive_id.clone(),
                                                                window,
                                                                cx,
                                                            )
                                                        },
                                                    );
                                                    menu.min_w(px(220.0))
                                                        .item(
                                                            PopupMenuItem::new(
                                                                strings.copy_session_id,
                                                            )
                                                            .icon(IconName::Copy)
                                                            .on_click(move |_, _, cx| {
                                                                cx.write_to_clipboard(
                                                                    ClipboardItem::new_string(
                                                                        copy_id.clone(),
                                                                    ),
                                                                )
                                                            }),
                                                        )
                                                        .separator()
                                                        .item(
                                                            PopupMenuItem::new(
                                                                strings.rename_session,
                                                            )
                                                            .icon(
                                                                Icon::default()
                                                                    .path(SESSION_RENAME_ICON_PATH),
                                                            )
                                                            .on_click(rename),
                                                        )
                                                        .item(
                                                            PopupMenuItem::new(strings.fork).icon(
                                                                Icon::default()
                                                                    .path(SESSION_FORK_ICON_PATH),
                                                            )
                                                            .on_click(fork),
                                                        )
                                                        .item(
                                                            PopupMenuItem::new(
                                                                strings.export_session,
                                                            )
                                                            .icon(
                                                                Icon::default()
                                                                    .path(SESSION_EXPORT_ICON_PATH),
                                                            )
                                                            .on_click(export),
                                                        )
                                                        .item(
                                                            PopupMenuItem::element(move |_, _| {
                                                                div()
                                                                    .text_color(theme_rgb(ERROR))
                                                                    .child(strings.archive)
                                                            })
                                                            .icon(
                                                                Icon::default()
                                                                    .path(
                                                                        SESSION_ARCHIVE_ICON_PATH,
                                                                    )
                                                                    .text_color(theme_rgb(ERROR)),
                                                            )
                                                            .on_click(archive),
                                                        )
                                                        .separator()
                                                        .label(created_at.clone())
                                                }),
                                            ),
                                    ),
                            )
                        }),
                )
                .when(session.busy, |item| {
                    item.aria_description(self.strings.native.working)
                })
                .into_any_element()
        }
    }

    fn workspace_row(
        &self,
        index: usize,
        workspace_key: String,
        label: String,
        root: String,
        collapsed: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.renaming_workspace_id.as_deref() == Some(workspace_key.as_str()) {
            return self.workspace_rename_row(index, cx);
        }
        let shell = cx.entity();
        let toggle_key = workspace_key.clone();
        let new_session_root = root.clone();
        let menu_root = root.clone();
        let menu_workspace_id = workspace_key.clone();
        let menu_label = label.clone();
        let strings = self.strings.native;
        let row_group = format!("workspace-row-{index}");
        let actions_group = row_group.clone();
        div()
            .id(("workspace-heading", index))
            .role(Role::ListItem)
            .group(row_group)
            .relative()
            .w_full()
            .h(px(36.0))
            .px_2()
            .flex()
            .items_center()
            .gap_1()
            .rounded_md()
            .text_size(caption_font_px())
            .font_semibold()
            .text_color(theme_rgb(TEXT_SECONDARY))
            .hover(|row| row.bg(theme_rgb(SURFACE_ACTIVE)))
            .child(
                div()
                    .id(("toggle-workspace", index))
                    .w_full()
                    .min_w_0()
                    .focusable()
                    .tab_stop(true)
                    .role(Role::Button)
                    .aria_label(label.clone())
                    .aria_expanded(!collapsed)
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .gap_1()
                    .pr(px(48.0))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.session_list.toggle_workspace(&toggle_key);
                        cx.notify();
                    }))
                    .child(
                        div()
                            .w(px(16.0))
                            .flex_none()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::new(if collapsed {
                                    IconName::FolderClosed
                                } else {
                                    IconName::FolderOpen
                                })
                                .xsmall()
                                .text_color(theme_rgb(TEXT_MUTED)),
                            ),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .line_clamp(1)
                            .child(label.clone()),
                    ),
            )
            .child(
                // Far-right action cluster: same placement as session rows.
                div()
                    .absolute()
                    .right(px(4.0))
                    .top_0()
                    .bottom_0()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap_0()
                    .invisible()
                    .group_hover(actions_group, |style| style.visible())
                    .child(
                        sidebar_icon_button(
                            ("workspace-actions", index),
                            IconName::Ellipsis,
                            self.strings.native.workspace_actions,
                        )
                        .dropdown_menu(move |menu, window, _| {
                            let copy_root = menu_root.clone();
                            let rename_id = menu_workspace_id.clone();
                            let rename_label = menu_label.clone();
                            let remove_id = menu_workspace_id.clone();
                            let remove_label = menu_label.clone();
                            let rename = window.listener_for(&shell, move |this, _, window, cx| {
                                this.begin_workspace_rename(
                                    rename_id.clone(),
                                    rename_label.clone(),
                                    window,
                                    cx,
                                )
                            });
                            let remove = window.listener_for(&shell, move |this, _, window, cx| {
                                this.confirm_remove_workspace(
                                    remove_id.clone(),
                                    remove_label.clone(),
                                    window,
                                    cx,
                                )
                            });
                            menu.min_w(px(210.0))
                                .item(PopupMenuItem::new(strings.copy_path).on_click(
                                    move |_, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            copy_root.clone(),
                                        ))
                                    },
                                ))
                                .separator()
                                .item(
                                    PopupMenuItem::new(strings.rename_workspace).on_click(rename),
                                )
                                .separator()
                                .item(
                                    PopupMenuItem::element(move |_, _| {
                                        div()
                                            .text_color(theme_rgb(ERROR))
                                            .child(strings.remove_workspace)
                                    })
                                    .on_click(remove),
                                )
                        }),
                    )
                    .child(
                        sidebar_icon_button(
                            ("new-session-in-workspace", index),
                            IconName::Plus,
                            self.strings.native.new_session_in_workspace,
                        )
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.begin_new_session_in_workspace(
                                new_session_root.clone(),
                                window,
                                cx,
                            )
                        })),
                    ),
            )
            .into_any_element()
    }

    fn workspace_rename_row(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        div()
            .id(("workspace-rename", index))
            .h(px(36.0))
            .px_1()
            .flex()
            .items_center()
            .gap_1()
            .child(
                accessible_input(
                    ("workspace-rename-input", index),
                    &self.workspace_rename_editor,
                    Role::TextInput,
                    self.strings.native.rename_workspace,
                    self.strings.native.rename_workspace,
                    Input::new(&self.workspace_rename_editor),
                    cx,
                )
                .h(px(28.0))
                .flex_1(),
            )
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
            .text_size(font_px(10.0))
            .text_color(theme_rgb(TEXT_MUTED))
            .hover(|row| {
                row.bg(theme_rgb(SURFACE_ACTIVE))
                    .text_color(theme_rgb(TEXT_SECONDARY))
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.session_list.toggle_expanded(&workspace_key);
                cx.notify();
            }))
            .child(label)
            .into_any_element()
    }
}

fn sidebar_icon_button(
    id: impl Into<gpui::ElementId>,
    icon: IconName,
    tooltip: impl Into<gpui::SharedString>,
) -> Button {
    Button::new(id)
        .xsmall()
        .ghost()
        .icon(icon)
        .text_color(theme_rgb(TEXT_MUTED))
        .tooltip(tooltip)
}
