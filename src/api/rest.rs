use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::daemon::Connection;
use crate::protocol::{Page, Session, SessionSnapshot};

use super::ApiError;

const API_PREFIX: &str = "/api/v1";

#[derive(Clone)]
pub struct KimiClient {
    connection: Connection,
    agent: ureq::Agent,
}

impl KimiClient {
    pub fn new(connection: Connection) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(30))
            .build();
        Self { connection, agent }
    }

    pub fn list_sessions(&self) -> Result<Page<Session>, ApiError> {
        self.get("/sessions?page_size=100&include_archive=false")
    }

    pub fn create_session(&self, cwd: &str) -> Result<Session, ApiError> {
        self.post(
            "/sessions",
            &serde_json::json!({ "metadata": { "cwd": cwd } }),
        )
    }

    pub fn snapshot(&self, session_id: &str) -> Result<SessionSnapshot, ApiError> {
        self.get(&format!("/sessions/{}/snapshot", segment(session_id)))
    }

    pub fn submit_prompt(&self, session_id: &str, text: &str) -> Result<PromptResult, ApiError> {
        let body = PromptBody {
            content: vec![PromptText { kind: "text", text }],
        };
        self.post(&format!("/sessions/{}/prompts", segment(session_id)), &body)
    }

    pub fn resolve_approval(
        &self,
        session_id: &str,
        approval_id: &str,
        approved: bool,
    ) -> Result<serde_json::Value, ApiError> {
        let decision = if approved { "approved" } else { "rejected" };
        self.post(
            &format!(
                "/sessions/{}/approvals/{}",
                segment(session_id),
                segment(approval_id)
            ),
            &serde_json::json!({ "decision": decision }),
        )
    }

    pub fn resolve_question(
        &self,
        session_id: &str,
        question_id: &str,
        item_id: &str,
        option_id: &str,
    ) -> Result<serde_json::Value, ApiError> {
        self.post(
            &format!(
                "/sessions/{}/questions/{}",
                segment(session_id),
                segment(question_id)
            ),
            &serde_json::json!({
              "answers": { item_id: { "kind": "single", "option_id": option_id } },
              "method": "click"
            }),
        )
    }

    pub fn abort_session(&self, session_id: &str) -> Result<serde_json::Value, ApiError> {
        self.post(
            &format!("/sessions/{}:abort", segment(session_id)),
            &serde_json::json!({}),
        )
    }

    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let request = self.authorize(self.agent.get(&self.url(path)));
        let response = request.call().map_err(transport_error)?;
        decode(response)
    }

    fn post<T: DeserializeOwned>(&self, path: &str, body: &impl Serialize) -> Result<T, ApiError> {
        let request = self
            .authorize(self.agent.post(&self.url(path)))
            .set("Content-Type", "application/json");
        let response = request.send_json(body).map_err(transport_error)?;
        decode(response)
    }

    fn authorize(&self, request: ureq::Request) -> ureq::Request {
        let request = request
            .set("X-Kimi-Client-Id", "kimini-native")
            .set("X-Kimi-Client-Name", "Kimini")
            .set("X-Kimi-Client-Version", env!("CARGO_PKG_VERSION"))
            .set("X-Kimi-Client-Ui-Mode", "native");
        match self.connection.token() {
            Some(token) => request.set("Authorization", &format!("Bearer {token}")),
            None => request,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}{}", self.connection.origin(), API_PREFIX, path)
    }
}

#[derive(Debug, Deserialize)]
struct Envelope<T> {
    code: i64,
    msg: String,
    data: Option<T>,
}

fn decode<T: DeserializeOwned>(response: ureq::Response) -> Result<T, ApiError> {
    let envelope: Envelope<T> = response
        .into_json()
        .map_err(|error| ApiError::InvalidResponse(error.to_string()))?;
    if envelope.code != 0 {
        return Err(ApiError::Daemon {
            code: envelope.code,
            message: envelope.msg,
        });
    }
    envelope.data.ok_or(ApiError::MissingData)
}

fn transport_error(error: ureq::Error) -> ApiError {
    ApiError::Transport(error.to_string())
}

fn segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes())
        .collect::<String>()
        .replace('+', "%20")
}

#[derive(Serialize)]
struct PromptBody<'a> {
    content: Vec<PromptText<'a>>,
}

#[derive(Serialize)]
struct PromptText<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    text: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PromptResult {
    pub prompt_id: String,
    pub user_message_id: String,
    #[serde(default)]
    pub status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_segments_are_encoded_without_touching_safe_ids() {
        assert_eq!(segment("sess_01"), "sess_01");
        assert_eq!(segment("a/b c"), "a%2Fb%20c");
    }
}
