use std::time::Duration;

use gpui::{AppContext, Context};

use crate::api::ApiError;
use crate::protocol::{Task, TaskList};

use super::super::app::{Shell, UtilityPanel};

const RUNNING_OUTPUT_BYTES: usize = 4 * 1024;
const FINISHED_OUTPUT_BYTES: usize = 32 * 1024;

impl Shell {
    pub(in crate::native) fn toggle_task_panel(&mut self, cx: &mut Context<Self>) {
        self.utility_panel = if self.utility_panel == Some(UtilityPanel::Tasks) {
            None
        } else {
            Some(UtilityPanel::Tasks)
        };
        self.task_poll_generation = self.task_poll_generation.wrapping_add(1);
        self.task_poll_scheduled = false;
        if self.utility_panel == Some(UtilityPanel::Tasks) {
            self.refresh_tasks(cx);
        }
        cx.notify();
    }

    pub(in crate::native) fn refresh_tasks(&mut self, cx: &mut Context<Self>) {
        if self.tasks_loading {
            return;
        }
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.tasks_loading = true;
        self.task_error = None;
        self.task_request_generation = self.task_request_generation.wrapping_add(1);
        let generation = self.task_request_generation;
        let request_session_id = session_id.clone();
        let task = cx.background_spawn(async move {
            let mut list = list_tasks_with_transport_retry(&client, &session_id)?;
            hydrate_outputs(&client, &session_id, &mut list);
            Ok::<_, ApiError>(list)
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if generation != this.task_request_generation
                    || !this.is_active_session(&request_session_id)
                {
                    return;
                }
                this.tasks_loading = false;
                match result {
                    Ok(tasks) => this.tasks.replace_rest(request_session_id, tasks.items),
                    Err(error) => this.task_error = Some(error),
                }
                this.schedule_task_poll(cx);
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn cancel_background_task(
        &mut self,
        task_id: String,
        cx: &mut Context<Self>,
    ) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.task_error = None;
        let request_session_id = session_id.clone();
        let task = cx.background_spawn(async move {
            client.cancel_task(&session_id, &task_id)?;
            Ok::<_, ApiError>(())
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session_id) {
                    return;
                }
                match result {
                    Ok(()) => this.refresh_tasks(cx),
                    Err(error) => {
                        this.task_error = Some(error);
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

    pub(in crate::native) fn schedule_task_poll(&mut self, cx: &mut Context<Self>) {
        let Some(session_id) = self
            .model
            .active_session()
            .map(|session| session.id.clone())
        else {
            return;
        };
        if self.utility_panel != Some(UtilityPanel::Tasks)
            || !self.tasks.has_running(&session_id)
            || self.task_poll_scheduled
        {
            return;
        }
        self.task_poll_scheduled = true;
        let generation = self.task_poll_generation;
        let timer = cx.background_executor().timer(Duration::from_secs(1));
        cx.spawn(async move |this, cx| {
            timer.await;
            let _ = this.update(cx, |this, cx| {
                if generation != this.task_poll_generation {
                    return;
                }
                this.task_poll_scheduled = false;
                this.refresh_tasks(cx);
            });
        })
        .detach();
    }
}

fn list_tasks_with_transport_retry(
    client: &crate::api::KimiClient,
    session_id: &str,
) -> Result<TaskList, ApiError> {
    match client.list_tasks(session_id) {
        Err(error) if retryable_task_error(&error) => client.list_tasks(session_id),
        result => result,
    }
}

fn retryable_task_error(error: &ApiError) -> bool {
    matches!(error, ApiError::Transport(_))
}

fn hydrate_outputs(client: &crate::api::KimiClient, session_id: &str, tasks: &mut TaskList) {
    for task in &mut tasks.items {
        let limit = output_limit(task);
        if let Ok(detail) = client.task_with_output(session_id, &task.id, limit) {
            task.merge_runtime_details(&detail);
        }
    }
}

fn output_limit(task: &Task) -> usize {
    if task.is_running() {
        RUNNING_OUTPUT_BYTES
    } else {
        FINISHED_OUTPUT_BYTES
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_limits_distinguish_live_and_finished_tasks() {
        let task = |status| {
            serde_json::from_value::<Task>(serde_json::json!({
                "id": "task", "session_id": "session", "kind": "bash",
                "description": "run", "status": status,
                "created_at": "2026-07-18T08:00:00.000Z"
            }))
            .unwrap()
        };

        assert_eq!(output_limit(&task("running")), 4 * 1024);
        assert_eq!(output_limit(&task("completed")), 32 * 1024);
    }

    #[test]
    fn only_transport_failures_are_eligible_for_the_single_retry() {
        assert!(retryable_task_error(&ApiError::Transport("reset".into())));
        assert!(!retryable_task_error(&ApiError::Daemon {
            code: 404,
            message: "missing".into(),
        }));
    }
}
