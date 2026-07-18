use crate::protocol::{QuestionAnswer, QuestionAnswers};

use super::{ApiError, KimiClient, decode_with_allowed_codes, segment, transport_error};

impl KimiClient {
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

    pub fn resolve_approval_for_session(
        &self,
        session_id: &str,
        approval_id: &str,
    ) -> Result<serde_json::Value, ApiError> {
        self.post(
            &format!(
                "/sessions/{}/approvals/{}",
                segment(session_id),
                segment(approval_id)
            ),
            &serde_json::json!({ "decision": "approved", "scope": "session" }),
        )
    }

    pub fn resolve_question(
        &self,
        session_id: &str,
        question_id: &str,
        item_id: &str,
        option_id: &str,
    ) -> Result<serde_json::Value, ApiError> {
        let answers = QuestionAnswers::from([(
            item_id.into(),
            QuestionAnswer::Single {
                option_id: option_id.into(),
            },
        )]);
        self.resolve_question_answers(session_id, question_id, &answers)
    }

    pub fn resolve_question_answers(
        &self,
        session_id: &str,
        question_id: &str,
        answers: &QuestionAnswers,
    ) -> Result<serde_json::Value, ApiError> {
        self.post(
            &format!(
                "/sessions/{}/questions/{}",
                segment(session_id),
                segment(question_id)
            ),
            &serde_json::json!({ "answers": answers, "method": "click" }),
        )
    }

    pub fn dismiss_question(&self, session_id: &str, question_id: &str) -> Result<(), ApiError> {
        let request = self
            .authorize(self.agent.post(&self.url(&format!(
                "/sessions/{}/questions/{}:dismiss",
                segment(session_id),
                segment(question_id)
            ))))
            .set("Content-Type", "application/json");
        let response = request
            .send_json(serde_json::json!({}))
            .map_err(transport_error)?;
        let _: serde_json::Value = decode_with_allowed_codes(response, &[40909])?;
        Ok(())
    }
}
