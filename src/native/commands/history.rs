use gpui::{AppContext, Context};

use super::super::app::{LoadState, Shell};

impl Shell {
    pub(in crate::native) fn load_more_sessions(&mut self, cx: &mut Context<Self>) {
        if self.sessions_loading || !self.model.has_more_sessions() {
            return;
        }
        let Some(client) = self.client.clone() else {
            return;
        };
        let Some(session_id) = self
            .model
            .sessions()
            .last()
            .map(|session| session.id.clone())
        else {
            return;
        };
        self.sessions_loading = true;
        cx.notify();
        let task = cx.background_spawn(async move { client.list_sessions_before(&session_id) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                this.sessions_loading = false;
                match result {
                    Ok(page) => {
                        this.model.append_session_page(page);
                    }
                    Err(error) => this.state = LoadState::Failed(error),
                }
                if this.session_search_open && this.model.has_more_sessions() {
                    this.load_remaining_sessions_for_search(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn load_remaining_sessions_for_search(&mut self, cx: &mut Context<Self>) {
        if self.sessions_loading || !self.model.has_more_sessions() {
            return;
        }
        let Some(client) = self.client.clone() else {
            return;
        };
        let Some(session_id) = self
            .model
            .sessions()
            .last()
            .map(|session| session.id.clone())
        else {
            return;
        };
        self.sessions_loading = true;
        cx.notify();
        let task =
            cx.background_spawn(async move { client.list_session_pages_before(&session_id) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                this.sessions_loading = false;
                match result {
                    Ok(pages) => {
                        for page in pages {
                            this.model.append_session_page(page);
                        }
                    }
                    Err(error) => this.state = LoadState::Failed(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn load_older_messages(&mut self, cx: &mut Context<Self>) {
        if self.history_loading {
            return;
        }
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        let Some(message_id) = self
            .model
            .active_conversation()
            .and_then(|conversation| conversation.messages.first())
            .map(|message| message.id.clone())
        else {
            return;
        };
        self.history_loading = true;
        cx.notify();
        let request_session_id = session_id.clone();
        let task = cx
            .background_spawn(async move { client.list_messages_before(&session_id, &message_id) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session_id) {
                    return;
                }
                this.history_loading = false;
                match result {
                    Ok(page) => {
                        this.model.prepend_messages(&request_session_id, page);
                        this.transcript.rebuild(&this.model);
                    }
                    Err(error) => this.state = LoadState::Failed(error),
                }
                cx.notify();
            });
        })
        .detach();
    }
}
