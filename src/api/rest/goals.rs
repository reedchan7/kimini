use crate::protocol::{GoalControl, GoalSnapshot, Session};

use super::{ApiError, KimiClient, segment};

impl KimiClient {
    pub fn session_goal(&self, session_id: &str) -> Result<Option<GoalSnapshot>, ApiError> {
        self.get_optional(&format!("/sessions/{}/goal", segment(session_id)))
    }

    pub fn set_goal_objective(
        &self,
        session_id: &str,
        objective: &str,
    ) -> Result<Session, ApiError> {
        self.update_session_config(
            session_id,
            serde_json::json!({ "goal_objective": objective }),
        )
    }

    pub fn control_goal(
        &self,
        session_id: &str,
        control: GoalControl,
    ) -> Result<Session, ApiError> {
        self.update_session_config(
            session_id,
            serde_json::json!({ "goal_control": control.as_str() }),
        )
    }
}
