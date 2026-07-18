use crate::protocol::{AuthSummary, ModelCatalogItem, PromptOptions, SessionStatus};

use super::app::Shell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PromptRuntime {
    model: String,
    thinking: String,
    permission: String,
    plan_mode: bool,
    swarm_mode: bool,
}

impl PromptRuntime {
    pub fn options(&self, agent_id: Option<String>) -> PromptOptions {
        PromptOptions {
            agent_id,
            model: Some(self.model.clone()),
            thinking: Some(self.thinking.clone()),
            permission_mode: Some(self.permission.clone()),
            plan_mode: Some(self.plan_mode),
            swarm_mode: Some(self.swarm_mode),
        }
    }

    pub fn profile_patch(&self) -> serde_json::Value {
        serde_json::json!({
            "model": self.model,
            "thinking": self.thinking,
            "permission_mode": self.permission,
            "plan_mode": self.plan_mode,
            "swarm_mode": self.swarm_mode,
        })
    }
}

impl Shell {
    pub(in crate::native) fn active_prompt_runtime(&self) -> Option<PromptRuntime> {
        resolve_prompt_runtime(
            self.model.active_runtime(),
            self.auth.summary.as_ref(),
            &self.models,
        )
    }

    pub(in crate::native) fn preferred_model(&self) -> Option<String> {
        configured_model(self.auth.summary.as_ref(), &self.models)
    }
}

fn resolve_prompt_runtime(
    runtime: Option<&SessionStatus>,
    auth: Option<&AuthSummary>,
    models: &[ModelCatalogItem],
) -> Option<PromptRuntime> {
    let model = runtime
        .and_then(|runtime| non_empty(runtime.model.as_deref()))
        .map(str::to_owned)
        .or_else(|| configured_model(auth, models))?;
    let catalog = models.iter().find(|item| item.model == model);
    let thinking = runtime
        .and_then(|runtime| non_empty(Some(&runtime.thinking_level)))
        .map(str::to_owned)
        .or_else(|| catalog.and_then(|item| item.default_effort.clone()))
        .unwrap_or_else(|| "off".into());
    let permission = runtime
        .and_then(|runtime| non_empty(Some(&runtime.permission)))
        .unwrap_or("manual")
        .to_owned();

    Some(PromptRuntime {
        model,
        thinking,
        permission,
        plan_mode: runtime.is_some_and(|runtime| runtime.plan_mode),
        swarm_mode: runtime.is_some_and(|runtime| runtime.swarm_mode),
    })
}

fn configured_model(auth: Option<&AuthSummary>, models: &[ModelCatalogItem]) -> Option<String> {
    auth.and_then(|summary| non_empty(summary.default_model.as_deref()))
        .map(str::to_owned)
        .or_else(|| {
            models
                .iter()
                .find_map(|model| non_empty(Some(&model.model)).map(str::to_owned))
        })
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(id: &str, default_effort: Option<&str>) -> ModelCatalogItem {
        ModelCatalogItem {
            provider: "kimi".into(),
            model: id.into(),
            display_name: None,
            max_context_size: 262_144,
            capabilities: vec!["thinking".into()],
            support_efforts: vec!["off".into(), "max".into()],
            default_effort: default_effort.map(str::to_owned),
        }
    }

    fn auth(default_model: Option<&str>) -> AuthSummary {
        AuthSummary {
            ready: true,
            providers_count: 1,
            default_model: default_model.map(str::to_owned),
            managed_provider: None,
        }
    }

    #[test]
    fn live_runtime_values_override_catalog_defaults() {
        let runtime = SessionStatus {
            busy: false,
            model: Some("kimi-code/k3".into()),
            thinking_level: "high".into(),
            permission: "yolo".into(),
            plan_mode: true,
            swarm_mode: false,
            context_tokens: 0,
            max_context_tokens: 262_144,
            context_usage: 0.0,
        };

        let resolved = resolve_prompt_runtime(
            Some(&runtime),
            Some(&auth(Some("fallback"))),
            &[model("kimi-code/k3", Some("max"))],
        )
        .unwrap();

        assert_eq!(
            resolved.options(None),
            PromptOptions {
                agent_id: None,
                model: Some("kimi-code/k3".into()),
                thinking: Some("high".into()),
                permission_mode: Some("yolo".into()),
                plan_mode: Some(true),
                swarm_mode: Some(false),
            }
        );
    }

    #[test]
    fn empty_new_session_status_uses_auth_and_catalog_defaults() {
        let runtime = SessionStatus {
            busy: false,
            model: None,
            thinking_level: "".into(),
            permission: "auto".into(),
            plan_mode: false,
            swarm_mode: true,
            context_tokens: 0,
            max_context_tokens: 0,
            context_usage: 0.0,
        };
        let resolved = resolve_prompt_runtime(
            Some(&runtime),
            Some(&auth(Some("kimi-code/k3"))),
            &[model("kimi-code/k3", Some("max"))],
        )
        .unwrap();

        assert_eq!(
            resolved.options(None).model.as_deref(),
            Some("kimi-code/k3")
        );
        assert_eq!(resolved.options(None).thinking.as_deref(), Some("max"));
        assert_eq!(
            resolved.options(None).permission_mode.as_deref(),
            Some("auto")
        );
    }

    #[test]
    fn catalog_fallback_keeps_first_send_available_without_auth_metadata() {
        let resolved = resolve_prompt_runtime(None, None, &[model("local/model", None)]).unwrap();

        assert_eq!(resolved.options(None).model.as_deref(), Some("local/model"));
        assert_eq!(resolved.options(None).thinking.as_deref(), Some("off"));
        assert_eq!(
            resolved.options(None).permission_mode.as_deref(),
            Some("manual")
        );
    }

    #[test]
    fn no_configured_model_blocks_prompt_construction() {
        assert!(resolve_prompt_runtime(None, Some(&auth(Some("  "))), &[]).is_none());
    }
}
