use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskKind {
    Subagent,
    Bash,
    Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub session_id: String,
    pub kind: TaskKind,
    pub description: String,
    pub status: TaskStatus,
    #[serde(default)]
    pub command: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub output_preview: Option<String>,
    #[serde(default)]
    pub output_bytes: Option<u64>,
    #[serde(default)]
    pub subagent_phase: Option<String>,
    #[serde(default)]
    pub subagent_type: Option<String>,
    #[serde(default)]
    pub parent_tool_call_id: Option<String>,
    #[serde(default)]
    pub suspended_reason: Option<String>,
    #[serde(default)]
    pub swarm_index: Option<u64>,
    #[serde(default)]
    pub run_in_background: Option<bool>,
}

impl Task {
    pub fn is_running(&self) -> bool {
        self.status == TaskStatus::Running
    }

    pub fn merge_runtime_details(&mut self, newer: &Self) {
        self.status = newer.status;
        self.command.clone_from(&newer.command);
        self.started_at.clone_from(&newer.started_at);
        self.completed_at.clone_from(&newer.completed_at);
        if newer.output_preview.is_some() {
            self.output_preview.clone_from(&newer.output_preview);
            self.output_bytes = newer.output_bytes;
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TaskList {
    #[serde(default)]
    pub items: Vec<Task>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_details_preserve_snapshot_only_subagent_identity() {
        let mut snapshot: Task = serde_json::from_value(serde_json::json!({
            "id": "agent_1", "session_id": "session", "kind": "subagent",
            "description": "review", "status": "running",
            "created_at": "2026-07-18T08:00:00.000Z", "subagent_type": "reviewer"
        }))
        .unwrap();
        let detail: Task = serde_json::from_value(serde_json::json!({
            "id": "agent_1", "session_id": "session", "kind": "subagent",
            "description": "review", "status": "completed",
            "created_at": "2026-07-18T08:00:00.000Z", "output_preview": "done"
        }))
        .unwrap();

        snapshot.merge_runtime_details(&detail);

        assert_eq!(snapshot.subagent_type.as_deref(), Some("reviewer"));
        assert_eq!(snapshot.output_preview.as_deref(), Some("done"));
        assert_eq!(snapshot.status, TaskStatus::Completed);
    }
}
