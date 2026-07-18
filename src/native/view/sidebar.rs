use gpui::{Context, IntoElement, Role, div, list, prelude::*, px, rgb};
use gpui_component::{
    Icon, IconName, Sizable as _, StyledExt, input::Input, scroll::ScrollableElement,
};

use super::super::app::Shell;
use super::super::theme::*;
use super::accessible_input::accessible_input;

impl Shell {
    pub(super) fn sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let archived = self.show_archived;
        let sessions = if archived {
            self.model.archived_sessions()
        } else {
            self.model.sessions()
        };
        let active = (!archived)
            .then(|| {
                self.model
                    .active_session()
                    .map(|session| session.id.clone())
            })
            .flatten();
        let query = self.session_search.read(cx).value().trim().to_lowercase();
        self.session_list.sync(sessions, &query, active.as_deref());
        let visible_count = self.session_list.session_count();
        let list_empty = self.session_list.is_empty();
        let list_state = self.session_list.list.clone();
        let has_more = if archived {
            self.model.has_more_archived_sessions()
        } else {
            self.model.has_more_sessions()
        };
        let loading = if archived {
            self.archives_loading
        } else {
            self.sessions_loading
        };
        let shell = cx.entity();

        div()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SIDEBAR))
            .child(self.sidebar_brand())
            .child(self.sidebar_primary_actions(cx))
            .child(self.sidebar_section_header(archived, cx))
            .child(
                div()
                    .id("session-list")
                    .role(Role::List)
                    .aria_label(self.strings.native.sessions_list)
                    .flex_1()
                    .min_h_0()
                    .px_2()
                    .when(list_empty, |container| {
                        container.child(
                            div()
                                .px_2()
                                .py_3()
                                .text_xs()
                                .text_color(rgb(TEXT_MUTED))
                                .child(if archived && self.model.archived_sessions_loaded() {
                                    self.strings.native.no_archived_sessions
                                } else if loading {
                                    self.strings.native.loading_sessions
                                } else {
                                    self.strings.native.search_sessions
                                }),
                        )
                    })
                    .when(!list_empty, |container| {
                        container.child(
                            div()
                                .size_full()
                                .relative()
                                .child(
                                    list(list_state.clone(), move |index, _, cx| {
                                        shell.update(cx, |this, cx| {
                                            this.sidebar_list_row(
                                                index,
                                                visible_count,
                                                active.as_deref(),
                                                archived,
                                                cx,
                                            )
                                        })
                                    })
                                    .size_full(),
                                )
                                .vertical_scrollbar(&list_state),
                        )
                    }),
            )
            .when(has_more, |sidebar| {
                sidebar.child(self.load_more_sessions_button(archived, loading, cx))
            })
            .child(self.sidebar_footer(cx))
    }

    fn sidebar_brand(&self) -> impl IntoElement {
        div()
            .h(px(HEADER_HEIGHT))
            .flex_none()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .child(
                div()
                    .size(px(22.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .bg(rgb(ACCENT))
                    .text_color(rgb(SURFACE))
                    .text_xs()
                    .font_semibold()
                    .child("K"),
            )
            .child(div().text_sm().font_semibold().child("Kimi Code"))
    }

    fn sidebar_primary_actions(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .pt_1()
            .pb_3()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                sidebar_action("new-session", self.strings.native.new_session)
                    .child(
                        Icon::new(IconName::Plus)
                            .xsmall()
                            .text_color(rgb(TEXT_SECONDARY)),
                    )
                    .child(div().child(self.strings.native.new_session.trim_start_matches("+ ")))
                    .on_click(cx.listener(|this, _, _, cx| this.choose_session_workspace(cx))),
            )
            .child(
                div().px_1().child(
                    accessible_input(
                        "session-search-input",
                        &self.session_search,
                        Role::SearchInput,
                        self.strings.native.search_sessions,
                        self.strings.native.search_sessions,
                        Input::new(&self.session_search),
                        cx,
                    )
                    .h(px(32.0)),
                ),
            )
    }

    fn sidebar_section_header(&self, archived: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let heading = if archived {
            self.strings.native.archived_sessions
        } else {
            self.strings.native.workspace_tools
        };
        let toggle = if archived {
            self.strings.native.active_sessions
        } else {
            self.strings.native.archived_sessions
        };
        div()
            .h(px(28.0))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .text_color(rgb(TEXT_MUTED))
            .child(
                div()
                    .id("sessions-heading")
                    .role(Role::Heading)
                    .aria_level(2)
                    .aria_label(heading)
                    .text_size(px(10.0))
                    .font_semibold()
                    .child(heading.to_uppercase()),
            )
            .child(
                sidebar_text_button(toggle, "toggle-archived-sessions")
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_archived_sessions(cx))),
            )
    }

    fn sidebar_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_none()
            .border_t_1()
            .border_color(rgb(BORDER))
            .p_2()
            .flex()
            .items_center()
            .gap_1()
            .child(
                sidebar_action("settings", self.strings.settings)
                    .flex_1()
                    .child(
                        Icon::new(IconName::Settings)
                            .xsmall()
                            .text_color(rgb(TEXT_SECONDARY)),
                    )
                    .child(self.strings.settings.trim_end_matches('…'))
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_auth_panel(cx))),
            )
            .child(
                sidebar_text_button(self.strings.native.switch_language, "language-toggle")
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_language(window, cx))),
            )
    }

    fn load_more_sessions_button(
        &self,
        archived: bool,
        loading: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        sidebar_text_button(self.strings.native.load_more_sessions, "load-more-sessions")
            .mx_3()
            .mb_2()
            .when(!loading, |item| {
                item.on_click(cx.listener(move |this, _, _, cx| {
                    if archived {
                        this.load_more_archived_sessions(cx);
                    } else {
                        this.load_more_sessions(cx);
                    }
                }))
            })
            .child(if loading {
                self.strings.native.loading_sessions
            } else {
                self.strings.native.load_more_sessions
            })
    }
}

fn sidebar_action(id: &'static str, label: &'static str) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(label)
        .cursor_pointer()
        .h(px(34.0))
        .px_2()
        .flex()
        .items_center()
        .gap_2()
        .rounded_md()
        .text_sm()
        .text_color(rgb(TEXT_SECONDARY))
        .hover(|item| item.bg(rgb(SURFACE_ACTIVE)).text_color(rgb(TEXT)))
}

fn sidebar_text_button(label: &'static str, id: &'static str) -> gpui::Stateful<gpui::Div> {
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
        .text_size(px(10.0))
        .text_color(rgb(TEXT_MUTED))
        .hover(|item| item.bg(rgb(SURFACE_ACTIVE)).text_color(rgb(TEXT_SECONDARY)))
}
