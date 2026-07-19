use serde::de::DeserializeOwned;

use crate::protocol::{ApprovalRequest, GoalSnapshot, Message, QuestionRequest, WireEvent};

use super::AppModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    Applied,
    Duplicate,
    ResyncRequired,
    Irrelevant,
}

impl AppModel {
    pub fn apply(&mut self, event: WireEvent) -> ApplyOutcome {
        let Some(session_id) = event.session_id.as_deref() else {
            return ApplyOutcome::Irrelevant;
        };
        let Some(conversation) = self.conversations.get_mut(session_id) else {
            if event.kind == "event.session.work_changed"
                && update_session_work(&mut self.sessions, session_id, &event.payload)
            {
                return ApplyOutcome::Applied;
            }
            return if self.active_session_id.as_deref() == Some(session_id) {
                ApplyOutcome::ResyncRequired
            } else {
                ApplyOutcome::Irrelevant
            };
        };

        if !event.volatile {
            if event.seq <= conversation.cursor.seq {
                return ApplyOutcome::Duplicate;
            }
            if event.seq != conversation.cursor.seq + 1 {
                return ApplyOutcome::ResyncRequired;
            }
        }

        match event.kind.as_str() {
            "turn.started"
            | "event.turn.started"
            | "turn.step.started"
            | "event.turn.step.started"
            | "turn.step.retrying"
            | "event.turn.step.retrying"
                if is_main_agent(&event) =>
            {
                conversation.assistant_stream = Some(String::new());
                conversation.thinking_stream = None;
            }
            "event.message.created" => {
                if let Some(message) = field::<Message>(&event, "message") {
                    if message.role == crate::protocol::MessageRole::Assistant {
                        conversation.assistant_stream = None;
                        conversation.thinking_stream = None;
                    } else if message.role == crate::protocol::MessageRole::User
                        && conversation.optimistic_user.is_some()
                    {
                        conversation.optimistic_user = None;
                    }
                    upsert_message(&mut conversation.messages, message);
                }
            }
            "assistant.delta" | "event.assistant.delta" if is_main_agent(&event) => {
                let delta = text_delta(&event);
                if !append_delta(&mut conversation.assistant_stream, event.offset, delta) {
                    return ApplyOutcome::ResyncRequired;
                }
            }
            "thinking.delta" | "event.thinking.delta" if is_main_agent(&event) => {
                conversation
                    .assistant_stream
                    .get_or_insert_with(String::new);
                let delta = event.payload.get("delta").and_then(|value| value.as_str());
                if !append_delta(&mut conversation.thinking_stream, event.offset, delta) {
                    return ApplyOutcome::ResyncRequired;
                }
            }
            "event.approval.requested" => {
                if let Ok(request) =
                    serde_json::from_value::<ApprovalRequest>(event.payload.clone())
                {
                    replace_by(&mut conversation.approvals, request, |item| {
                        &item.approval_id
                    });
                }
            }
            "event.approval.resolved" | "event.approval.expired" => {
                if let Some(id) = event
                    .payload
                    .get("approval_id")
                    .and_then(|value| value.as_str())
                {
                    conversation.approvals.retain(|item| item.approval_id != id);
                }
            }
            "event.question.requested" => {
                if let Ok(request) =
                    serde_json::from_value::<QuestionRequest>(event.payload.clone())
                {
                    replace_by(&mut conversation.questions, request, |item| {
                        &item.question_id
                    });
                }
            }
            "event.question.answered" | "event.question.dismissed" => {
                if let Some(id) = event
                    .payload
                    .get("question_id")
                    .and_then(|value| value.as_str())
                {
                    conversation.questions.retain(|item| item.question_id != id);
                }
            }
            "event.session.work_changed" => {
                update_session_work(&mut self.sessions, session_id, &event.payload);
            }
            "goal.updated" | "event.goal.updated" => {
                conversation.goal = event
                    .payload
                    .get("snapshot")
                    .filter(|snapshot| !snapshot.is_null())
                    .and_then(|snapshot| {
                        serde_json::from_value::<GoalSnapshot>(snapshot.clone()).ok()
                    })
                    .filter(|goal| !goal.is_complete());
            }
            _ => {}
        }

        if !event.volatile {
            conversation.cursor.seq = event.seq;
            if let Some(epoch) = event.epoch {
                conversation.cursor.epoch = Some(epoch);
            }
        }
        ApplyOutcome::Applied
    }
}

fn field<T: DeserializeOwned>(event: &WireEvent, name: &str) -> Option<T> {
    serde_json::from_value(event.payload.get(name)?.clone()).ok()
}

fn text_delta(event: &WireEvent) -> Option<&str> {
    event
        .payload
        .get("delta")
        .and_then(|delta| delta.as_str().or_else(|| delta.get("text")?.as_str()))
}

fn is_main_agent(event: &WireEvent) -> bool {
    event
        .payload
        .get("agentId")
        .or_else(|| event.payload.get("agent_id"))
        .and_then(serde_json::Value::as_str)
        .is_none_or(|agent_id| agent_id == "main")
}

fn update_session_work(
    sessions: &mut [crate::protocol::Session],
    session_id: &str,
    payload: &serde_json::Value,
) -> bool {
    let Some(session) = sessions.iter_mut().find(|item| item.id == session_id) else {
        return false;
    };
    if let Some(busy) = payload.get("busy").and_then(serde_json::Value::as_bool) {
        session.busy = busy;
    }
    if let Some(active) = payload
        .get("main_turn_active")
        .and_then(serde_json::Value::as_bool)
    {
        session.main_turn_active = Some(active);
    }
    if let Some(reason) = payload
        .get("last_turn_reason")
        .and_then(serde_json::Value::as_str)
    {
        session.last_turn_reason = Some(reason.to_owned());
    }
    true
}

fn append_delta(target: &mut Option<String>, offset: Option<usize>, delta: Option<&str>) -> bool {
    let Some(delta) = delta else { return true };
    let text = target.get_or_insert_with(String::new);
    if let Some(offset) = offset {
        let mut length = text.encode_utf16().count();
        if offset == 0 && length > 0 {
            text.clear();
            length = 0;
        }
        if offset < length {
            return true;
        }
        if offset > length {
            return false;
        }
    }
    text.push_str(delta);
    true
}

fn upsert_message(messages: &mut Vec<Message>, message: Message) {
    if let Some(existing) = messages.iter_mut().find(|item| item.id == message.id) {
        *existing = message;
    } else {
        messages.push(message);
    }
}

fn replace_by<T>(items: &mut Vec<T>, item: T, id: impl Fn(&T) -> &str) {
    let item_id = id(&item).to_owned();
    if let Some(existing) = items.iter_mut().find(|existing| id(existing) == item_id) {
        *existing = item;
    } else {
        items.push(item);
    }
}
