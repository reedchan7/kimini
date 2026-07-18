use gpui::{AppContext, Context, Window};

use crate::api::ApiError;
use crate::native::slash::SlashCommand;

use super::super::super::app::{LoadState, Shell};
use super::super::goals::GoalSubmission;
use super::{SkillSubmission, SubmissionMode};

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
                    Ok(_) => {
                        this.attachments.clear_sent(&request_session_id);
                        this.drafts.remove(&request_session_id);
                        if this.is_active_session(&request_session_id) {
                            this.load_snapshot(request_session_id.clone(), cx);
                        }
                    }
                    Err(error) => {
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
}
