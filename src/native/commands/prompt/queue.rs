use gpui::{AppContext, Context};

use crate::api::ApiError;

use super::super::super::app::{LoadState, Shell};

impl Shell {
    pub(in crate::native) fn steer_queued_prompts(&mut self, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        let prompt_ids = self.prompt_queues.queued_ids(&session_id);
        if prompt_ids.is_empty() {
            return;
        }
        self.state = LoadState::Working(self.strings.native.steering.into());
        let task = cx.background_spawn(async move {
            client.steer_prompts(&session_id, &prompt_ids)?;
            Ok::<_, ApiError>(session_id)
        });
        self.reload_after(task, cx);
    }

    pub(in crate::native) fn remove_queued_prompt(
        &mut self,
        prompt_id: String,
        cx: &mut Context<Self>,
    ) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let task = cx.background_spawn(async move {
            client.abort_prompt(&session_id, &prompt_id)?;
            Ok::<_, ApiError>(session_id)
        });
        self.reload_after(task, cx);
    }
}
