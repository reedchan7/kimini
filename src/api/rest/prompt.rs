use serde::{Deserialize, Serialize};

use crate::protocol::{
    PromptAbortResult, PromptOptions, PromptPart, PromptQueue, PromptSteerResult,
};

use super::{ApiError, KimiClient, decode_with_allowed_codes, segment, transport_error};

impl KimiClient {
    pub fn submit_prompt(&self, session_id: &str, text: &str) -> Result<PromptResult, ApiError> {
        self.submit_prompt_parts(session_id, &[PromptPart::text(text)])
    }

    pub fn submit_prompt_parts(
        &self,
        session_id: &str,
        content: &[PromptPart],
    ) -> Result<PromptResult, ApiError> {
        self.submit_prompt_with_options(session_id, content, &PromptOptions::default())
    }

    pub fn submit_prompt_with_options(
        &self,
        session_id: &str,
        content: &[PromptPart],
        options: &PromptOptions,
    ) -> Result<PromptResult, ApiError> {
        self.post(
            &format!("/sessions/{}/prompts", segment(session_id)),
            &PromptBody { content, options },
        )
    }

    pub fn list_prompts(&self, session_id: &str) -> Result<PromptQueue, ApiError> {
        self.get(&format!("/sessions/{}/prompts", segment(session_id)))
    }

    pub fn steer_prompts(
        &self,
        session_id: &str,
        prompt_ids: &[String],
    ) -> Result<PromptSteerResult, ApiError> {
        self.post(
            &format!("/sessions/{}/prompts::steer", segment(session_id)),
            &serde_json::json!({ "prompt_ids": prompt_ids }),
        )
    }

    pub fn abort_prompt(
        &self,
        session_id: &str,
        prompt_id: &str,
    ) -> Result<PromptAbortResult, ApiError> {
        let request = self
            .authorize(self.agent.post(&self.url(&format!(
                "/sessions/{}/prompts/{}:abort",
                segment(session_id),
                segment(prompt_id)
            ))))
            .set("Content-Type", "application/json");
        let response = request
            .send_json(serde_json::json!({}))
            .map_err(transport_error)?;
        decode_with_allowed_codes(response, &[40903])
    }
}

#[derive(Serialize)]
struct PromptBody<'a> {
    content: &'a [PromptPart],
    #[serde(flatten)]
    options: &'a PromptOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PromptResult {
    pub prompt_id: String,
    pub user_message_id: String,
    #[serde(default)]
    pub status: Option<String>,
}
