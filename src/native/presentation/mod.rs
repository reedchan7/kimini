mod content;
mod tool;

use gpui::{FollowMode, ListAlignment, ListState, px};

use crate::model::AppModel;

pub(super) use content::{AttachmentKind, TranscriptBlock, TranscriptRow};
pub(super) use tool::ToolCard;

#[derive(Debug)]
pub(super) struct Transcript {
    pub rows: Vec<TranscriptRow>,
    pub list: ListState,
    session_id: Option<String>,
}

impl Default for Transcript {
    fn default() -> Self {
        let list = ListState::new(0, ListAlignment::Top, px(900.));
        list.set_follow_mode(FollowMode::Tail);
        Self {
            rows: Vec::new(),
            list,
            session_id: None,
        }
    }
}

impl Transcript {
    pub fn latest_thinking(&self) -> Option<&str> {
        self.rows.iter().rev().find_map(|row| {
            row.blocks.iter().rev().find_map(|block| match block {
                TranscriptBlock::Thinking(text) if !text.trim().is_empty() => Some(text.as_str()),
                _ => None,
            })
        })
    }

    pub fn rebuild(&mut self, model: &AppModel) {
        let Some(conversation) = model.active_conversation() else {
            self.replace(None, Vec::new());
            return;
        };
        let session_id = model.active_session().map(|session| session.id.clone());
        let tool_index = tool::ToolIndex::from_messages(&conversation.messages);
        let mut rows = conversation
            .messages
            .iter()
            .filter_map(|message| TranscriptRow::from_message(message, &tool_index))
            .collect::<Vec<_>>();
        if let Some(stream) = TranscriptRow::from_stream(
            session_id.as_deref().unwrap_or_default(),
            conversation.thinking_stream.as_deref(),
            conversation.assistant_stream.as_deref(),
        ) {
            rows.push(stream);
        }
        self.replace(session_id, rows);
    }

    pub fn sync_stream(&mut self, model: &AppModel) {
        self.rebuild(model);
    }

    fn replace(&mut self, session_id: Option<String>, rows: Vec<TranscriptRow>) {
        if self.session_id != session_id {
            self.session_id = session_id;
            self.rows = rows;
            self.list.reset(self.rows.len());
            self.list.set_follow_mode(FollowMode::Tail);
            return;
        }

        let prepended = rows.len().saturating_sub(self.rows.len());
        if prepended > 0
            && !self.rows.is_empty()
            && self
                .rows
                .iter()
                .zip(&rows[prepended..])
                .all(|(old, new)| old.id == new.id)
        {
            let changed = self
                .rows
                .iter()
                .zip(&rows[prepended..])
                .enumerate()
                .filter_map(|(index, (old, new))| (old != new).then_some(index + prepended))
                .collect::<Vec<_>>();
            self.rows = rows;
            self.list.splice(0..0, prepended);
            for index in changed {
                self.list.remeasure_items(index..index + 1);
            }
            return;
        }

        let common = self
            .rows
            .iter()
            .zip(&rows)
            .take_while(|(old, new)| old.id == new.id)
            .count();
        if common < self.rows.len().min(rows.len()) {
            let old_len = self.rows.len();
            self.rows = rows;
            self.list.splice(common..old_len, self.rows.len() - common);
            return;
        }

        let changed = self
            .rows
            .iter()
            .zip(&rows)
            .enumerate()
            .filter_map(|(index, (old, new))| (old != new).then_some(index))
            .collect::<Vec<_>>();
        let old_len = self.rows.len();
        self.rows = rows;
        if old_len != self.rows.len() {
            self.list.splice(common..old_len, self.rows.len() - common);
        }
        for index in changed {
            self.list.remeasure_items(index..index + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::MessageRole;

    fn row(id: &str, text: &str) -> TranscriptRow {
        TranscriptRow {
            id: id.into(),
            role: MessageRole::Assistant,
            blocks: vec![TranscriptBlock::Text(text.into())],
            streaming: false,
        }
    }

    #[test]
    fn replacement_preserves_a_stable_prefix_and_updates_the_list_count() {
        let mut transcript = Transcript::default();
        transcript.replace(Some("session".into()), vec![row("one", "old")]);
        transcript.replace(
            Some("session".into()),
            vec![row("one", "new"), row("two", "next")],
        );

        assert_eq!(transcript.list.item_count(), 2);
        assert_eq!(transcript.rows[0].accessible_text(), "new");
    }

    #[test]
    fn prepending_history_keeps_existing_rows_as_a_stable_suffix() {
        let mut transcript = Transcript::default();
        transcript.replace(
            Some("session".into()),
            vec![row("two", "middle"), row("three", "latest")],
        );
        transcript.replace(
            Some("session".into()),
            vec![
                row("one", "oldest"),
                row("two", "middle"),
                row("three", "latest"),
            ],
        );

        assert_eq!(transcript.list.item_count(), 3);
        assert_eq!(transcript.rows[0].id, "one");
        assert_eq!(transcript.rows[2].id, "three");
    }

    #[test]
    fn latest_thinking_skips_empty_protocol_blocks() {
        let transcript = Transcript {
            rows: vec![
                TranscriptRow {
                    id: "useful".into(),
                    role: MessageRole::Assistant,
                    blocks: vec![TranscriptBlock::Thinking("inspect the route".into())],
                    streaming: false,
                },
                TranscriptRow {
                    id: "empty".into(),
                    role: MessageRole::Assistant,
                    blocks: vec![TranscriptBlock::Thinking("  ".into())],
                    streaming: false,
                },
            ],
            ..Default::default()
        };

        assert_eq!(transcript.latest_thinking(), Some("inspect the route"));
    }
}
