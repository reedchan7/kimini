mod controls;
mod title;

use gpui::{Context, IntoElement, Role, div, prelude::*, px};
use gpui_component::{
    IconName, Sizable as _,
    button::{Button, ButtonVariants},
    menu::DropdownMenu,
};

use crate::native::{
    ArchiveSession, CompactSession, ExportSession, ForkSession, RenameSession, ToggleBrowser,
    ToggleFiles, ToggleSideChat, ToggleSkills, ToggleTasks, ToggleTerminal, UndoSession,
    app::{LoadState, Shell},
    theme::*,
};

impl Shell {
    pub(super) fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let draft_open = self.new_session_draft.is_some();
        let session = (!draft_open).then(|| self.model.active_session()).flatten();
        let show_connection = !matches!(&self.state, LoadState::Ready);
        div()
            .h(px(HEADER_HEIGHT))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .border_b_1()
            .border_color(theme_rgb(BORDER))
            .bg(theme_rgb(SURFACE))
            .child(
                div()
                    .id("app-title")
                    .when(!draft_open, |title| {
                        title.role(Role::Heading).aria_level(1).aria_label(
                            session
                                .map(|item| item.title.as_str())
                                .unwrap_or(self.strings.native.start_session),
                        )
                    })
                    .flex()
                    .items_center()
                    .gap_1()
                    .when(self.sidebar_collapsed, |title| {
                        title.child(self.sidebar_restore_button(cx))
                    })
                    .children(session.map(|item| self.session_title(item)))
                    .when(session.is_some(), |title| {
                        title.child(self.session_actions())
                    })
                    .when(session.is_none() && !draft_open, |title| {
                        title
                            .text_size(font_px(13.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(self.strings.native.start_session)
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .when(show_connection, |toolbar| {
                        toolbar.child(
                            div()
                                .id("connection-status")
                                .role(Role::Status)
                                .aria_label(self.status_text())
                                .flex()
                                .items_center()
                                .gap_2()
                                .text_size(font_px(11.0))
                                .text_color(theme_rgb(TEXT_MUTED))
                                .child(
                                    div()
                                        .size(px(6.0))
                                        .rounded_full()
                                        .bg(theme_rgb(self.connection_status_color())),
                                )
                                .child(self.status_text()),
                        )
                    })
                    .when_some(
                        (!draft_open)
                            .then(|| {
                                self.files
                                    .git
                                    .as_ref()
                                    .map(|git| git.branch.clone())
                                    .filter(|branch| !branch.is_empty())
                            })
                            .flatten(),
                        |toolbar, branch| toolbar.child(self.git_branch(branch, cx)),
                    ),
            )
    }

    fn session_actions(&self) -> impl IntoElement {
        let strings = self.strings.native;
        Button::new("session-actions-button")
            .xsmall()
            .ghost()
            .icon(IconName::Ellipsis)
            .tooltip(strings.session_actions)
            .dropdown_menu(move |menu, _, _| {
                menu.menu(strings.rename_session, Box::new(RenameSession))
                    .menu(strings.fork, Box::new(ForkSession))
                    .menu(strings.compact, Box::new(CompactSession))
                    .menu(strings.undo, Box::new(UndoSession))
                    .menu(strings.export_session, Box::new(ExportSession))
                    .menu(strings.archive, Box::new(ArchiveSession))
                    .separator()
                    .menu(strings.files, Box::new(ToggleFiles))
                    .menu(strings.skills, Box::new(ToggleSkills))
                    .menu(strings.terminal, Box::new(ToggleTerminal))
                    .menu(strings.tasks, Box::new(ToggleTasks))
                    .menu(strings.side_chat, Box::new(ToggleSideChat))
                    .menu(strings.browser, Box::new(ToggleBrowser))
            })
    }

    fn sidebar_restore_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("expand-sidebar")
            .role(Role::Button)
            .aria_label(self.strings.native.expand_sidebar)
            .cursor_pointer()
            .size(px(28.0))
            .flex_none()
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
                this.sidebar_collapsed = false;
                cx.notify();
            }))
            .child(gpui_component::Icon::new(IconName::PanelLeftOpen).xsmall())
    }

    fn git_branch(&self, branch: String, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("git-branch")
            .role(Role::Button)
            .aria_label(format!("{}: {branch}", self.strings.native.files))
            .cursor_pointer()
            .rounded_md()
            .px_2()
            .py_1()
            .text_size(font_px(11.0))
            .font_family("SFMono-Regular")
            .text_color(theme_rgb(TEXT_MUTED))
            .hover(|item| {
                item.bg(theme_rgb(SURFACE_ACTIVE))
                    .text_color(theme_rgb(TEXT))
            })
            .on_click(cx.listener(|this, _, _, cx| this.toggle_file_panel(cx)))
            .child(branch)
    }
}
