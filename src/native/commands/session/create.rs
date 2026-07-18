use gpui::{AppContext, Context, PathPromptOptions};

use crate::native::app::{LoadState, Shell};

impl Shell {
    pub(in crate::native) fn choose_session_workspace(&mut self, cx: &mut Context<Self>) {
        if self.client.is_none() {
            return;
        }
        let selection = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(self.strings.native.choose_folder.into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = selection.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let cwd = path.to_string_lossy().into_owned();
            let _ = this.update(cx, |this, cx| this.create_session(cwd, cx));
        })
        .detach();
    }

    fn create_session(&mut self, cwd: String, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let model = self.preferred_model();
        self.state = LoadState::Working(self.strings.native.working.into());
        let task =
            cx.background_spawn(async move { client.create_session(&cwd, model.as_deref()) });
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
}
