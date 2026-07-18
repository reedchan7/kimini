use std::collections::{HashMap, HashSet};

use crate::protocol::WireEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SideChatTurn {
    pub user: String,
    pub assistant: String,
    pub thinking: String,
}

#[derive(Debug)]
pub(super) struct SideChat {
    pub agent_id: String,
    pub turns: Vec<SideChatTurn>,
    pub sending: bool,
    pub running: bool,
    pub error: Option<String>,
}

#[derive(Debug, Default)]
pub(super) struct SideChats {
    by_session: HashMap<String, SideChat>,
    opening: HashSet<String>,
    open_errors: HashMap<String, String>,
}

impl SideChats {
    pub fn get(&self, session_id: &str) -> Option<&SideChat> {
        self.by_session.get(session_id)
    }

    pub fn begin(&mut self, session_id: String, agent_id: String) {
        self.opening.remove(&session_id);
        self.open_errors.remove(&session_id);
        self.by_session.entry(session_id).or_insert(SideChat {
            agent_id,
            turns: Vec::new(),
            sending: false,
            running: false,
            error: None,
        });
    }

    pub fn begin_open(&mut self, session_id: &str) -> bool {
        if self.by_session.contains_key(session_id) || !self.opening.insert(session_id.into()) {
            return false;
        }
        self.open_errors.remove(session_id);
        true
    }

    pub fn fail_open(&mut self, session_id: &str, error: String) {
        self.opening.remove(session_id);
        self.open_errors.insert(session_id.into(), error);
    }

    pub fn is_opening(&self, session_id: &str) -> bool {
        self.opening.contains(session_id)
    }

    pub fn open_error(&self, session_id: &str) -> Option<&str> {
        self.open_errors.get(session_id).map(String::as_str)
    }

    pub fn start_turn(&mut self, session_id: &str, text: String) -> bool {
        let Some(chat) = self.by_session.get_mut(session_id) else {
            return false;
        };
        chat.turns.push(SideChatTurn {
            user: text,
            assistant: String::new(),
            thinking: String::new(),
        });
        chat.sending = true;
        chat.running = true;
        chat.error = None;
        true
    }

    pub fn fail_turn(&mut self, session_id: &str, error: String) {
        if let Some(chat) = self.by_session.get_mut(session_id) {
            chat.turns.pop();
            chat.sending = false;
            chat.running = false;
            chat.error = Some(error);
        }
    }

    pub fn finish_turn(&mut self, session_id: &str) {
        if let Some(chat) = self.by_session.get_mut(session_id) {
            chat.sending = false;
            chat.running = false;
        }
    }

    pub fn set_error(&mut self, session_id: &str, error: String) {
        if let Some(chat) = self.by_session.get_mut(session_id) {
            chat.error = Some(error);
        }
    }

    pub fn owns_event(&self, event: &WireEvent) -> bool {
        let Some(session_id) = event.session_id.as_deref() else {
            return false;
        };
        let Some(chat) = self.by_session.get(session_id) else {
            return false;
        };
        let event_agent = string_field(&event.payload, "agentId", "agent_id");
        let subagent = string_field(&event.payload, "subagentId", "subagent_id");
        event_agent == Some(chat.agent_id.as_str()) || subagent == Some(chat.agent_id.as_str())
    }

    pub fn apply_event(&mut self, event: &WireEvent) -> bool {
        if !self.owns_event(event) {
            return false;
        }
        let Some(session_id) = event.session_id.as_deref() else {
            return false;
        };
        let Some(chat) = self.by_session.get_mut(session_id) else {
            return false;
        };
        let kind = event.kind.strip_prefix("event.").unwrap_or(&event.kind);

        match kind {
            "assistant.delta" => {
                let delta = delta_text(&event.payload);
                let Some(turn) = chat.turns.last_mut() else {
                    return false;
                };
                chat.sending = false;
                append_delta(&mut turn.assistant, event.offset, delta)
            }
            "thinking.delta" => {
                let delta = delta_text(&event.payload);
                let Some(turn) = chat.turns.last_mut() else {
                    return false;
                };
                chat.sending = false;
                append_delta(&mut turn.thinking, event.offset, delta)
            }
            "turn.ended" => {
                chat.sending = false;
                chat.running = false;
                true
            }
            "subagent.completed" => {
                chat.sending = false;
                chat.running = false;
                if let Some(summary) =
                    string_field(&event.payload, "resultSummary", "result_summary")
                    && let Some(turn) = chat.turns.last_mut()
                    && turn.assistant.is_empty()
                {
                    turn.assistant.push_str(summary);
                }
                true
            }
            "subagent.failed" => {
                chat.sending = false;
                chat.running = false;
                chat.error = string_field(&event.payload, "error", "error").map(str::to_owned);
                true
            }
            _ => false,
        }
    }
}

fn string_field<'a>(payload: &'a serde_json::Value, camel: &str, snake: &str) -> Option<&'a str> {
    payload
        .get(camel)
        .or_else(|| payload.get(snake))
        .and_then(serde_json::Value::as_str)
}

fn delta_text(payload: &serde_json::Value) -> Option<&str> {
    payload
        .get("delta")
        .and_then(|delta| delta.as_str().or_else(|| delta.get("text")?.as_str()))
}

fn append_delta(target: &mut String, offset: Option<usize>, delta: Option<&str>) -> bool {
    let Some(delta) = delta else { return false };
    if let Some(offset) = offset {
        let mut length = target.encode_utf16().count();
        if offset == 0 && length > 0 {
            target.clear();
            length = 0;
        }
        if offset < length {
            return true;
        }
        if offset > length {
            return false;
        }
    }
    target.push_str(delta);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: &str, agent_id: &str, delta: Option<&str>) -> WireEvent {
        serde_json::from_value(serde_json::json!({
            "type": kind,
            "session_id": "session",
            "timestamp": "now",
            "payload": { "agentId": agent_id, "delta": delta }
        }))
        .unwrap()
    }

    #[test]
    fn side_channel_deltas_stay_isolated_by_agent_and_turn() {
        let mut chats = SideChats::default();
        chats.begin("session".into(), "btw".into());
        assert!(chats.start_turn("session", "question".into()));
        assert!(!chats.apply_event(&event("assistant.delta", "other", Some("wrong"))));
        assert!(chats.apply_event(&event("thinking.delta", "btw", Some("reason"))));
        assert!(chats.apply_event(&event("assistant.delta", "btw", Some("answer"))));
        assert!(chats.apply_event(&event("turn.ended", "btw", None)));

        let chat = chats.get("session").unwrap();
        assert_eq!(chat.turns[0].thinking, "reason");
        assert_eq!(chat.turns[0].assistant, "answer");
        assert!(!chat.running);
    }

    #[test]
    fn failed_submissions_remove_only_the_optimistic_turn() {
        let mut chats = SideChats::default();
        chats.begin("session".into(), "btw".into());
        chats.start_turn("session", "question".into());
        chats.fail_turn("session", "offline".into());
        let chat = chats.get("session").unwrap();
        assert!(chat.turns.is_empty());
        assert_eq!(chat.error.as_deref(), Some("offline"));
    }

    #[test]
    fn opening_state_is_per_session_and_recoverable() {
        let mut chats = SideChats::default();
        assert!(chats.begin_open("one"));
        assert!(!chats.begin_open("one"));
        assert!(chats.is_opening("one"));
        chats.fail_open("one", "offline".into());
        assert!(!chats.is_opening("one"));
        assert_eq!(chats.open_error("one"), Some("offline"));
        assert!(chats.begin_open("one"));
        chats.begin("one".into(), "btw".into());
        assert!(!chats.is_opening("one"));
        assert!(chats.open_error("one").is_none());
    }
}
