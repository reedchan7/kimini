use gpui::{AppContext, Context, Window};

use crate::native::app::{LoadState, Shell};

impl Shell {
    pub(in crate::native) fn begin_session_rename(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(session) = self.model.active_session() else {
            return;
        };
        let title = session.title.clone();
        self.renaming_session = true;
        self.rename_editor.update(cx, |input, cx| {
            input.set_value(title, window, cx);
            input.focus(window, cx);
        });
        cx.notify();
    }

    pub(in crate::native) fn cancel_session_rename(&mut self, cx: &mut Context<Self>) {
        self.renaming_session = false;
        cx.notify();
    }

    pub(in crate::native) fn commit_session_rename(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.renaming_session {
            return;
        }
        let title = self.rename_editor.read(cx).value().trim().to_owned();
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        if title.is_empty() {
            self.rename_editor
                .update(cx, |input, cx| input.focus(window, cx));
            return;
        }
        self.renaming_session = false;
        self.state = LoadState::Working(self.strings.native.working.into());
        let task = cx.background_spawn(async move { client.rename_session(&session_id, &title) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| match result {
                Ok(session) => {
                    this.model.add_session(session);
                    this.state = LoadState::Ready;
                    cx.notify();
                }
                Err(error) => this.fail(error, cx),
            });
        })
        .detach();
    }
}
