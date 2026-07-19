use serde::{Deserialize, Serialize};

use super::{ApiError, KimiClient};

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct KimiConfig {
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub thinking: Option<ThinkingConfig>,
    #[serde(default)]
    pub default_permission_mode: Option<String>,
    #[serde(default)]
    pub default_plan_mode: Option<bool>,
    #[serde(default)]
    pub merge_all_available_skills: Option<bool>,
    #[serde(default)]
    pub telemetry: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ThinkingConfig {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub effort: Option<String>,
}

impl KimiClient {
    pub fn get_config(&self) -> Result<KimiConfig, ApiError> {
        self.get("/config")
    }

    pub fn patch_config(&self, patch: &serde_json::Value) -> Result<KimiConfig, ApiError> {
        self.post("/config", patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_accepts_the_daemon_agent_defaults_shape() {
        let config: KimiConfig = serde_json::from_value(serde_json::json!({
            "default_model": "kimi-code/k3",
            "default_permission_mode": "yolo",
            "default_plan_mode": false,
            "thinking": { "enabled": true, "effort": "max" },
            "merge_all_available_skills": true,
            "telemetry": false,
            "providers": {}
        }))
        .unwrap();

        assert_eq!(config.default_model.as_deref(), Some("kimi-code/k3"));
        assert_eq!(
            config.thinking.and_then(|thinking| thinking.enabled),
            Some(true)
        );
        assert_eq!(config.default_permission_mode.as_deref(), Some("yolo"));
        assert_eq!(config.merge_all_available_skills, Some(true));
        assert_eq!(config.telemetry, Some(false));
    }
}
