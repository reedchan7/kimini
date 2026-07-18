mod queue;
mod send;
mod skills;
mod slash;

use crate::api::KimiClient;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SubmissionMode {
    Send,
    Steer,
}

pub(super) struct SkillSubmission {
    pub client: KimiClient,
    pub session_id: String,
    pub name: String,
    pub args: Option<String>,
    pub submitted_text: String,
}
