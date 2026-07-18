use std::collections::HashMap;

use crate::protocol::{
    ApprovalRequest, Message, QuestionRequest, Session, SessionCursor, SessionSnapshot,
};

#[derive(Debug, Default)]
pub struct AppModel {
    pub(crate) sessions: Vec<Session>,
    pub(crate) conversations: HashMap<String, Conversation>,
    pub(crate) active_session_id: Option<String>,
}

#[derive(Debug)]
pub struct Conversation {
    pub messages: Vec<Message>,
    pub assistant_stream: Option<String>,
    pub thinking_stream: Option<String>,
    pub approvals: Vec<ApprovalRequest>,
    pub questions: Vec<QuestionRequest>,
    pub cursor: SessionCursor,
}

impl AppModel {
    pub fn replace_sessions(&mut self, sessions: Vec<Session>) {
        self.sessions = sessions;
    }

    pub fn add_session(&mut self, session: Session) {
        if let Some(existing) = self.sessions.iter_mut().find(|item| item.id == session.id) {
            *existing = session;
        } else {
            self.sessions.insert(0, session);
        }
    }

    pub fn sessions(&self) -> &[Session] {
        &self.sessions
    }

    pub fn seed(&mut self, snapshot: SessionSnapshot) {
        let session_id = snapshot.session.id.clone();
        let cursor = snapshot.cursor();
        let assistant_stream = snapshot
            .in_flight_turn
            .as_ref()
            .map(|turn| turn.assistant_text.clone())
            .filter(|text| !text.is_empty());
        let thinking_stream = snapshot
            .in_flight_turn
            .as_ref()
            .map(|turn| turn.thinking_text.clone())
            .filter(|text| !text.is_empty());
        let conversation = Conversation {
            messages: snapshot.messages.items,
            assistant_stream,
            thinking_stream,
            approvals: snapshot.pending_approvals,
            questions: snapshot.pending_questions,
            cursor,
        };

        self.add_session(snapshot.session);
        self.conversations.insert(session_id.clone(), conversation);
        self.active_session_id = Some(session_id);
    }

    pub fn active_session(&self) -> Option<&Session> {
        let id = self.active_session_id.as_deref()?;
        self.sessions.iter().find(|session| session.id == id)
    }

    pub fn active_conversation(&self) -> Option<&Conversation> {
        self.conversations.get(self.active_session_id.as_deref()?)
    }
}
