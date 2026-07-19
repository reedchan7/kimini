use gpui::{AppContext, Context, PromptLevel, Window};

use crate::native::app::{LoadState, Shell};

impl Shell {
    pub(in crate::native) fn begin_workspace_rename(
        &mut self,
        workspace_id: String,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.renaming_workspace_id = Some(workspace_id);
        self.workspace_rename_editor.update(cx, |input, cx| {
            input.set_value(name, window, cx);
        });
        cx.defer_in(window, |this, window, cx| {
            if this.renaming_workspace_id.is_some() {
                this.workspace_rename_editor
                    .update(cx, |input, cx| input.focus(window, cx));
            }
        });
        cx.notify();
    }

    pub(in crate::native) fn cancel_workspace_rename(&mut self, cx: &mut Context<Self>) {
        self.renaming_workspace_id = None;
        cx.notify();
    }

    pub(in crate::native) fn commit_workspace_rename(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(workspace_id) = self.renaming_workspace_id.clone() else {
            return;
        };
        let name = self
            .workspace_rename_editor
            .read(cx)
            .value()
            .trim()
            .to_owned();
        if name.is_empty() {
            self.workspace_rename_editor
                .update(cx, |input, cx| input.focus(window, cx));
            return;
        }
        let Some(client) = self.client.clone() else {
            return;
        };
        self.renaming_workspace_id = None;
        self.state = LoadState::Working(self.strings.native.working.into());
        let task =
            cx.background_spawn(async move { client.rename_workspace(&workspace_id, &name) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| match result {
                Ok(workspace) => {
                    this.model.upsert_workspace(workspace);
                    this.state = LoadState::Ready;
                    cx.notify();
                }
                Err(error) => this.fail(error, cx),
            });
        })
        .detach();
    }

    pub(in crate::native) fn confirm_remove_workspace(
        &mut self,
        workspace_id: String,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let question = self
            .strings
            .native
            .remove_workspace_question
            .replace("{name}", &name);
        let answer = window.prompt(
            PromptLevel::Warning,
            &question,
            Some(self.strings.native.remove_workspace_detail),
            &[
                self.strings.native.remove_workspace,
                self.strings.native.cancel,
            ],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if answer.await != Ok(0) {
                return;
            }
            let _ = this.update(cx, |this, cx| {
                this.remove_workspace(workspace_id.clone(), cx)
            });
        })
        .detach();
    }

    fn remove_workspace(&mut self, workspace_id: String, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let request_id = workspace_id.clone();
        let task = cx.background_spawn(async move { client.remove_workspace(&workspace_id) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| match result {
                Ok(_) => {
                    if this.model.remove_workspace(&request_id) {
                        this.socket_generation = this.socket_generation.wrapping_add(1);
                        this.socket = None;
                        this.renaming_session_id = None;
                        this.transcript.rebuild(&this.model);
                    }
                    this.state = LoadState::Ready;
                    cx.notify();
                }
                Err(error) => this.fail(error, cx),
            });
        })
        .detach();
    }
}
