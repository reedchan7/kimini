use crate::protocol::{FsDiff, FsGitStatusSummary, FsList, FsPreview, FsSearchResults};

use super::{ApiError, KimiClient, segment};

impl KimiClient {
    pub fn list_files(&self, session_id: &str, path: &str) -> Result<FsList, ApiError> {
        self.post(
            &format!("/sessions/{}/fs:list", segment(session_id)),
            &serde_json::json!({
                "path": path,
                "depth": 1,
                "limit": 500,
                "show_hidden": false,
                "follow_gitignore": true,
                "sort": "type_first",
                "include_git_status": true
            }),
        )
    }

    pub fn read_workspace_file(&self, session_id: &str, path: &str) -> Result<FsPreview, ApiError> {
        self.post(
            &format!("/sessions/{}/fs:read", segment(session_id)),
            &serde_json::json!({
                "path": path,
                "offset": 0,
                "length": 1_048_576,
                "encoding": "auto"
            }),
        )
    }

    pub fn workspace_file_diff(&self, session_id: &str, path: &str) -> Result<FsDiff, ApiError> {
        self.post(
            &format!("/sessions/{}/fs:diff", segment(session_id)),
            &serde_json::json!({ "path": path }),
        )
    }

    pub fn workspace_git_status(&self, session_id: &str) -> Result<FsGitStatusSummary, ApiError> {
        self.post(
            &format!("/sessions/{}/fs:git_status", segment(session_id)),
            &serde_json::json!({}),
        )
    }

    pub fn search_workspace_files(
        &self,
        session_id: &str,
        query: &str,
    ) -> Result<FsSearchResults, ApiError> {
        self.post(
            &format!("/sessions/{}/fs:search", segment(session_id)),
            &serde_json::json!({
                "query": query,
                "limit": 100,
                "follow_gitignore": true
            }),
        )
    }
}
