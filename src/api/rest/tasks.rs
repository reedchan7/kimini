use crate::protocol::{Task, TaskList};

use super::{ApiError, KimiClient, segment};

impl KimiClient {
    pub fn list_tasks(&self, session_id: &str) -> Result<TaskList, ApiError> {
        self.get(&format!("/sessions/{}/tasks", segment(session_id)))
    }

    pub fn task_with_output(
        &self,
        session_id: &str,
        task_id: &str,
        output_bytes: usize,
    ) -> Result<Task, ApiError> {
        self.get(&format!(
            "/sessions/{}/tasks/{}?with_output=true&output_bytes={output_bytes}",
            segment(session_id),
            segment(task_id)
        ))
    }

    pub fn cancel_task(
        &self,
        session_id: &str,
        task_id: &str,
    ) -> Result<serde_json::Value, ApiError> {
        self.post(
            &format!(
                "/sessions/{}/tasks/{}:cancel",
                segment(session_id),
                segment(task_id)
            ),
            &serde_json::json!({}),
        )
    }
}
