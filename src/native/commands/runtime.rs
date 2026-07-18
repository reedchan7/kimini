use gpui::{AppContext, Context};

use crate::api::ApiError;

use super::super::app::{LoadState, Shell};

impl Shell {
    pub(in crate::native) fn set_model(&mut self, model: String, cx: &mut Context<Self>) {
        self.update_runtime(serde_json::json!({ "model": model }), cx);
    }

    pub(in crate::native) fn set_thinking(&mut self, effort: String, cx: &mut Context<Self>) {
        self.update_runtime(serde_json::json!({ "thinking": effort }), cx);
    }

    pub(in crate::native) fn set_permission(&mut self, mode: String, cx: &mut Context<Self>) {
        self.update_runtime(serde_json::json!({ "permission_mode": mode }), cx);
    }

    pub(in crate::native) fn toggle_plan_mode(&mut self, cx: &mut Context<Self>) {
        let enabled = !self
            .model
            .active_runtime()
            .is_some_and(|runtime| runtime.plan_mode);
        self.update_runtime(serde_json::json!({ "plan_mode": enabled }), cx);
    }

    pub(in crate::native) fn toggle_swarm_mode(&mut self, cx: &mut Context<Self>) {
        let enabled = !self
            .model
            .active_runtime()
            .is_some_and(|runtime| runtime.swarm_mode);
        self.update_runtime(serde_json::json!({ "swarm_mode": enabled }), cx);
    }

    pub(in crate::native) fn set_swarm_mode(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.update_runtime(serde_json::json!({ "swarm_mode": enabled }), cx);
    }

    pub(in crate::native) fn cycle_thinking(&mut self, cx: &mut Context<Self>) {
        let current = self
            .model
            .active_runtime()
            .map(|runtime| runtime.thinking_level.as_str())
            .unwrap_or("off");
        let mut efforts = self
            .model
            .active_runtime()
            .and_then(|runtime| runtime.model.as_deref())
            .and_then(|model| self.models.iter().find(|item| item.model == model))
            .map(|model| model.support_efforts.clone())
            .unwrap_or_else(|| vec!["off".into(), "on".into()]);
        if !efforts.iter().any(|effort| effort == "off") {
            efforts.insert(0, "off".into());
        }
        let next = efforts
            .iter()
            .position(|effort| effort == current)
            .map(|index| (index + 1) % efforts.len())
            .unwrap_or(0);
        self.set_thinking(efforts[next].clone(), cx);
    }

    fn update_runtime(&mut self, config: serde_json::Value, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.state = LoadState::Working(self.strings.native.working.into());
        let request_session_id = session_id.clone();
        let task = cx.background_spawn(async move {
            let session = client.update_session_config(&session_id, config)?;
            let runtime = client.session_status(&session_id)?;
            Ok::<_, ApiError>((session, runtime))
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session_id) {
                    return;
                }
                match result {
                    Ok((session, runtime)) => {
                        this.model.add_session(session);
                        this.model.set_runtime(&request_session_id, runtime);
                        this.state = LoadState::Ready;
                    }
                    Err(error) => this.state = LoadState::Failed(error),
                }
                cx.notify();
            });
        })
        .detach();
    }
}
