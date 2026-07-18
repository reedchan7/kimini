use gpui::{AppContext, Context, Task, Window};

use crate::api::{ApiError, KimiClient};

use super::app::{LoadState, Shell};

impl Shell {
    pub(super) fn create_session(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("/"))
            .to_string_lossy()
            .into_owned();
        self.state = LoadState::Working("Creating session…".into());
        let task = cx.background_spawn(async move { client.create_session(&cwd) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| match result {
                Ok(session) => {
                    let id = session.id.clone();
                    this.model.add_session(session);
                    this.load_snapshot(id, cx);
                }
                Err(error) => this.fail(error, cx),
            });
        })
        .detach();
    }

    pub(super) fn resolve_approval(
        &mut self,
        approval_id: String,
        approved: bool,
        cx: &mut Context<Self>,
    ) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working("Resolving approval…".into());
        let task = cx.background_spawn(async move {
            client.resolve_approval(&session_id, &approval_id, approved)?;
            Ok::<_, ApiError>(session_id)
        });
        self.reload_after(task, cx);
    }

    pub(super) fn resolve_question(
        &mut self,
        question_id: String,
        item_id: String,
        option_id: String,
        cx: &mut Context<Self>,
    ) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working("Answering question…".into());
        let task = cx.background_spawn(async move {
            client.resolve_question(&session_id, &question_id, &item_id, &option_id)?;
            Ok::<_, ApiError>(session_id)
        });
        self.reload_after(task, cx);
    }

    pub(super) fn abort(&mut self, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working("Stopping…".into());
        let task = cx.background_spawn(async move {
            client.abort_session(&session_id)?;
            Ok::<_, ApiError>(session_id)
        });
        self.reload_after(task, cx);
    }

    pub(super) fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.composer.read(cx).value().trim().to_owned();
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        if text.is_empty() {
            return;
        }
        self.composer
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.state = LoadState::Working("Sending…".into());
        let request_session_id = session_id.clone();
        let task = cx.background_spawn(async move { client.submit_prompt(&session_id, &text) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session_id) {
                    return;
                }
                match result {
                    Ok(_) => this.load_snapshot(request_session_id.clone(), cx),
                    Err(error) => this.state = LoadState::Failed(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn active_request_context(&self) -> Option<(KimiClient, String)> {
        Some((
            self.client.clone()?,
            self.model.active_session()?.id.clone(),
        ))
    }

    fn reload_after(&mut self, task: Task<Result<String, ApiError>>, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| match result {
                Ok(session_id) if this.is_active_session(&session_id) => {
                    this.load_snapshot(session_id, cx)
                }
                Ok(_) => {}
                Err(error) => this.fail(error, cx),
            });
        })
        .detach();
    }

    fn is_active_session(&self, session_id: &str) -> bool {
        self.model
            .active_session()
            .is_some_and(|session| session.id == session_id)
    }

    fn fail(&mut self, error: String, cx: &mut Context<Self>) {
        self.state = LoadState::Failed(error);
        cx.notify();
    }
}
