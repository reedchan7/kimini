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
        if let Some(message) = conversation.optimistic_user.as_ref()
            && !conversation
                .messages
                .iter()
                .any(|existing| existing.id == message.id)
        {
            rows.push(TranscriptRow::from_optimistic_user(message));
        }
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
            self.restore_followed_tail();
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
            self.restore_followed_tail();
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
        self.restore_followed_tail();
    }

    fn restore_followed_tail(&self) {
        if self.list.is_following_tail() {
            self.list.scroll_to_end();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MessageRole, PromptPart, SessionSnapshot, WireEvent};

    fn snapshot() -> SessionSnapshot {
        serde_json::from_value(serde_json::json!({
            "as_of_seq": 7,
            "epoch": "ep_01",
            "session": {
                "id": "session",
                "workspace_id": "workspace",
                "title": "Test",
                "created_at": "2026-07-20T00:00:00Z",
                "updated_at": "2026-07-20T00:00:00Z",
                "busy": false,
                "metadata": { "cwd": "/workspace/kimini" },
                "agent_config": { "model": "kimi-code/k2p5" },
                "usage": {
                    "input_tokens": 0,
                    "output_tokens": 0,
                    "cache_read_tokens": 0,
                    "cache_creation_tokens": 0,
                    "total_cost_usd": 0,
                    "context_tokens": 0,
                    "context_limit": 131072,
                    "turn_count": 0
                },
                "permission_rules": [],
                "message_count": 1,
                "last_seq": 7
            },
            "messages": {
                "items": [{
                    "id": "existing",
                    "session_id": "session",
                    "role": "user",
                    "content": [{ "type": "text", "text": "existing" }],
                    "created_at": "2026-07-20T00:00:00Z"
                }],
                "has_more": false
            },
            "in_flight_turn": null,
            "pending_approvals": [],
            "pending_questions": []
        }))
        .unwrap()
    }

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

    #[test]
    fn submitted_prompt_is_visible_until_the_snapshot_reconciles_it() {
        let mut model = AppModel::default();
        model.seed(snapshot());
        assert!(model.begin_prompt("session", vec![PromptPart::text("next question")]));

        let mut transcript = Transcript::default();
        transcript.rebuild(&model);
        assert_eq!(transcript.rows[1].accessible_text(), "next question");
        assert_eq!(transcript.rows[1].role, MessageRole::User);
        assert!(transcript.rows[2].streaming);
        assert!(transcript.rows[2].blocks.is_empty());
        assert_eq!(
            transcript.list.logical_scroll_top().item_ix,
            transcript.rows.len(),
            "a followed transcript must keep its tail anchor after optimistic rows are inserted"
        );
        let pending_id = transcript.rows[2].id.clone();

        model.accept_prompt("session", "server-user-message");
        transcript.rebuild(&model);
        assert_eq!(transcript.rows[1].id, "server-user-message");

        assert_eq!(
            model.apply(WireEvent {
                kind: "assistant.delta".into(),
                seq: 7,
                epoch: Some("ep_01".into()),
                volatile: true,
                offset: Some(0),
                session_id: Some("session".into()),
                timestamp: "2026-07-20T00:00:01Z".into(),
                payload: serde_json::json!({ "agentId": "main", "delta": "answer" }),
            }),
            crate::model::ApplyOutcome::Applied
        );
        transcript.sync_stream(&model);
        assert_eq!(transcript.rows[2].id, pending_id);
        assert_eq!(transcript.rows[2].accessible_text(), "answer");

        model.seed(snapshot());
        transcript.rebuild(&model);
        assert_eq!(transcript.rows.len(), 1);
        assert!(!transcript.rows.iter().any(|row| row.streaming));

        assert!(model.begin_prompt("session", vec![PromptPart::text("failed prompt")]));
        model.fail_prompt("session");
        transcript.rebuild(&model);
        assert_eq!(transcript.rows.len(), 1);

        assert!(model.begin_prompt("session", vec![PromptPart::text("echo race")]));
        assert_eq!(
            model.apply(WireEvent {
                kind: "event.message.created".into(),
                seq: 7,
                epoch: Some("ep_01".into()),
                volatile: true,
                offset: None,
                session_id: Some("session".into()),
                timestamp: "2026-07-20T00:00:02Z".into(),
                payload: serde_json::json!({
                    "message": {
                        "id": "echoed-user",
                        "session_id": "session",
                        "role": "user",
                        "content": [{ "type": "text", "text": "echo race" }],
                        "created_at": "2026-07-20T00:00:02Z"
                    }
                }),
            }),
            crate::model::ApplyOutcome::Applied
        );
        transcript.rebuild(&model);
        assert_eq!(
            transcript
                .rows
                .iter()
                .filter(|row| row.accessible_text() == "echo race")
                .count(),
            1,
            "an early server echo must replace the optimistic user row"
        );

        let new_session = snapshot().session;
        model.activate_submitted_session(
            new_session,
            vec![PromptPart::text("first question")],
            "new-session-user-message".into(),
        );
        transcript.rebuild(&model);
        assert_eq!(transcript.rows[0].id, "new-session-user-message");
        assert_eq!(transcript.rows[0].accessible_text(), "first question");
        assert!(transcript.rows[1].streaming);
        assert!(transcript.rows[1].blocks.is_empty());
    }
}
