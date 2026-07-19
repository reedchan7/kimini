use gpui::{AppContext, Context, PromptLevel, Window};

use crate::api::ApiError;
use crate::native::app::{LoadState, Shell};

impl Shell {
    pub(in crate::native) fn fork_active_session(&mut self, cx: &mut Context<Self>) {
        let Some(session_id) = self
            .model
            .active_session()
            .map(|session| session.id.clone())
        else {
            return;
        };
        self.fork_session(session_id, cx);
    }

    pub(in crate::native) fn fork_session(&mut self, session_id: String, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let task = cx.background_spawn(async move { client.fork_session(&session_id) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| match result {
                Ok(session) => {
                    let session_id = session.id.clone();
                    this.model.add_session(session);
                    this.load_snapshot(session_id, cx);
                }
                Err(error) => this.fail(error, cx),
            });
        })
        .detach();
    }

    pub(in crate::native) fn confirm_compact(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.model.active_session().is_none() {
            return;
        }
        let answer = window.prompt(
            PromptLevel::Info,
            self.strings.native.compact_question,
            Some(self.strings.native.compact_detail),
            &[self.strings.native.compact, self.strings.native.cancel],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if answer.await != Ok(0) {
                return;
            }
            let _ = this.update(cx, |this, cx| this.compact_active_session(None, cx));
        })
        .detach();
    }

    pub(in crate::native) fn compact_active_session(
        &mut self,
        instruction: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let task = cx.background_spawn(async move {
            client.compact_session_with_instruction(&session_id, instruction.as_deref())?;
            Ok::<_, ApiError>(session_id)
        });
        self.reload_after(task, cx);
    }

    pub(in crate::native) fn confirm_undo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.model.active_session().is_none() {
            return;
        }
        let answer = window.prompt(
            PromptLevel::Warning,
            self.strings.native.undo_question,
            Some(self.strings.native.undo_detail),
            &[self.strings.native.undo, self.strings.native.cancel],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if answer.await != Ok(0) {
                return;
            }
            let _ = this.update(cx, |this, cx| this.undo_active_session(cx));
        })
        .detach();
    }

    fn undo_active_session(&mut self, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let task = cx.background_spawn(async move {
            client.undo_session(&session_id)?;
            Ok::<_, ApiError>(session_id)
        });
        self.reload_after(task, cx);
    }
}
