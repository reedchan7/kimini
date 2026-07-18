use gpui::{AppContext, Context};

use crate::api::ApiError;

use super::super::app::{LoadState, Shell};

impl Shell {
    pub(in crate::native) fn resolve_approval(
        &mut self,
        approval_id: String,
        approved: bool,
        for_session: bool,
        cx: &mut Context<Self>,
    ) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let task = cx.background_spawn(async move {
            if for_session {
                client.resolve_approval_for_session(&session_id, &approval_id)?;
            } else {
                client.resolve_approval(&session_id, &approval_id, approved)?;
            }
            Ok::<_, ApiError>(session_id)
        });
        self.reload_after(task, cx);
    }

    pub(in crate::native) fn submit_question(
        &mut self,
        question_id: String,
        cx: &mut Context<Self>,
    ) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        let Some(request) = self.model.active_conversation().and_then(|conversation| {
            conversation
                .questions
                .iter()
                .find(|request| request.question_id == question_id)
        }) else {
            return;
        };
        let Some(answers) = self.question_drafts.answers(request) else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let request_question_id = question_id.clone();
        let task = cx.background_spawn(async move {
            client.resolve_question_answers(&session_id, &question_id, &answers)?;
            Ok::<_, ApiError>(session_id)
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| match result {
                Ok(session_id) if this.is_active_session(&session_id) => {
                    this.question_drafts.remove(&request_question_id);
                    this.load_snapshot(session_id, cx);
                }
                Ok(_) => {}
                Err(error) => this.fail(error, cx),
            });
        })
        .detach();
    }

    pub(in crate::native) fn dismiss_question(
        &mut self,
        question_id: String,
        cx: &mut Context<Self>,
    ) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let request_question_id = question_id.clone();
        let task = cx.background_spawn(async move {
            client.dismiss_question(&session_id, &question_id)?;
            Ok::<_, ApiError>(session_id)
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| match result {
                Ok(session_id) if this.is_active_session(&session_id) => {
                    this.question_drafts.remove(&request_question_id);
                    this.load_snapshot(session_id, cx);
                }
                Ok(_) => {}
                Err(error) => this.fail(error, cx),
            });
        })
        .detach();
    }
}
