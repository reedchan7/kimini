use crate::protocol::{CreateTerminal, Terminal, TerminalList};

use super::{ApiError, KimiClient, segment};

impl KimiClient {
    pub fn list_terminals(&self, session_id: &str) -> Result<TerminalList, ApiError> {
        self.get(&format!("/sessions/{}/terminals", segment(session_id)))
    }

    pub fn create_terminal(
        &self,
        session_id: &str,
        size: &CreateTerminal,
    ) -> Result<Terminal, ApiError> {
        self.post(
            &format!("/sessions/{}/terminals", segment(session_id)),
            size,
        )
    }

    pub fn close_terminal(
        &self,
        session_id: &str,
        terminal_id: &str,
    ) -> Result<serde_json::Value, ApiError> {
        self.post(
            &format!(
                "/sessions/{}/terminals/{}:close",
                segment(session_id),
                segment(terminal_id)
            ),
            &serde_json::json!({}),
        )
    }
}
