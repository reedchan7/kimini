use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub root: String,
    pub name: String,
    pub created_at: String,
    pub last_opened_at: String,
    pub session_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceList {
    pub items: Vec<Workspace>,
}
