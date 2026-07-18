use crate::protocol::{ActivateSkillRequest, ActivateSkillResult, SkillList};

use super::{ApiError, KimiClient, segment};

impl KimiClient {
    pub fn list_skills(&self, session_id: &str) -> Result<SkillList, ApiError> {
        self.get(&format!("/sessions/{}/skills", segment(session_id)))
    }

    pub fn activate_skill(
        &self,
        session_id: &str,
        skill_name: &str,
        args: Option<&str>,
    ) -> Result<ActivateSkillResult, ApiError> {
        self.post(
            &format!(
                "/sessions/{}/skills/{}:activate",
                segment(session_id),
                segment(skill_name)
            ),
            &ActivateSkillRequest { args },
        )
    }
}
