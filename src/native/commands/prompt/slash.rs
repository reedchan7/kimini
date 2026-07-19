use gpui::{Context, Window};

use crate::native::slash::SlashCommand;

use super::super::super::app::{SettingsTab, Shell};

impl Shell {
    pub(super) fn run_slash_command(
        &mut self,
        command: SlashCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match command {
            SlashCommand::New => self.begin_new_session(window, cx),
            SlashCommand::Fork => self.fork_active_session(cx),
            SlashCommand::Export => self.export_active_session(cx),
            SlashCommand::Undo => self.confirm_undo(window, cx),
            SlashCommand::Plan => self.toggle_plan_mode(cx),
            SlashCommand::Permission(mode) => self.set_permission(mode.into(), cx),
            SlashCommand::Thinking => self.cycle_thinking(cx),
            SlashCommand::Login => self.open_auth_panel(SettingsTab::Account, cx),
            SlashCommand::Compact(None) => self.confirm_compact(window, cx),
            SlashCommand::Compact(Some(instruction)) => {
                self.compact_active_session(Some(instruction), cx)
            }
            SlashCommand::Swarm(None) => self.toggle_swarm_mode(cx),
            SlashCommand::Swarm(Some(enabled)) => self.set_swarm_mode(enabled, cx),
            SlashCommand::Goal(command) => self.run_goal_slash(command, window, cx),
            SlashCommand::Btw(initial) => self.open_side_chat(initial, window, cx),
        }
    }
}
