use gpui::{Context, IntoElement, Role, div, img, list, prelude::*, px};
use gpui_component::{Icon, IconName, Sizable as _, StyledExt, scroll::ScrollableElement};

use super::super::app::Shell;
use super::super::shell::APP_ICON_PATH;
use super::super::theme::*;

impl Shell {
    pub(super) fn sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let archived = self.show_archived;
        let sessions = if archived {
            self.model.archived_sessions()
        } else {
            self.model.sessions()
        };
        let active = (!archived && self.new_session_draft.is_none())
            .then(|| {
                self.model
                    .active_session()
                    .map(|session| session.id.clone())
            })
            .flatten();
        self.session_list.sync(sessions, "", active.as_deref());
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
            .border_color(theme_rgb(BORDER))
            .bg(theme_rgb(SIDEBAR))
            .child(self.sidebar_brand(cx))
            .child(self.sidebar_primary_actions(cx))
            .child(self.sidebar_section_header(archived, cx))
            .child(
                div()
                    .id("session-list")
                    .role(Role::List)
                    .aria_label(self.strings.native.sessions_list)
                    .flex_1()
                    .min_h_0()
                    .px_3()
                    .when(list_empty, |container| {
                        container.child(
                            div()
                                .px_2()
                                .py_3()
                                .text_size(font_px(12.0))
                                .text_color(theme_rgb(TEXT_MUTED))
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

    fn sidebar_brand(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(HEADER_HEIGHT))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px(px(20.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(img(APP_ICON_PATH).size(px(24.0)).rounded_md())
                    .child(
                        div()
                            .text_size(font_px(13.0))
                            .font_semibold()
                            .child("Kimini"),
                    ),
            )
            .child(
                div()
                    .id("collapse-sidebar")
                    .focusable()
                    .tab_stop(true)
                    .role(Role::Button)
                    .aria_label(self.strings.native.collapse_sidebar)
                    .cursor_pointer()
                    .size(px(28.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .text_color(theme_rgb(TEXT_MUTED))
                    .hover(|item| {
                        item.bg(theme_rgb(SURFACE_ACTIVE))
                            .text_color(theme_rgb(TEXT))
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sidebar_collapsed = true;
                        cx.notify();
                    }))
                    .child(Icon::new(IconName::PanelLeft).xsmall()),
            )
    }

    fn sidebar_primary_actions(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_3()
            .pb_2()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                sidebar_action("new-session", self.strings.native.new_session)
                    .child(
                        Icon::new(IconName::Plus)
                            .xsmall()
                            .text_color(theme_rgb(TEXT_SECONDARY)),
                    )
                    .child(div().child(self.strings.native.new_session))
                    .on_click(
                        cx.listener(|this, _, window, cx| this.begin_new_session(window, cx)),
                    ),
            )
            .child(
                sidebar_action("session-search", self.strings.native.search_sessions)
                    .child(
                        Icon::new(IconName::Search)
                            .xsmall()
                            .text_color(theme_rgb(TEXT_SECONDARY)),
                    )
                    .child(div().flex_1().child(self.strings.native.search_sessions))
                    .child(search_shortcut_hint())
                    .on_click(
                        cx.listener(|this, _, window, cx| this.open_session_search(window, cx)),
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
            .h(px(32.0))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px(px(20.0))
            .text_color(theme_rgb(TEXT_MUTED))
            .child(
                div()
                    .id("sessions-heading")
                    .role(Role::Heading)
                    .aria_level(2)
                    .aria_label(heading)
                    .text_size(font_px(10.0))
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
            .border_color(theme_rgb(BORDER))
            .px_3()
            .py_2()
            .flex()
            .items_center()
            .gap_1()
            .child(
                sidebar_action("settings", self.strings.settings)
                    .flex_1()
                    .child(
                        Icon::new(IconName::Settings)
                            .xsmall()
                            .text_color(theme_rgb(TEXT_SECONDARY)),
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
        .text_size(font_px(13.0))
        .text_color(theme_rgb(TEXT_SECONDARY))
        .hover(|item| {
            item.bg(theme_rgb(SURFACE_ACTIVE))
                .text_color(theme_rgb(TEXT))
        })
}

fn search_shortcut_hint() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_1()
        .children(["⌘", "K"].map(|key| {
            div()
                .min_w(px(16.0))
                .h(px(18.0))
                .px_1()
                .flex()
                .items_center()
                .justify_center()
                .rounded_sm()
                .border_1()
                .border_color(theme_rgb(BORDER))
                .bg(theme_rgb(SURFACE_SUBTLE))
                .text_size(font_px(10.0))
                .text_color(theme_rgb(TEXT_MUTED))
                .child(key)
        }))
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
        .text_size(font_px(10.0))
        .text_color(theme_rgb(TEXT_MUTED))
        .hover(|item| {
            item.bg(theme_rgb(SURFACE_ACTIVE))
                .text_color(theme_rgb(TEXT_SECONDARY))
        })
}
