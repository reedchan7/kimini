use std::sync::atomic::AtomicBool;

use crate::api::KimiClient;
use crate::daemon::{Connection, discover_connection};
use crate::protocol::{
    AuthSummary, GoalSnapshot, ModelCatalogItem, Page, PromptQueue, Session, SessionSnapshot,
    SessionStatus, SkillList, TaskList,
};

pub(super) struct Bootstrap {
    pub connection: Connection,
    pub sessions: Page<Session>,
    pub active: Option<LoadedSession>,
    pub models: Vec<ModelCatalogItem>,
    pub auth: Option<AuthSummary>,
}

pub(super) struct LoadedSession {
    pub snapshot: SessionSnapshot,
    pub status: Option<SessionStatus>,
    pub prompts: Option<PromptQueue>,
    pub tasks: Option<TaskList>,
    pub skills: Option<SkillList>,
    pub goal: Option<GoalSnapshot>,
}

pub(super) fn load() -> Result<Bootstrap, String> {
    let stop = AtomicBool::new(false);
    let connection = discover_connection(&stop, &|_| {})
        .ok_or_else(|| "Kimi daemon discovery was cancelled".to_owned())?;
    let client = KimiClient::new(connection.clone());
    let sessions = client.list_sessions().map_err(|error| error.to_string())?;
    let models = client
        .list_models()
        .map(|catalog| catalog.items)
        .unwrap_or_default();
    let auth = client.auth_summary().ok();
    let active = sessions
        .items
        .iter()
        .find(|session| !session.archived)
        .map(|session| load_session(&client, &session.id))
        .transpose()
        .map_err(|error| error.to_string())?;
    Ok(Bootstrap {
        connection,
        sessions,
        active,
        models,
        auth,
    })
}

pub(super) fn load_session(
    client: &KimiClient,
    session_id: &str,
) -> Result<LoadedSession, crate::api::ApiError> {
    let snapshot = client.snapshot(session_id)?;
    Ok(LoadedSession {
        status: client.session_status(session_id).ok(),
        prompts: client.list_prompts(session_id).ok(),
        tasks: client.list_tasks(session_id).ok(),
        skills: client.list_skills(session_id).ok(),
        goal: client.session_goal(session_id).ok().flatten(),
        snapshot,
    })
}
