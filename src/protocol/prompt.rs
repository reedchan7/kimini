use serde::{Deserialize, Serialize};

use super::{FileMeta, MessageContent};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PromptPart {
    Text {
        text: String,
    },
    Image {
        source: UploadedFileSource,
    },
    Video {
        source: UploadedFileSource,
    },
    File {
        file_id: String,
        name: String,
        media_type: String,
        size: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum UploadedFileSource {
    File { file_id: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PromptOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_mode: Option<bool>,
}

impl PromptPart {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    pub fn uploaded(file: &FileMeta) -> Self {
        let source = UploadedFileSource::File {
            file_id: file.id.clone(),
        };
        if file.media_type.starts_with("image/") {
            Self::Image { source }
        } else if file.media_type.starts_with("video/") {
            Self::Video { source }
        } else {
            Self::File {
                file_id: file.id.clone(),
                name: file.name.clone(),
                media_type: file.media_type.clone(),
                size: file.size,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptStatus {
    Running,
    Queued,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PromptItem {
    pub prompt_id: String,
    pub user_message_id: String,
    pub status: PromptStatus,
    pub content: Vec<MessageContent>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PromptQueue {
    pub active: Option<PromptItem>,
    pub queued: Vec<PromptItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PromptSteerResult {
    pub steered: bool,
    pub prompt_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PromptAbortResult {
    pub aborted: bool,
    pub at_seq: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(media_type: &str) -> FileMeta {
        FileMeta {
            id: "f_01".into(),
            name: "asset.bin".into(),
            media_type: media_type.into(),
            size: 42,
            created_at: "2026-07-18T08:00:00.000Z".into(),
            expires_at: None,
        }
    }

    #[test]
    fn uploaded_media_and_files_follow_the_daemon_wire_shape() {
        assert_eq!(
            serde_json::to_value(PromptPart::uploaded(&file("image/png"))).unwrap(),
            serde_json::json!({
                "type": "image", "source": { "kind": "file", "file_id": "f_01" }
            })
        );
        assert_eq!(
            serde_json::to_value(PromptPart::uploaded(&file("application/pdf"))).unwrap(),
            serde_json::json!({
                "type": "file", "file_id": "f_01", "name": "asset.bin",
                "media_type": "application/pdf", "size": 42
            })
        );
    }

    #[test]
    fn side_channel_options_serialize_only_selected_runtime_fields() {
        let options = PromptOptions {
            agent_id: Some("btw_01".into()),
            model: None,
            thinking: Some("high".into()),
            permission_mode: None,
            plan_mode: Some(true),
            swarm_mode: None,
        };
        assert_eq!(
            serde_json::to_value(options).unwrap(),
            serde_json::json!({
                "agent_id": "btw_01",
                "thinking": "high",
                "plan_mode": true
            })
        );
    }
}
