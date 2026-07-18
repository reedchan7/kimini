use crate::protocol::{MessagePage, Page, Session, SessionSnapshot, SessionStatus};

use super::{ApiError, KimiClient, segment};

impl KimiClient {
    pub fn list_sessions(&self) -> Result<Page<Session>, ApiError> {
        self.get("/sessions?page_size=100&include_archive=false")
    }

    pub fn list_sessions_before(&self, session_id: &str) -> Result<Page<Session>, ApiError> {
        self.get(&format!(
            "/sessions?before_id={}&page_size=100&include_archive=false",
            segment(session_id)
        ))
    }

    pub fn list_archived_sessions(&self) -> Result<Page<Session>, ApiError> {
        self.get("/sessions?page_size=100&archived_only=true")
    }

    pub fn list_archived_sessions_before(
        &self,
        session_id: &str,
    ) -> Result<Page<Session>, ApiError> {
        self.get(&format!(
            "/sessions?before_id={}&page_size=100&archived_only=true",
            segment(session_id)
        ))
    }

    pub fn create_session(&self, cwd: &str, model: Option<&str>) -> Result<Session, ApiError> {
        let mut body = serde_json::json!({ "metadata": { "cwd": cwd } });
        if let Some(model) = model.filter(|model| !model.is_empty()) {
            body["agent_config"] = serde_json::json!({ "model": model });
        }
        self.post("/sessions", &body)
    }

    pub fn snapshot(&self, session_id: &str) -> Result<SessionSnapshot, ApiError> {
        self.get(&format!("/sessions/{}/snapshot", segment(session_id)))
    }

    pub fn list_messages_before(
        &self,
        session_id: &str,
        message_id: &str,
    ) -> Result<MessagePage, ApiError> {
        self.get(&format!(
            "/sessions/{}/messages?before_id={}&page_size=100",
            segment(session_id),
            segment(message_id)
        ))
    }

    pub fn session_status(&self, session_id: &str) -> Result<SessionStatus, ApiError> {
        self.get(&format!("/sessions/{}/status", segment(session_id)))
    }

    pub fn update_session_config(
        &self,
        session_id: &str,
        agent_config: serde_json::Value,
    ) -> Result<Session, ApiError> {
        self.update_session_profile(
            session_id,
            serde_json::json!({ "agent_config": agent_config }),
        )
    }

    pub fn rename_session(&self, session_id: &str, title: &str) -> Result<Session, ApiError> {
        self.update_session_profile(session_id, serde_json::json!({ "title": title }))
    }

    fn update_session_profile(
        &self,
        session_id: &str,
        body: serde_json::Value,
    ) -> Result<Session, ApiError> {
        self.post(&format!("/sessions/{}/profile", segment(session_id)), &body)
    }

    pub fn abort_session(&self, session_id: &str) -> Result<serde_json::Value, ApiError> {
        self.post(
            &format!("/sessions/{}:abort", segment(session_id)),
            &serde_json::json!({}),
        )
    }

    pub fn fork_session(&self, session_id: &str) -> Result<Session, ApiError> {
        self.post(
            &format!("/sessions/{}:fork", segment(session_id)),
            &serde_json::json!({}),
        )
    }

    pub fn compact_session(&self, session_id: &str) -> Result<serde_json::Value, ApiError> {
        self.compact_session_with_instruction(session_id, None)
    }

    pub fn compact_session_with_instruction(
        &self,
        session_id: &str,
        instruction: Option<&str>,
    ) -> Result<serde_json::Value, ApiError> {
        self.post(
            &format!("/sessions/{}:compact", segment(session_id)),
            &match instruction {
                Some(instruction) => serde_json::json!({ "instruction": instruction }),
                None => serde_json::json!({}),
            },
        )
    }

    pub fn undo_session(&self, session_id: &str) -> Result<serde_json::Value, ApiError> {
        self.post(
            &format!("/sessions/{}:undo", segment(session_id)),
            &serde_json::json!({ "count": 1, "page_size": 100 }),
        )
    }

    pub fn archive_session(&self, session_id: &str) -> Result<serde_json::Value, ApiError> {
        self.post(
            &format!("/sessions/{}:archive", segment(session_id)),
            &serde_json::json!({}),
        )
    }

    pub fn restore_session(&self, session_id: &str) -> Result<Session, ApiError> {
        self.post(
            &format!("/sessions/{}:restore", segment(session_id)),
            &serde_json::json!({}),
        )
    }
}
