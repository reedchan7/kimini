use std::collections::HashMap;

use crate::protocol::{MessageContent, PromptItem, PromptQueue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct QueuedPrompt {
    pub id: String,
    pub text: String,
    pub attachment_count: usize,
}

impl From<&PromptItem> for QueuedPrompt {
    fn from(prompt: &PromptItem) -> Self {
        let text = prompt
            .content
            .iter()
            .filter_map(|part| match part {
                MessageContent::Text(text) => Some(text.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        let attachment_count = prompt
            .content
            .iter()
            .filter(|part| {
                matches!(
                    part,
                    MessageContent::Image(_) | MessageContent::Video(_) | MessageContent::File(_)
                )
            })
            .count();
        Self {
            id: prompt.prompt_id.clone(),
            text,
            attachment_count,
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct PromptQueues {
    by_session: HashMap<String, PromptQueue>,
}

impl PromptQueues {
    pub fn replace(&mut self, session_id: String, queue: PromptQueue) {
        self.by_session.insert(session_id, queue);
    }

    pub fn queued(&self, session_id: &str) -> Vec<QueuedPrompt> {
        self.by_session
            .get(session_id)
            .map(|queue| queue.queued.iter().map(QueuedPrompt::from).collect())
            .unwrap_or_default()
    }

    pub fn queued_ids(&self, session_id: &str) -> Vec<String> {
        self.by_session
            .get(session_id)
            .map(|queue| {
                queue
                    .queued
                    .iter()
                    .map(|prompt| prompt.prompt_id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queue() -> PromptQueue {
        serde_json::from_value(serde_json::json!({
            "active": null,
            "queued": [{
                "prompt_id": "p1",
                "user_message_id": "m1",
                "status": "queued",
                "content": [
                    { "type": "text", "text": "follow up" },
                    { "type": "image", "source": { "kind": "file", "file_id": "f1" } }
                ],
                "created_at": "2026-07-18T08:00:00.000Z"
            }]
        }))
        .unwrap()
    }

    #[test]
    fn projects_server_queue_without_leaking_between_sessions() {
        let mut queues = PromptQueues::default();
        queues.replace("a".into(), queue());

        assert_eq!(
            queues.queued("a"),
            vec![QueuedPrompt {
                id: "p1".into(),
                text: "follow up".into(),
                attachment_count: 1,
            }]
        );
        assert!(queues.queued("b").is_empty());
        assert_eq!(queues.queued_ids("a"), vec!["p1"]);
    }
}
