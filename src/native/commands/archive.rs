use gpui::{AppContext, Context, PromptLevel, Window};

use super::super::app::{LoadState, Shell};
use crate::api::ApiError;

impl Shell {
    pub(in crate::native) fn confirm_archive(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.model.active_session().is_none() {
            return;
        }
        let answer = window.prompt(
            PromptLevel::Warning,
            self.strings.native.archive_question,
            Some(self.strings.native.archive_detail),
            &[self.strings.native.archive, self.strings.native.cancel],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if answer.await != Ok(0) {
                return;
            }
            let _ = this.update(cx, |this, cx| this.archive_active_session(cx));
        })
        .detach();
    }

    fn archive_active_session(&mut self, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let request_session_id = session_id.clone();
        let task = cx.background_spawn(async move {
            client.archive_session(&session_id)?;
            Ok::<_, ApiError>(session_id)
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| match result {
                Ok(_) => {
                    this.model.invalidate_archived_sessions();
                    if let Some(next) = this.model.remove_session(&request_session_id) {
                        this.load_snapshot(next, cx);
                    } else {
                        this.transcript.rebuild(&this.model);
                        this.state = LoadState::Ready;
                        cx.notify();
                    }
                }
                Err(error) => this.fail(error, cx),
            });
        })
        .detach();
    }

    pub(in crate::native) fn toggle_archived_sessions(&mut self, cx: &mut Context<Self>) {
        self.show_archived = !self.show_archived;
        if self.show_archived && !self.model.archived_sessions_loaded() && !self.archives_loading {
            self.fetch_archived_sessions(None, cx);
        } else {
            cx.notify();
        }
    }

    pub(in crate::native) fn load_more_archived_sessions(&mut self, cx: &mut Context<Self>) {
        if self.archives_loading || !self.model.has_more_archived_sessions() {
            return;
        }
        let before = self
            .model
            .archived_sessions()
            .last()
            .map(|session| session.id.clone());
        self.fetch_archived_sessions(before, cx);
    }

    fn fetch_archived_sessions(&mut self, before: Option<String>, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        self.archives_loading = true;
        cx.notify();
        let initial = before.is_none();
        let task = cx.background_spawn(async move {
            match before {
                Some(session_id) => client.list_archived_sessions_before(&session_id),
                None => client.list_archived_sessions(),
            }
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                this.archives_loading = false;
                match result {
                    Ok(page) if initial => this.model.replace_archived_session_page(page),
                    Ok(page) => {
                        this.model.append_archived_session_page(page);
                    }
                    Err(error) => this.state = LoadState::Failed(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn restore_archived_session(
        &mut self,
        session_id: String,
        cx: &mut Context<Self>,
    ) {
        let Some(client) = self.client.clone() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let request_session_id = session_id.clone();
        let task = cx.background_spawn(async move { client.restore_session(&session_id) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| match result {
                Ok(session) => {
                    let session_id = session.id.clone();
                    this.model.remove_archived_session(&request_session_id);
                    this.model.add_session(session);
                    this.show_archived = false;
                    this.load_snapshot(session_id, cx);
                }
                Err(error) => this.fail(error, cx),
            });
        })
        .detach();
    }
}
