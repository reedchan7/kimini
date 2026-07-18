use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMeta {
    pub id: String,
    pub name: String,
    pub media_type: String,
    pub size: u64,
    pub created_at: String,
    #[serde(default)]
    pub expires_at: Option<String>,
}
