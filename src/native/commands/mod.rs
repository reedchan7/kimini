mod archive;
mod auth;
pub(in crate::native) mod config;
mod export;
mod files;
mod goals;
mod history;
mod interaction;
mod language;
mod prompt;
mod recovery;
mod runtime;
mod session;
mod side_chat;
mod skills;
mod tasks;
mod terminal;
mod workspace;

use gpui::{Context, Task};

use crate::api::{ApiError, KimiClient};

use super::app::{LoadState, Shell};

impl Shell {
    fn active_request_context(&self) -> Option<(KimiClient, String)> {
        if self.new_session_draft.is_some() {
            return None;
        }
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
        self.new_session_draft.is_none()
            && self
                .model
                .active_session()
                .is_some_and(|session| session.id == session_id)
    }

    fn fail(&mut self, error: String, cx: &mut Context<Self>) {
        self.state = LoadState::Failed(error);
        cx.notify();
    }
}
