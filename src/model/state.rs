use std::collections::HashMap;

use crate::protocol::{
    ApprovalRequest, GoalSnapshot, Message, MessagePage, Page, PromptPart, QuestionRequest,
    Session, SessionCursor, SessionSnapshot, SessionStatus, Workspace,
};

#[derive(Debug, Default)]
pub struct AppModel {
    pub(crate) workspaces: Vec<Workspace>,
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
    pub(crate) optimistic_user: Option<OptimisticUserMessage>,
    pub approvals: Vec<ApprovalRequest>,
    pub questions: Vec<QuestionRequest>,
    pub cursor: SessionCursor,
    pub runtime: Option<SessionStatus>,
    pub goal: Option<GoalSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OptimisticUserMessage {
    pub id: String,
    pub content: Vec<PromptPart>,
}

impl AppModel {
    pub fn replace_workspaces(&mut self, workspaces: Vec<Workspace>) {
        self.workspaces = workspaces;
    }

    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }

    pub fn upsert_workspace(&mut self, workspace: Workspace) {
        if let Some(existing) = self
            .workspaces
            .iter_mut()
            .find(|item| item.id == workspace.id)
        {
            *existing = workspace;
        } else {
            self.workspaces.push(workspace);
        }
    }

    pub fn remove_workspace(&mut self, workspace_id: &str) -> bool {
        let removed_root = self
            .workspaces
            .iter()
            .find(|workspace| workspace.id == workspace_id)
            .map(|workspace| workspace.root.as_str());
        let removed_active_session = self.active_session().is_some_and(|session| {
            session.workspace_id == workspace_id
                || removed_root.is_some_and(|root| session.metadata.cwd == root)
        });
        self.workspaces.retain(|item| item.id != workspace_id);
        if removed_active_session {
            self.active_session_id = None;
        }
        removed_active_session
    }

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
        let in_flight_turn = snapshot.in_flight_turn.as_ref();
        let turn_active = snapshot.session.busy
            || snapshot.session.main_turn_active.unwrap_or(false)
            || in_flight_turn.is_some();
        // Daemon 0.27+ often omits assistant.delta / message.created from the
        // event journal. While a turn is still active, keep an empty stream so
        // the UI continues to show the waiting indicator until settlement.
        let assistant_stream = match in_flight_turn {
            Some(turn) => Some(turn.assistant_text.clone()),
            None if turn_active => Some(String::new()),
            None => None,
        };
        let thinking_stream = in_flight_turn
            .map(|turn| turn.thinking_text.clone())
            .filter(|text| !text.is_empty());
        let conversation = Conversation {
            messages: snapshot.messages.items,
            has_more_messages,
            assistant_stream,
            thinking_stream,
            optimistic_user: None,
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

    pub fn begin_prompt(&mut self, session_id: &str, content: Vec<PromptPart>) -> bool {
        let Some(conversation) = self.conversations.get_mut(session_id) else {
            return false;
        };
        conversation.optimistic_user = Some(OptimisticUserMessage {
            id: format!("optimistic-user:{session_id}"),
            content,
        });
        conversation.assistant_stream = Some(String::new());
        conversation.thinking_stream = None;
        true
    }

    pub fn activate_submitted_session(
        &mut self,
        session: Session,
        content: Vec<PromptPart>,
        user_message_id: String,
    ) {
        let session_id = session.id.clone();
        self.add_session(session);
        self.conversations.insert(
            session_id.clone(),
            Conversation {
                messages: Vec::new(),
                has_more_messages: false,
                assistant_stream: Some(String::new()),
                thinking_stream: None,
                optimistic_user: Some(OptimisticUserMessage {
                    id: user_message_id,
                    content,
                }),
                approvals: Vec::new(),
                questions: Vec::new(),
                cursor: SessionCursor::new(0, None),
                runtime: None,
                goal: None,
            },
        );
        self.active_session_id = Some(session_id);
    }

    pub fn accept_prompt(&mut self, session_id: &str, user_message_id: &str) {
        let Some(conversation) = self.conversations.get_mut(session_id) else {
            return;
        };
        if conversation
            .messages
            .iter()
            .any(|message| message.id == user_message_id)
        {
            conversation.optimistic_user = None;
        } else if let Some(message) = conversation.optimistic_user.as_mut() {
            message.id = user_message_id.into();
        }
    }

    pub fn fail_prompt(&mut self, session_id: &str) {
        let Some(conversation) = self.conversations.get_mut(session_id) else {
            return;
        };
        conversation.optimistic_user = None;
        conversation.assistant_stream = None;
        conversation.thinking_stream = None;
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
