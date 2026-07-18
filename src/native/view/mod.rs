mod accessible_input;
mod auth;
mod browser;
mod composer;
mod composer_attachment;
mod composer_controls;
mod composer_slash;
mod conversation;
mod files;
mod goal;
mod interaction;
mod message;
mod panel;
mod prompt_queue;
mod recovery;
mod side_chat;
mod sidebar;
mod sidebar_row;
mod skills;
mod tasks;
mod terminal;
mod thinking;
mod tool_card;
mod toolbar;

use gpui::{Context, IntoElement, Render, Role, Window, div, prelude::*, rgb};

use super::app::{Shell, UtilityPanel};
use super::theme::*;
use super::{
    ArchiveSession, CompactSession, ExportSession, FocusNext, FocusPrevious, ForkSession,
    RenameSession, SetModel, SetPermission, SetThinking, SteerPrompt, ToggleFiles, ToggleSkills,
    ToggleTasks, ToggleTerminal, UndoSession,
};

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_composer_draft(window, cx);
        self.sync_question_inputs(window, cx);
        div()
            .id("kimini-root")
            .role(Role::Application)
            .aria_label("Kimini")
            .on_action(cx.listener(|_, _: &FocusNext, window, cx| window.focus_next(cx)))
            .on_action(cx.listener(|_, _: &FocusPrevious, window, cx| window.focus_prev(cx)))
            .on_action(
                cx.listener(|this, _: &SteerPrompt, window, cx| this.steer_prompt(window, cx)),
            )
            .on_action(cx.listener(|this, _: &ToggleTasks, _, cx| this.toggle_task_panel(cx)))
            .on_action(cx.listener(|this, _: &ToggleFiles, _, cx| this.toggle_file_panel(cx)))
            .on_action(cx.listener(|this, _: &ToggleSkills, _, cx| this.toggle_skill_panel(cx)))
            .on_action(
                cx.listener(|this, _: &ToggleTerminal, window, cx| {
                    this.toggle_terminal(window, cx)
                }),
            )
            .on_action(cx.listener(|this, action: &SetModel, _, cx| {
                this.set_model(action.model.clone(), cx)
            }))
            .on_action(cx.listener(|this, action: &SetThinking, _, cx| {
                this.set_thinking(action.effort.clone(), cx)
            }))
            .on_action(cx.listener(|this, action: &SetPermission, _, cx| {
                this.set_permission(action.mode.clone(), cx)
            }))
            .on_action(cx.listener(|this, _: &RenameSession, window, cx| {
                this.begin_session_rename(window, cx)
            }))
            .on_action(cx.listener(|this, _: &ForkSession, _, cx| this.fork_active_session(cx)))
            .on_action(
                cx.listener(|this, _: &CompactSession, window, cx| {
                    this.confirm_compact(window, cx)
                }),
            )
            .on_action(
                cx.listener(|this, _: &UndoSession, window, cx| this.confirm_undo(window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &ArchiveSession, window, cx| {
                    this.confirm_archive(window, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &ExportSession, _, cx| this.export_active_session(cx)))
            .size_full()
            .flex()
            .bg(rgb(CANVAS))
            .text_color(rgb(TEXT))
            .child(self.sidebar(cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(self.toolbar(cx))
                    .children(self.recovery_banner(cx))
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .flex()
                            .child(if self.browser.is_some() {
                                self.browser_surface(cx).into_any_element()
                            } else {
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .h_full()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .child(self.conversation(cx))
                                    .children(self.goal_strip(cx))
                                    .child(self.composer(window, cx))
                                    .into_any_element()
                            })
                            .when(
                                self.utility_panel == Some(UtilityPanel::Thinking)
                                    && self.browser.is_none(),
                                |layout| layout.child(self.thinking_panel(cx)),
                            )
                            .when(
                                self.utility_panel == Some(UtilityPanel::Tasks)
                                    && self.browser.is_none(),
                                |layout| layout.child(self.task_panel(cx)),
                            )
                            .when(
                                self.utility_panel == Some(UtilityPanel::Auth)
                                    && self.browser.is_none(),
                                |layout| layout.child(self.auth_panel(cx)),
                            )
                            .when(
                                self.utility_panel == Some(UtilityPanel::Files)
                                    && self.browser.is_none(),
                                |layout| layout.child(self.file_panel(cx)),
                            )
                            .when(
                                self.utility_panel == Some(UtilityPanel::Skills)
                                    && self.browser.is_none(),
                                |layout| layout.child(self.skill_panel(cx)),
                            )
                            .when(
                                self.utility_panel == Some(UtilityPanel::SideChat)
                                    && self.browser.is_none(),
                                |layout| layout.child(self.side_chat_panel(cx)),
                            )
                            .when(
                                self.utility_panel == Some(UtilityPanel::Terminal)
                                    && self.browser.is_none(),
                                |layout| layout.child(self.terminal_panel(cx)),
                            ),
                    ),
            )
    }
}
