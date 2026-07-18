use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub workspace_id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub busy: bool,
    #[serde(default)]
    pub main_turn_active: Option<bool>,
    #[serde(default)]
    pub pending_interaction: Option<String>,
    #[serde(default)]
    pub last_turn_reason: Option<String>,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub current_prompt_id: Option<String>,
    #[serde(default)]
    pub last_prompt: Option<String>,
    pub metadata: SessionMetadata,
    pub agent_config: AgentConfig,
    pub usage: SessionUsage,
    #[serde(default)]
    pub permission_rules: Vec<serde_json::Value>,
    pub message_count: u64,
    pub last_seq: u64,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub cwd: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub total_cost_usd: f64,
    pub context_tokens: u64,
    pub context_limit: u64,
    pub turn_count: u64,
}
