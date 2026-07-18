use serde::{Deserialize, Serialize};

use super::{ApprovalRequest, MessagePage, QuestionRequest, Session, SessionCursor, Task};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub as_of_seq: u64,
    pub epoch: String,
    pub session: Session,
    pub messages: MessagePage,
    pub in_flight_turn: Option<InFlightTurn>,
    #[serde(default)]
    pub subagents: Vec<Task>,
    #[serde(default)]
    pub pending_approvals: Vec<ApprovalRequest>,
    #[serde(default)]
    pub pending_questions: Vec<QuestionRequest>,
}

impl SessionSnapshot {
    pub fn cursor(&self) -> SessionCursor {
        SessionCursor::new(self.as_of_seq, Some(self.epoch.clone()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InFlightTurn {
    pub turn_id: u64,
    pub assistant_text: String,
    pub thinking_text: String,
    #[serde(default)]
    pub running_tools: Vec<serde_json::Value>,
    #[serde(default)]
    pub current_prompt_id: Option<String>,
}
