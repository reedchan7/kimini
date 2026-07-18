use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireEvent {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub seq: u64,
    #[serde(default)]
    pub epoch: Option<String>,
    #[serde(default)]
    pub volatile: bool,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub timestamp: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}
