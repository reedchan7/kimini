use gpui::{AppContext, Context};

use crate::api::KimiConfig;

use super::super::app::{DefaultPermission, LoadState, Shell};

pub(in crate::native) enum ConfigPreference {
    DefaultModel(String),
    DefaultPermission(DefaultPermission),
    DefaultThinking(bool),
    DefaultPlanMode(bool),
    MergeSkills(bool),
    Telemetry(bool),
}

impl ConfigPreference {
    fn patch(&self) -> serde_json::Value {
        match self {
            Self::DefaultModel(model) => serde_json::json!({
                "default_model": model
            }),
            Self::DefaultPermission(permission) => serde_json::json!({
                "default_permission_mode": permission.as_mode()
            }),
            Self::DefaultThinking(enabled) => {
                serde_json::json!({ "thinking": { "enabled": enabled } })
            }
            Self::DefaultPlanMode(enabled) => {
                serde_json::json!({ "default_plan_mode": enabled })
            }
            Self::MergeSkills(enabled) => {
                serde_json::json!({ "merge_all_available_skills": enabled })
            }
            Self::Telemetry(enabled) => serde_json::json!({ "telemetry": enabled }),
        }
    }
}

impl Shell {
    pub(in crate::native) fn update_config_preference(
        &mut self,
        preference: ConfigPreference,
        cx: &mut Context<Self>,
    ) {
        if self.config_saving {
            return;
        }
        let Some(client) = self.client.clone() else {
            return;
        };
        self.config_saving = true;
        self.config_error = None;
        let patch = preference.patch();
        let task = cx.background_spawn(async move { client.patch_config(&patch) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.config_saving = false;
                match result {
                    Ok(config) => this.install_daemon_config(config),
                    Err(error) => {
                        let error = error.to_string();
                        this.config_error = Some(error.clone());
                        this.state = LoadState::Failed(error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn install_daemon_config(&mut self, config: KimiConfig) {
        if let Some(summary) = self.auth.summary.as_mut() {
            summary.default_model = config.default_model.clone();
        }
        self.daemon_config = Some(config);
        self.config_error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_patch_preserves_the_catalog_identifier() {
        let patch = ConfigPreference::DefaultModel("provider/future-model".into()).patch();

        assert_eq!(
            patch,
            serde_json::json!({ "default_model": "provider/future-model" })
        );
    }
}
