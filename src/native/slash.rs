use crate::protocol::GoalControl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SlashCommand {
    New,
    Fork,
    Export,
    Undo,
    Plan,
    Permission(&'static str),
    Thinking,
    Login,
    Compact(Option<String>),
    Swarm(Option<bool>),
    Goal(GoalSlashCommand),
    Btw(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GoalSlashCommand {
    Toggle,
    Create(String),
    Control(GoalControl),
}

impl SlashCommand {
    pub const fn available_in_new_session(&self) -> bool {
        matches!(
            self,
            Self::New
                | Self::Plan
                | Self::Permission(_)
                | Self::Thinking
                | Self::Login
                | Self::Swarm(_)
        )
    }

    pub fn suggestions(input: &str, limit: usize) -> Vec<&'static str> {
        const COMMANDS: [&str; 14] = [
            "/new",
            "/clear",
            "/btw",
            "/login",
            "/plan",
            "/swarm",
            "/goal",
            "/auto",
            "/yolo",
            "/thinking",
            "/compact",
            "/undo",
            "/fork",
            "/export",
        ];
        if !input.starts_with('/') || input.chars().any(char::is_whitespace) {
            return Vec::new();
        }
        let query = input.to_ascii_lowercase();
        let mut matches = COMMANDS
            .into_iter()
            .filter_map(|command| {
                let score = if command == query {
                    0
                } else if command.starts_with(&query) {
                    1
                } else if command.contains(&query) {
                    2
                } else {
                    return None;
                };
                Some((score, command))
            })
            .collect::<Vec<_>>();
        matches.sort_by_key(|(score, command)| (*score, *command));
        matches
            .into_iter()
            .take(limit)
            .map(|(_, command)| command)
            .collect()
    }

    pub fn parse(input: &str) -> Option<Self> {
        let command = input.strip_prefix('/')?;
        let (name, argument) = command
            .split_once(char::is_whitespace)
            .unwrap_or((command, ""));
        let argument = argument.trim();
        match (name, argument) {
            ("new" | "clear", "") => Some(Self::New),
            ("fork", "") => Some(Self::Fork),
            ("export", "") => Some(Self::Export),
            ("undo", "") => Some(Self::Undo),
            ("plan", "") => Some(Self::Plan),
            ("auto", "") => Some(Self::Permission("auto")),
            ("yolo", "") => Some(Self::Permission("yolo")),
            ("thinking", "") => Some(Self::Thinking),
            ("login", "") => Some(Self::Login),
            ("compact", "") => Some(Self::Compact(None)),
            ("compact", instruction) => Some(Self::Compact(Some(instruction.to_owned()))),
            ("swarm", "") => Some(Self::Swarm(None)),
            ("swarm", "on") => Some(Self::Swarm(Some(true))),
            ("swarm", "off") => Some(Self::Swarm(Some(false))),
            ("goal", "") => Some(Self::Goal(GoalSlashCommand::Toggle)),
            ("goal", "pause") => Some(Self::Goal(GoalSlashCommand::Control(GoalControl::Pause))),
            ("goal", "resume") => Some(Self::Goal(GoalSlashCommand::Control(GoalControl::Resume))),
            ("goal", "cancel") => Some(Self::Goal(GoalSlashCommand::Control(GoalControl::Cancel))),
            ("goal", objective) => Some(Self::Goal(GoalSlashCommand::Create(objective.to_owned()))),
            ("btw", "") => Some(Self::Btw(None)),
            ("btw", question) => Some(Self::Btw(Some(question.to_owned()))),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_requires_exact_known_commands_and_preserves_compact_instruction() {
        assert_eq!(SlashCommand::parse("/clear"), Some(SlashCommand::New));
        assert_eq!(
            SlashCommand::parse("/swarm off"),
            Some(SlashCommand::Swarm(Some(false)))
        );
        assert_eq!(
            SlashCommand::parse("/compact focus on APIs"),
            Some(SlashCommand::Compact(Some("focus on APIs".into())))
        );
        assert_eq!(SlashCommand::parse(" /undo"), None);
        assert_eq!(SlashCommand::parse("/undo twice"), None);
        assert_eq!(SlashCommand::parse("/unknown"), None);
        assert_eq!(SlashCommand::suggestions("/co", 5), vec!["/compact"]);
        assert_eq!(SlashCommand::parse("/btw"), Some(SlashCommand::Btw(None)));
        assert_eq!(
            SlashCommand::parse("/btw what changed?"),
            Some(SlashCommand::Btw(Some("what changed?".into())))
        );
        assert_eq!(
            SlashCommand::parse("/goal pause"),
            Some(SlashCommand::Goal(GoalSlashCommand::Control(
                GoalControl::Pause
            )))
        );
        assert_eq!(
            SlashCommand::parse("/goal ship the GUI"),
            Some(SlashCommand::Goal(GoalSlashCommand::Create(
                "ship the GUI".into()
            )))
        );
        assert!(SlashCommand::suggestions("/compact ", 5).is_empty());
    }

    #[test]
    fn new_drafts_expose_only_commands_that_do_not_need_a_session() {
        assert!(!SlashCommand::Fork.available_in_new_session());
        assert!(!SlashCommand::Export.available_in_new_session());
        assert!(!SlashCommand::Undo.available_in_new_session());
        assert!(!SlashCommand::Compact(None).available_in_new_session());
        assert!(!SlashCommand::Goal(GoalSlashCommand::Toggle).available_in_new_session());
        assert!(!SlashCommand::Btw(None).available_in_new_session());
        assert!(SlashCommand::New.available_in_new_session());
        assert!(SlashCommand::Plan.available_in_new_session());
        assert!(SlashCommand::Permission("auto").available_in_new_session());
        assert!(SlashCommand::Thinking.available_in_new_session());
        assert!(SlashCommand::Login.available_in_new_session());
        assert!(SlashCommand::Swarm(None).available_in_new_session());
    }
}
