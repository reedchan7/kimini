use std::collections::HashMap;

use crate::protocol::{Task, TaskKind, TaskStatus, WireEvent};

#[derive(Debug, Default)]
pub(super) struct TaskRosters {
    by_session: HashMap<String, Vec<Task>>,
}

impl TaskRosters {
    pub fn install(&mut self, session_id: String, snapshot: Vec<Task>, rest: Vec<Task>) {
        let mut tasks = snapshot;
        merge_tasks(&mut tasks, rest);
        sort_tasks(&mut tasks);
        self.by_session.insert(session_id, tasks);
    }

    pub fn replace_rest(&mut self, session_id: String, rest: Vec<Task>) {
        let mut tasks = self
            .by_session
            .remove(&session_id)
            .unwrap_or_default()
            .into_iter()
            .filter(|task| task.subagent_phase.is_some())
            .collect::<Vec<_>>();
        merge_tasks(&mut tasks, rest);
        sort_tasks(&mut tasks);
        self.by_session.insert(session_id, tasks);
    }

    pub fn for_session(&self, session_id: &str) -> &[Task] {
        self.by_session
            .get(session_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn has_running(&self, session_id: &str) -> bool {
        self.for_session(session_id).iter().any(Task::is_running)
    }

    pub fn apply_event(&mut self, event: &WireEvent) -> bool {
        let Some(session_id) = event.session_id.as_deref() else {
            return false;
        };
        let kind = event.kind.strip_prefix("event.").unwrap_or(&event.kind);
        match kind {
            "subagent.spawned" => {
                let Some(id) = string_field(&event.payload, "subagentId", "subagent_id") else {
                    return false;
                };
                let description = string_field(&event.payload, "description", "description")
                    .or_else(|| string_field(&event.payload, "subagentName", "subagent_name"))
                    .unwrap_or("Subagent")
                    .to_owned();
                let task = Task {
                    id: id.to_owned(),
                    session_id: session_id.to_owned(),
                    kind: TaskKind::Subagent,
                    description,
                    status: TaskStatus::Running,
                    command: None,
                    created_at: event.timestamp.clone(),
                    started_at: None,
                    completed_at: None,
                    output_preview: None,
                    output_bytes: None,
                    subagent_phase: Some("queued".into()),
                    subagent_type: string_field(&event.payload, "subagentName", "subagent_name")
                        .map(str::to_owned),
                    parent_tool_call_id: string_field(
                        &event.payload,
                        "parentToolCallId",
                        "parent_tool_call_id",
                    )
                    .map(str::to_owned),
                    suspended_reason: None,
                    swarm_index: event
                        .payload
                        .get("swarmIndex")
                        .or_else(|| event.payload.get("swarm_index"))
                        .and_then(serde_json::Value::as_u64),
                    run_in_background: event
                        .payload
                        .get("runInBackground")
                        .or_else(|| event.payload.get("run_in_background"))
                        .and_then(serde_json::Value::as_bool),
                };
                self.upsert(session_id, task);
                true
            }
            "subagent.started" => self.patch_subagent(session_id, event, |task| {
                task.subagent_phase = Some("working".into());
                task.started_at = Some(event.timestamp.clone());
            }),
            "subagent.suspended" => self.patch_subagent(session_id, event, |task| {
                task.subagent_phase = Some("suspended".into());
                task.suspended_reason =
                    string_field(&event.payload, "reason", "reason").map(str::to_owned);
            }),
            "subagent.completed" => self.patch_subagent(session_id, event, |task| {
                task.status = TaskStatus::Completed;
                task.subagent_phase = Some("completed".into());
                task.completed_at = Some(event.timestamp.clone());
                task.output_preview =
                    string_field(&event.payload, "resultSummary", "result_summary")
                        .map(str::to_owned);
            }),
            "subagent.failed" => self.patch_subagent(session_id, event, |task| {
                task.status = TaskStatus::Failed;
                task.subagent_phase = Some("failed".into());
                task.completed_at = Some(event.timestamp.clone());
                task.output_preview =
                    string_field(&event.payload, "error", "error").map(str::to_owned);
            }),
            _ => false,
        }
    }

    fn patch_subagent(
        &mut self,
        session_id: &str,
        event: &WireEvent,
        patch: impl FnOnce(&mut Task),
    ) -> bool {
        let Some(id) = string_field(&event.payload, "subagentId", "subagent_id") else {
            return false;
        };
        let Some(task) = self
            .by_session
            .get_mut(session_id)
            .and_then(|tasks| tasks.iter_mut().find(|task| task.id == id))
        else {
            return false;
        };
        patch(task);
        true
    }

    fn upsert(&mut self, session_id: &str, task: Task) {
        let tasks = self.by_session.entry(session_id.into()).or_default();
        if let Some(existing) = tasks.iter_mut().find(|existing| existing.id == task.id) {
            *existing = task;
        } else {
            tasks.push(task);
            sort_tasks(tasks);
        }
    }
}

fn merge_tasks(target: &mut Vec<Task>, incoming: Vec<Task>) {
    for task in incoming {
        if let Some(existing) = target.iter_mut().find(|existing| existing.id == task.id) {
            existing.merge_runtime_details(&task);
        } else {
            target.push(task);
        }
    }
}

fn sort_tasks(tasks: &mut [Task]) {
    tasks.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn string_field<'a>(payload: &'a serde_json::Value, camel: &str, snake: &str) -> Option<&'a str> {
    payload
        .get(camel)
        .or_else(|| payload.get(snake))
        .and_then(serde_json::Value::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: &str, kind: &str, status: &str) -> Task {
        serde_json::from_value(serde_json::json!({
            "id": id, "session_id": "session", "kind": kind,
            "description": id, "status": status,
            "created_at": "2026-07-18T08:00:00.000Z"
        }))
        .unwrap()
    }

    #[test]
    fn rest_refresh_keeps_snapshot_only_subagents_and_drops_stale_processes() {
        let mut rosters = TaskRosters::default();
        let mut subagent = task("agent", "subagent", "running");
        subagent.subagent_phase = Some("working".into());
        rosters.install(
            "session".into(),
            vec![subagent],
            vec![task("old", "bash", "completed")],
        );

        rosters.replace_rest("session".into(), vec![task("current", "bash", "running")]);

        let ids = rosters
            .for_session("session")
            .iter()
            .map(|task| task.id.as_str())
            .collect::<Vec<_>>();
        assert!(ids.contains(&"agent"));
        assert!(ids.contains(&"current"));
        assert!(!ids.contains(&"old"));
    }

    #[test]
    fn subagent_events_form_a_complete_live_lifecycle() {
        let mut rosters = TaskRosters::default();
        let spawned: WireEvent = serde_json::from_value(serde_json::json!({
            "type": "subagent.spawned", "session_id": "session", "timestamp": "t1",
            "payload": { "subagentId": "agent", "subagentName": "reviewer", "description": "Review" }
        }))
        .unwrap();
        let completed: WireEvent = serde_json::from_value(serde_json::json!({
            "type": "subagent.completed", "session_id": "session", "timestamp": "t2",
            "payload": { "subagentId": "agent", "resultSummary": "Looks good" }
        }))
        .unwrap();

        assert!(rosters.apply_event(&spawned));
        assert!(rosters.has_running("session"));
        assert!(rosters.apply_event(&completed));
        let task = &rosters.for_session("session")[0];
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.output_preview.as_deref(), Some("Looks good"));
    }
}
