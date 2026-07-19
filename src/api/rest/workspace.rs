use crate::protocol::{Workspace, WorkspaceList};

use super::{ApiError, KimiClient, segment};

impl KimiClient {
    pub fn list_workspaces(&self) -> Result<WorkspaceList, ApiError> {
        self.get("/workspaces")
    }

    pub fn register_workspace(&self, root: &str) -> Result<Workspace, ApiError> {
        self.post("/workspaces", &serde_json::json!({ "root": root }))
    }

    pub fn rename_workspace(&self, workspace_id: &str, name: &str) -> Result<Workspace, ApiError> {
        self.patch(
            &format!("/workspaces/{}", segment(workspace_id)),
            &serde_json::json!({ "name": name }),
        )
    }

    pub fn remove_workspace(&self, workspace_id: &str) -> Result<serde_json::Value, ApiError> {
        self.delete(&format!("/workspaces/{}", segment(workspace_id)))
    }
}
