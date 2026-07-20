use gpui::{AppContext, Context, Window};

use crate::api::{ApiError, PromptResult};
use crate::native::slash::SlashCommand;
use crate::protocol::{Session, Workspace};

use super::super::super::app::{LoadState, Shell};
use super::super::goals::GoalSubmission;
use super::{SkillSubmission, SubmissionMode};

struct StartedSession {
    session: Session,
    prompt_result: Result<PromptResult, ApiError>,
    workspaces: Option<Vec<Workspace>>,
}

impl StartedSession {
    fn from_prompt_result(session: Session, result: Result<PromptResult, ApiError>) -> Self {
        Self {
            session,
            prompt_result: result,
            workspaces: None,
        }
    }
}

impl Shell {
    pub(in crate::native) fn abort(&mut self, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let task = cx.background_spawn(async move {
            client.abort_session(&session_id)?;
            Ok::<_, ApiError>(session_id)
        });
        self.reload_after(task, cx);
    }

    pub(in crate::native) fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.submit_with_mode(SubmissionMode::Send, window, cx);
    }

    pub(in crate::native) fn steer_prompt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.submit_with_mode(SubmissionMode::Steer, window, cx);
    }

    fn submit_with_mode(
        &mut self,
        mode: SubmissionMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = self.composer.read(cx).value().trim().to_owned();
        if mode == SubmissionMode::Send && self.new_session_draft.is_some() {
            self.submit_new_session(text, window, cx);
            return;
        }
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        if self.attachments.has_uploads(&session_id) {
            return;
        }
        let parts = self.attachments.prompt_parts(&session_id, &text);
        if mode == SubmissionMode::Send
            && let Some(command) = SlashCommand::parse(&text)
        {
            if parts.len() > 1 {
                self.state =
                    LoadState::Failed(self.strings.native.command_attachments_unsupported.into());
                cx.notify();
                return;
            }
            self.composer
                .update(cx, |input, cx| input.set_value("", window, cx));
            self.drafts.remove(&session_id);
            self.run_slash_command(command, window, cx);
            return;
        }
        if mode == SubmissionMode::Send
            && let Some(activation) = self.skills.parse_activation(&text)
        {
            if parts.len() > 1 {
                self.state =
                    LoadState::Failed(self.strings.native.skill_attachments_unsupported.into());
                cx.notify();
                return;
            }
            self.submit_skill_activation(
                SkillSubmission {
                    client,
                    session_id,
                    name: activation.name,
                    args: activation.args,
                    submitted_text: text,
                },
                window,
                cx,
            );
            return;
        }
        if mode == SubmissionMode::Send && self.goals.is_armed(&session_id) && !text.is_empty() {
            self.begin_goal_submission(
                GoalSubmission {
                    client,
                    session_id,
                    objective: text.clone(),
                    parts,
                    restore_text: text,
                },
                window,
                cx,
            );
            return;
        }
        if parts.is_empty() {
            if mode == SubmissionMode::Steer {
                self.steer_queued_prompts(cx);
            }
            return;
        }
        let Some(runtime) = self.active_prompt_runtime() else {
            self.state = LoadState::Failed(self.strings.native.model_required.into());
            cx.notify();
            return;
        };
        let options = runtime.options(None);
        self.composer
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.state = LoadState::Working(
            if mode == SubmissionMode::Steer {
                self.strings.native.steering
            } else {
                self.strings.native.working
            }
            .into(),
        );
        if mode == SubmissionMode::Send {
            self.model.begin_prompt(&session_id, parts.clone());
            self.transcript.rebuild(&self.model);
        }
        cx.notify();
        let request_session_id = session_id.clone();
        let submitted_text = text;
        let task = cx.background_spawn(async move {
            let result = client.submit_prompt_with_options(&session_id, &parts, &options)?;
            if mode == SubmissionMode::Steer && result.status.as_deref() == Some("queued") {
                let prompt_ids = client
                    .list_prompts(&session_id)
                    .map(|queue| {
                        queue
                            .queued
                            .into_iter()
                            .map(|prompt| prompt.prompt_id)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_else(|_| vec![result.prompt_id.clone()]);
                if !prompt_ids.is_empty() {
                    let _ = client.steer_prompts(&session_id, &prompt_ids);
                }
            }
            Ok::<_, ApiError>(result)
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(result) => {
                        this.attachments.clear_sent(&request_session_id);
                        this.drafts.remove(&request_session_id);
                        if mode == SubmissionMode::Send {
                            this.model
                                .accept_prompt(&request_session_id, &result.user_message_id);
                            if this.is_active_session(&request_session_id) {
                                this.transcript.rebuild(&this.model);
                                // Backup path: daemon may not stream assistant.delta;
                                // poll until idle and reload the settled snapshot.
                                this.watch_turn_until_idle(request_session_id.clone(), cx);
                            }
                        } else if this.is_active_session(&request_session_id) {
                            this.load_snapshot(request_session_id.clone(), cx);
                        }
                    }
                    Err(error) => {
                        if mode == SubmissionMode::Send {
                            this.model.fail_prompt(&request_session_id);
                            if this.is_active_session(&request_session_id) {
                                this.transcript.rebuild(&this.model);
                            }
                        }
                        this.restore_composer_draft(&request_session_id, submitted_text.clone());
                        if this.is_active_session(&request_session_id) {
                            this.state = LoadState::Failed(error.to_string());
                        }
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn submit_new_session(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(draft) = self.new_session_draft.clone() else {
            return;
        };
        if draft.submitting {
            return;
        }
        let draft_key = draft.key();
        if let Some(command) = SlashCommand::parse(&text) {
            if !command.available_in_new_session() {
                self.state = LoadState::Failed(self.strings.native.command_requires_session.into());
                cx.notify();
                return;
            }
            if !self.attachments.for_session(&draft_key).is_empty() {
                self.state =
                    LoadState::Failed(self.strings.native.command_attachments_unsupported.into());
                cx.notify();
                return;
            }
            self.composer
                .update(cx, |input, cx| input.set_value("", window, cx));
            self.drafts.remove(&draft_key);
            self.run_slash_command(command, window, cx);
            return;
        }
        if self.attachments.has_uploads(&draft_key) {
            return;
        }
        let parts = self.attachments.prompt_parts(&draft_key, &text);
        if parts.is_empty() {
            return;
        }
        let Some(runtime) = self.active_prompt_runtime() else {
            self.state = LoadState::Failed(self.strings.native.model_required.into());
            cx.notify();
            return;
        };
        let Some(client) = self.client.clone() else {
            return;
        };
        let options = runtime.options(None);
        let model = (!draft.model.is_empty()).then_some(draft.model.clone());
        if let Some(active) = self.new_session_draft.as_mut()
            && active.key() == draft_key
        {
            active.submitting = true;
            active.submitted_parts = parts.clone();
        }
        self.composer
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.state = LoadState::Working(self.strings.native.working.into());
        let submitted_text = text;
        let submitted_parts = parts.clone();
        let cwd = draft.cwd;
        let request_key = draft_key.clone();
        let task = cx.background_spawn(async move {
            let session = client.create_session(&cwd, model.as_deref())?;
            let prompt_result = client.submit_prompt_with_options(&session.id, &parts, &options);
            let mut started = StartedSession::from_prompt_result(session, prompt_result);
            started.workspaces = client.list_workspaces().ok().map(|list| list.items);
            Ok::<_, ApiError>(started)
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(started) => {
                        let StartedSession {
                            session,
                            prompt_result,
                            workspaces,
                        } = started;
                        if let Some(workspaces) = workspaces {
                            this.model.replace_workspaces(workspaces);
                        }
                        let session_id = session.id.clone();
                        match prompt_result {
                            Err(error) => {
                                this.model.add_session(session);
                                this.attachments.move_session(&request_key, &session_id);
                                this.drafts.remove(&request_key);
                                this.drafts.set(&session_id, submitted_text.clone());
                                if this.active_composer_key().as_deref()
                                    == Some(request_key.as_str())
                                {
                                    this.new_session_draft = None;
                                    this.composer_session_id = None;
                                    this.load_snapshot_with_notice(
                                        session_id,
                                        Some(error.to_string()),
                                        cx,
                                    );
                                }
                            }
                            Ok(prompt_result) => {
                                this.attachments.discard_session(&request_key);
                                this.drafts.remove(&request_key);
                                if this.active_composer_key().as_deref()
                                    == Some(request_key.as_str())
                                {
                                    this.model.activate_submitted_session(
                                        session,
                                        submitted_parts.clone(),
                                        prompt_result.user_message_id,
                                    );
                                    this.transcript.rebuild(&this.model);
                                    this.new_session_draft = None;
                                    this.composer_session_id = None;
                                    this.load_snapshot(session_id.clone(), cx);
                                    this.watch_turn_until_idle(session_id, cx);
                                } else {
                                    this.model.add_session(session);
                                }
                            }
                        }
                    }
                    Err(error) => {
                        if this.active_composer_key().as_deref() == Some(request_key.as_str()) {
                            this.restore_new_session_draft(&request_key, submitted_text.clone());
                            if let Some(draft) = this.new_session_draft.as_mut() {
                                draft.submitting = false;
                                draft.submitted_parts.clear();
                            }
                        } else {
                            this.attachments.discard_session(&request_key);
                            this.drafts.remove(&request_key);
                        }
                        this.state = LoadState::Failed(error.to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Session {
        serde_json::from_value(serde_json::json!({
            "id": "session-1",
            "workspace_id": "workspace-1",
            "title": "",
            "created_at": "2026-07-19T00:00:00Z",
            "updated_at": "2026-07-19T00:00:00Z",
            "busy": false,
            "metadata": { "cwd": "/workspace/kimini" },
            "agent_config": { "model": "kimi-code/k3" },
            "usage": {
                "input_tokens": 0,
                "output_tokens": 0,
                "cache_read_tokens": 0,
                "cache_creation_tokens": 0,
                "total_cost_usd": 0.0,
                "context_tokens": 0,
                "context_limit": 0,
                "turn_count": 0
            },
            "message_count": 0,
            "last_seq": 0
        }))
        .unwrap()
    }

    #[test]
    fn a_failed_first_prompt_keeps_the_created_session() {
        let started = StartedSession::from_prompt_result(
            session(),
            Err(ApiError::Daemon {
                code: 500,
                message: "prompt failed".into(),
            }),
        );

        assert_eq!(started.session.id, "session-1");
        assert!(started.prompt_result.is_err());
    }
}
