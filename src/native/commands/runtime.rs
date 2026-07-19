use gpui::{AppContext, Context};

use crate::api::ApiError;

use super::super::app::{DefaultPermission, LoadState, Shell};
use super::super::prompt_runtime::thinking_segments;

impl Shell {
    pub(in crate::native) fn set_model(&mut self, model: String, cx: &mut Context<Self>) {
        if let Some(draft) = self.new_session_draft.as_mut() {
            draft.model = model.clone();
            if let Some(catalog) = self.models.iter().find(|item| item.model == model) {
                let segments = thinking_segments(Some(catalog));
                if !segments.iter().any(|segment| segment == &draft.thinking) {
                    draft.thinking = catalog
                        .default_effort
                        .clone()
                        .filter(|effort| segments.contains(effort))
                        .or_else(|| segments.first().cloned())
                        .unwrap_or_else(|| "off".into());
                }
            }
            cx.notify();
            return;
        }
        self.update_runtime(serde_json::json!({ "model": model }), cx);
    }

    pub(in crate::native) fn set_thinking(&mut self, effort: String, cx: &mut Context<Self>) {
        if let Some(draft) = self.new_session_draft.as_mut() {
            draft.thinking = effort;
            cx.notify();
            return;
        }
        self.update_runtime(serde_json::json!({ "thinking": effort }), cx);
    }

    pub(in crate::native) fn set_permission(&mut self, mode: String, cx: &mut Context<Self>) {
        if let Some(permission) = DefaultPermission::from_mode(&mode) {
            self.update_preferences(
                |preferences| preferences.composer_permission = permission,
                cx,
            );
        }
        if let Some(draft) = self.new_session_draft.as_mut() {
            draft.permission = mode;
            cx.notify();
            return;
        }
        self.update_runtime(serde_json::json!({ "permission_mode": mode }), cx);
    }

    pub(in crate::native) fn toggle_plan_mode(&mut self, cx: &mut Context<Self>) {
        if let Some(draft) = self.new_session_draft.as_mut() {
            draft.plan_mode = !draft.plan_mode;
            cx.notify();
            return;
        }
        let enabled = !self
            .model
            .active_runtime()
            .is_some_and(|runtime| runtime.plan_mode);
        self.update_runtime(serde_json::json!({ "plan_mode": enabled }), cx);
    }

    pub(in crate::native) fn toggle_swarm_mode(&mut self, cx: &mut Context<Self>) {
        if let Some(draft) = self.new_session_draft.as_mut() {
            draft.swarm_mode = !draft.swarm_mode;
            cx.notify();
            return;
        }
        let enabled = !self
            .model
            .active_runtime()
            .is_some_and(|runtime| runtime.swarm_mode);
        self.update_runtime(serde_json::json!({ "swarm_mode": enabled }), cx);
    }

    pub(in crate::native) fn set_swarm_mode(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if let Some(draft) = self.new_session_draft.as_mut() {
            draft.swarm_mode = enabled;
            cx.notify();
            return;
        }
        self.update_runtime(serde_json::json!({ "swarm_mode": enabled }), cx);
    }

    pub(in crate::native) fn cycle_thinking(&mut self, cx: &mut Context<Self>) {
        let current = self
            .new_session_draft
            .as_ref()
            .map(|draft| draft.thinking.as_str())
            .or_else(|| {
                self.model
                    .active_runtime()
                    .map(|runtime| runtime.thinking_level.as_str())
            })
            .unwrap_or("off");
        let model_id = self
            .new_session_draft
            .as_ref()
            .map(|draft| draft.model.as_str())
            .or_else(|| {
                self.model
                    .active_runtime()
                    .and_then(|runtime| runtime.model.as_deref())
            });
        let model = model_id.and_then(|model| self.models.iter().find(|item| item.model == model));
        let efforts = thinking_segments(model);
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
