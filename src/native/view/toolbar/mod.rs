mod controls;
mod title;

use gpui::{Context, IntoElement, Role, div, prelude::*, px, rgb};
use gpui_component::{
    IconName, Sizable as _,
    button::{Button, ButtonVariants},
    menu::DropdownMenu,
};

use crate::native::{
    ArchiveSession, CompactSession, ExportSession, ForkSession, RenameSession, ToggleFiles,
    ToggleSkills, ToggleTasks, ToggleTerminal, UndoSession, app::Shell, theme::*,
};

impl Shell {
    pub(super) fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let session = self.model.active_session();
        let runtime = self.model.active_runtime();
        div()
            .h(px(HEADER_HEIGHT))
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .border_b_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SURFACE))
            .child(
                div()
                    .id("app-title")
                    .role(Role::Heading)
                    .aria_level(1)
                    .aria_label(
                        session
                            .map(|item| item.title.as_str())
                            .unwrap_or(self.strings.native.start_session),
                    )
                    .flex()
                    .items_center()
                    .children(session.map(|item| self.session_title(item, cx)))
                    .when(session.is_none(), |title| {
                        title
                            .text_sm()
                            .text_color(rgb(TEXT_MUTED))
                            .child(self.strings.native.start_session)
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .id("connection-status")
                            .role(Role::Status)
                            .aria_label(self.status_text())
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_size(px(11.0))
                            .text_color(rgb(TEXT_MUTED))
                            .child(
                                div()
                                    .size(px(6.0))
                                    .rounded_full()
                                    .bg(rgb(self.connection_status_color())),
                            )
                            .child(self.status_text()),
                    )
                    .child(
                        div()
                            .id("runtime-status")
                            .role(Role::Status)
                            .aria_label(self.strings.native.session_runtime)
                            .text_size(px(11.0))
                            .text_color(rgb(TEXT_MUTED))
                            .children(runtime.map(|item| {
                                div().child(format!(
                                    "{}% {}",
                                    item.context_percent(),
                                    self.strings.native.context_label
                                ))
                            })),
                    )
                    .children(self.thinking_button(cx))
                    .when(session.is_some(), |toolbar| {
                        toolbar
                            .child(self.session_actions())
                            .child(self.workspace_actions())
                    })
                    .child(self.browser_button(cx)),
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
            })
    }

    fn workspace_actions(&self) -> impl IntoElement {
        let strings = self.strings.native;
        Button::new("workspace-actions-button")
            .xsmall()
            .ghost()
            .icon(IconName::Frame)
            .tooltip(strings.workspace_tools)
            .dropdown_menu(move |menu, _, _| {
                menu.menu(strings.files, Box::new(ToggleFiles))
                    .menu(strings.skills, Box::new(ToggleSkills))
                    .menu(strings.terminal, Box::new(ToggleTerminal))
                    .menu(strings.tasks, Box::new(ToggleTasks))
            })
    }
}
