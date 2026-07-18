use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub approval_id: String,
    pub session_id: String,
    #[serde(default)]
    pub turn_id: Option<u64>,
    pub tool_call_id: String,
    pub tool_name: String,
    pub action: String,
    #[serde(default)]
    pub tool_input_display: serde_json::Value,
    pub expires_at: String,
    pub created_at: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionRequest {
    pub question_id: String,
    pub session_id: String,
    #[serde(default)]
    pub turn_id: Option<u64>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub questions: Vec<QuestionItem>,
    pub created_at: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionItem {
    pub id: String,
    pub question: String,
    #[serde(default)]
    pub header: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub options: Vec<QuestionOption>,
    #[serde(default)]
    pub multi_select: bool,
    #[serde(default)]
    pub allow_other: bool,
    #[serde(default)]
    pub other_label: Option<String>,
    #[serde(default)]
    pub other_description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionOption {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub recommended: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QuestionAnswer {
    Single {
        option_id: String,
    },
    Multi {
        option_ids: Vec<String>,
    },
    Other {
        text: String,
    },
    MultiWithOther {
        option_ids: Vec<String>,
        other_text: String,
    },
    Skipped,
}

pub type QuestionAnswers = BTreeMap<String, QuestionAnswer>;
