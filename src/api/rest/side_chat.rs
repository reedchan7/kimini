use crate::protocol::SideChatStart;

use super::{ApiError, KimiClient, segment};

impl KimiClient {
    pub fn start_side_chat(&self, session_id: &str) -> Result<SideChatStart, ApiError> {
        self.post(
            &format!("/sessions/{}:btw", segment(session_id)),
            &serde_json::json!({}),
        )
    }
}
