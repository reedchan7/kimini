use std::collections::HashMap;

use crate::protocol::{
    ApprovalRequest, GoalSnapshot, Message, MessagePage, Page, QuestionRequest, Session,
    SessionCursor, SessionSnapshot, SessionStatus,
};

#[derive(Debug, Default)]
pub struct AppModel {
    pub(crate) sessions: Vec<Session>,
    pub(crate) has_more_sessions: bool,
    pub(crate) archived_sessions: Vec<Session>,
    pub(crate) has_more_archived_sessions: bool,
    pub(crate) archived_sessions_loaded: bool,
    pub(crate) conversations: HashMap<String, Conversation>,
    pub(crate) active_session_id: Option<String>,
}

#[derive(Debug)]
pub struct Conversation {
    pub messages: Vec<Message>,
    pub has_more_messages: bool,
    pub assistant_stream: Option<String>,
    pub thinking_stream: Option<String>,
    pub approvals: Vec<ApprovalRequest>,
    pub questions: Vec<QuestionRequest>,
    pub cursor: SessionCursor,
    pub runtime: Option<SessionStatus>,
    pub goal: Option<GoalSnapshot>,
}

impl AppModel {
    pub fn replace_sessions(&mut self, sessions: Vec<Session>) {
        self.replace_session_page(Page {
            items: sessions,
            has_more: false,
        });
    }

    pub fn replace_session_page(&mut self, page: Page<Session>) {
        self.sessions = page.items;
        self.has_more_sessions = page.has_more;
    }

    pub fn append_session_page(&mut self, page: Page<Session>) -> usize {
        let mut added = 0;
        for session in page.items {
            if self.sessions.iter().any(|current| current.id == session.id) {
                continue;
            }
            self.sessions.push(session);
            added += 1;
        }
        self.has_more_sessions = page.has_more && added > 0;
        added
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

    pub fn has_more_sessions(&self) -> bool {
        self.has_more_sessions
    }

    pub fn replace_archived_session_page(&mut self, page: Page<Session>) {
        self.archived_sessions = page.items;
        self.has_more_archived_sessions = page.has_more;
        self.archived_sessions_loaded = true;
    }

    pub fn append_archived_session_page(&mut self, page: Page<Session>) -> usize {
        let mut added = 0;
        for session in page.items {
            if self
                .archived_sessions
                .iter()
                .any(|current| current.id == session.id)
            {
                continue;
            }
            self.archived_sessions.push(session);
            added += 1;
        }
        self.has_more_archived_sessions = page.has_more && added > 0;
        self.archived_sessions_loaded = true;
        added
    }

    pub fn archived_sessions(&self) -> &[Session] {
        &self.archived_sessions
    }

    pub fn archived_sessions_loaded(&self) -> bool {
        self.archived_sessions_loaded
    }

    pub fn has_more_archived_sessions(&self) -> bool {
        self.has_more_archived_sessions
    }

    pub fn remove_archived_session(&mut self, session_id: &str) {
        self.archived_sessions
            .retain(|session| session.id != session_id);
    }

    pub fn invalidate_archived_sessions(&mut self) {
        self.archived_sessions.clear();
        self.has_more_archived_sessions = false;
        self.archived_sessions_loaded = false;
    }

    pub fn remove_session(&mut self, session_id: &str) -> Option<String> {
        self.sessions.retain(|session| session.id != session_id);
        self.conversations.remove(session_id);
        if self.active_session_id.as_deref() == Some(session_id) {
            self.active_session_id = self.sessions.first().map(|session| session.id.clone());
        }
        self.active_session_id.clone()
    }

    pub fn seed(&mut self, snapshot: SessionSnapshot) {
        let session_id = snapshot.session.id.clone();
        let cursor = snapshot.cursor();
        let has_more_messages = snapshot.messages.has_more;
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
            has_more_messages,
            assistant_stream,
            thinking_stream,
            approvals: snapshot.pending_approvals,
            questions: snapshot.pending_questions,
            cursor,
            runtime: None,
            goal: None,
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

    pub fn set_runtime(&mut self, session_id: &str, runtime: SessionStatus) {
        if let Some(conversation) = self.conversations.get_mut(session_id) {
            conversation.runtime = Some(runtime);
        }
    }

    pub fn active_runtime(&self) -> Option<&SessionStatus> {
        self.active_conversation()?.runtime.as_ref()
    }

    pub fn set_goal(&mut self, session_id: &str, goal: Option<GoalSnapshot>) {
        if let Some(conversation) = self.conversations.get_mut(session_id) {
            conversation.goal = goal.filter(|goal| !goal.is_complete());
        }
    }

    pub fn active_goal(&self) -> Option<&GoalSnapshot> {
        self.active_conversation()?.goal.as_ref()
    }

    pub fn prepend_messages(&mut self, session_id: &str, mut page: MessagePage) -> bool {
        let Some(conversation) = self.conversations.get_mut(session_id) else {
            return false;
        };
        page.items.reverse();
        page.items.retain(|candidate| {
            !conversation
                .messages
                .iter()
                .any(|message| message.id == candidate.id)
        });
        let added = page.items.len();
        conversation.messages.splice(0..0, page.items);
        conversation.has_more_messages = page.has_more && added > 0;
        true
    }
}
