use gpui::{AppContext, Context, PromptLevel, Window};

use crate::api::{ApiError, KimiClient};
use crate::native::slash::GoalSlashCommand;
use crate::protocol::{GoalControl, GoalSnapshot, PromptPart, Session};

use super::super::app::{LoadState, Shell};

pub(super) struct GoalSubmission {
    pub client: KimiClient,
    pub session_id: String,
    pub objective: String,
    pub parts: Vec<PromptPart>,
    pub restore_text: String,
}

struct StartedGoal {
    session: Session,
    goal: Option<GoalSnapshot>,
    prompt_error: Option<ApiError>,
}

impl Shell {
    pub(in crate::native) fn run_goal_slash(
        &mut self,
        command: GoalSlashCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match command {
            GoalSlashCommand::Toggle => self.toggle_goal_mode(cx),
            GoalSlashCommand::Control(control) => self.control_active_goal(control, cx),
            GoalSlashCommand::Create(objective) => {
                let Some((client, session_id)) = self.active_request_context() else {
                    return;
                };
                self.begin_goal_submission(
                    GoalSubmission {
                        client,
                        session_id,
                        parts: vec![PromptPart::text(&objective)],
                        restore_text: format!("/goal {objective}"),
                        objective,
                    },
                    window,
                    cx,
                );
            }
        }
    }

    pub(in crate::native) fn toggle_goal_mode(&mut self, cx: &mut Context<Self>) {
        let Some(session_id) = self
            .model
            .active_session()
            .map(|session| session.id.clone())
        else {
            return;
        };
        self.goals.toggle_armed(&session_id);
        cx.notify();
    }

    pub(super) fn begin_goal_submission(
        &mut self,
        request: GoalSubmission,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.composer
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.drafts.remove(&request.session_id);
        let requires_confirmation = self
            .model
            .active_runtime()
            .is_none_or(|runtime| runtime.permission == "manual");
        if !requires_confirmation {
            self.submit_goal(request, cx);
            return;
        }

        let answer = window.prompt(
            PromptLevel::Warning,
            self.strings.native.goal_start_question,
            Some(&request.objective),
            &[self.strings.native.start_goal, self.strings.native.cancel],
            cx,
        );
        cx.spawn(async move |this, cx| {
            let confirmed = answer.await == Ok(0);
            let _ = this.update(cx, |this, cx| {
                if confirmed {
                    this.submit_goal(request, cx);
                } else {
                    this.restore_composer_draft(&request.session_id, request.restore_text);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn submit_goal(&mut self, request: GoalSubmission, cx: &mut Context<Self>) {
        let Some(runtime) = self.active_prompt_runtime() else {
            self.restore_composer_draft(&request.session_id, request.restore_text);
            self.state = LoadState::Failed(self.strings.native.model_required.into());
            cx.notify();
            return;
        };
        let options = runtime.options(None);
        self.state = LoadState::Working(self.strings.native.starting_goal.into());
        let request_session = request.session_id.clone();
        let objective = request.objective.clone();
        let restore_text = request.restore_text;
        let task = cx.background_spawn(async move {
            let session = request
                .client
                .set_goal_objective(&request.session_id, &request.objective)?;
            let prompt_error = request
                .client
                .submit_prompt_with_options(&request.session_id, &request.parts, &options)
                .err();
            let goal = request
                .client
                .session_goal(&request.session_id)
                .ok()
                .flatten();
            Ok::<_, ApiError>(StartedGoal {
                session,
                goal,
                prompt_error,
            })
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(started) => {
                        this.model.add_session(started.session);
                        this.model.set_goal(&request_session, started.goal);
                        this.goals.disarm(&request_session);
                        if let Some(error) = started.prompt_error {
                            this.restore_composer_draft(&request_session, objective.clone());
                            this.state = LoadState::Failed(error.to_string());
                        } else {
                            this.attachments.clear_sent(&request_session);
                            this.drafts.remove(&request_session);
                            if this.is_active_session(&request_session) {
                                this.load_snapshot(request_session.clone(), cx);
                            }
                        }
                    }
                    Err(error) => {
                        this.restore_composer_draft(&request_session, restore_text.clone());
                        this.state = LoadState::Failed(error.to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn control_active_goal(
        &mut self,
        control: GoalControl,
        cx: &mut Context<Self>,
    ) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let request_session = session_id.clone();
        let task = cx.background_spawn(async move {
            let session = client.control_goal(&session_id, control)?;
            let goal = client.session_goal(&session_id).ok().flatten();
            Ok::<_, ApiError>((session, goal))
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session) {
                    return;
                }
                match result {
                    Ok((session, goal)) => {
                        this.model.add_session(session);
                        this.model.set_goal(&request_session, goal);
                        this.state = LoadState::Ready;
                    }
                    Err(error) => this.state = LoadState::Failed(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn confirm_cancel_goal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.model.active_goal().is_none() {
            return;
        }
        let answer = window.prompt(
            PromptLevel::Warning,
            self.strings.native.goal_cancel_question,
            Some(self.strings.native.goal_cancel_detail),
            &[
                self.strings.native.cancel_goal,
                self.strings.native.keep_goal,
            ],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if answer.await != Ok(0) {
                return;
            }
            let _ = this.update(cx, |this, cx| {
                this.control_active_goal(GoalControl::Cancel, cx)
            });
        })
        .detach();
    }
}
