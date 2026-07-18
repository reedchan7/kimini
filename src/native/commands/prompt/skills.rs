use gpui::{AppContext, Context, Window};

use super::super::super::app::{LoadState, Shell};
use super::SkillSubmission;

impl Shell {
    pub(super) fn submit_skill_activation(
        &mut self,
        request: SkillSubmission,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.composer
            .update(cx, |input, cx| input.set_value("", window, cx));
        let Some(runtime) = self.active_prompt_runtime() else {
            self.restore_composer_draft(&request.session_id, request.submitted_text);
            self.state = LoadState::Failed(self.strings.native.model_required.into());
            cx.notify();
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let request_session = request.session_id.clone();
        let submitted_text = request.submitted_text;
        let task = cx.background_spawn(async move {
            request
                .client
                .update_session_config(&request.session_id, runtime.profile_patch())?;
            request.client.activate_skill(
                &request.session_id,
                &request.name,
                request.args.as_deref(),
            )
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(result) if result.activated => {
                        this.drafts.remove(&request_session);
                        this.skills.activated = Some(result.skill_name);
                        if this.is_active_session(&request_session) {
                            this.load_snapshot(request_session.clone(), cx);
                        }
                    }
                    Ok(_) => {
                        this.restore_composer_draft(&request_session, submitted_text.clone());
                        this.state = LoadState::Failed(
                            this.strings.native.skill_activation_unacknowledged.into(),
                        );
                    }
                    Err(error) => {
                        this.restore_composer_draft(&request_session, submitted_text.clone());
                        this.state = LoadState::Failed(error.to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}
