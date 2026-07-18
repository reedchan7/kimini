use crate::protocol::{Message, MessageContent, MessageRole};

use super::tool::{ToolCard, ToolIndex, display_value};

#[derive(Debug, Clone, PartialEq)]
pub(in crate::native) enum TranscriptBlock {
    Text(String),
    Thinking(String),
    Tool(ToolCard),
    Attachment {
        kind: AttachmentKind,
        name: String,
        detail: String,
    },
    Unknown {
        kind: String,
        value: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::native) enum AttachmentKind {
    Image,
    Video,
    File,
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::native) struct TranscriptRow {
    pub(in crate::native) id: String,
    pub(in crate::native) role: MessageRole,
    pub(in crate::native) blocks: Vec<TranscriptBlock>,
    pub(in crate::native) streaming: bool,
}

impl TranscriptRow {
    pub(super) fn from_message(message: &Message, tools: &ToolIndex) -> Option<Self> {
        let blocks = message
            .content
            .iter()
            .filter_map(|content| block_from_content(content, tools))
            .collect::<Vec<_>>();
        (!blocks.is_empty()).then(|| Self {
            id: message.id.clone(),
            role: message.role,
            blocks,
            streaming: false,
        })
    }

    pub(in crate::native) fn from_stream(
        session_id: &str,
        thinking: Option<&str>,
        assistant: Option<&str>,
    ) -> Option<Self> {
        let mut blocks = Vec::new();
        if let Some(text) = thinking.filter(|text| !text.is_empty()) {
            blocks.push(TranscriptBlock::Thinking(text.into()));
        }
        if let Some(text) = assistant.filter(|text| !text.is_empty()) {
            blocks.push(TranscriptBlock::Text(text.into()));
        }
        (!blocks.is_empty()).then(|| Self {
            id: format!("stream:{session_id}"),
            role: MessageRole::Assistant,
            blocks,
            streaming: true,
        })
    }

    pub(in crate::native) fn accessible_text(&self) -> String {
        self.blocks
            .iter()
            .map(|block| match block {
                TranscriptBlock::Text(text) | TranscriptBlock::Thinking(text) => text.clone(),
                TranscriptBlock::Tool(tool) => tool.accessible_text(),
                TranscriptBlock::Attachment { kind, name, detail } => {
                    format!("{kind:?}: {name} {detail}")
                }
                TranscriptBlock::Unknown { kind, value } => format!("{kind}: {value}"),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn block_from_content(content: &MessageContent, tools: &ToolIndex) -> Option<TranscriptBlock> {
    match content {
        MessageContent::Text(text) => Some(TranscriptBlock::Text(text.text.clone())),
        MessageContent::Thinking(thinking) => {
            Some(TranscriptBlock::Thinking(thinking.thinking.clone()))
        }
        MessageContent::ToolUse(tool) => Some(TranscriptBlock::Tool(tools.card(
            &tool.tool_call_id,
            &tool.tool_name,
            &tool.input,
        ))),
        MessageContent::ToolResult(result) if tools.has_use(&result.tool_call_id) => None,
        MessageContent::ToolResult(result) => {
            Some(TranscriptBlock::Tool(ToolCard::detached_result(
                &result.tool_call_id,
                display_value(&result.output),
                result.is_error,
            )))
        }
        MessageContent::Image(image) => Some(TranscriptBlock::Attachment {
            kind: AttachmentKind::Image,
            name: "Image".into(),
            detail: image.source.display_reference(),
        }),
        MessageContent::Video(video) => Some(TranscriptBlock::Attachment {
            kind: AttachmentKind::Video,
            name: "Video".into(),
            detail: video.source.display_reference(),
        }),
        MessageContent::File(file) => Some(TranscriptBlock::Attachment {
            kind: AttachmentKind::File,
            name: file.name.clone(),
            detail: format!("{} · {} bytes", file.media_type, file.size),
        }),
        MessageContent::Other(value) => Some(TranscriptBlock::Unknown {
            kind: value
                .get("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("Unsupported content")
                .to_owned(),
            value: display_value(value),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaming_row_keeps_thinking_and_answer_separate() {
        let row = TranscriptRow::from_stream("s", Some("plan"), Some("answer")).unwrap();
        assert_eq!(
            row.blocks,
            vec![
                TranscriptBlock::Thinking("plan".into()),
                TranscriptBlock::Text("answer".into())
            ]
        );
        assert!(row.streaming);
    }

    #[test]
    fn empty_protocol_messages_do_not_create_blank_conversation_cards() {
        let message: Message = serde_json::from_value(serde_json::json!({
            "id": "empty", "session_id": "session", "role": "assistant",
            "content": [], "created_at": "2026-07-18T08:00:00.000Z"
        }))
        .unwrap();

        assert!(TranscriptRow::from_message(&message, &ToolIndex::default()).is_none());
    }

    #[test]
    fn tool_results_are_attached_to_their_invocation_and_not_rendered_twice() {
        let messages: Vec<Message> = serde_json::from_value(serde_json::json!([
            {
                "id": "use", "session_id": "session", "role": "assistant",
                "content": [{
                    "type": "tool_use", "tool_call_id": "call", "tool_name": "Read",
                    "input": { "path": "/tmp/main.rs" }
                }],
                "created_at": "2026-07-18T08:00:00.000Z"
            },
            {
                "id": "result", "session_id": "session", "role": "tool",
                "content": [{
                    "type": "tool_result", "tool_call_id": "call", "output": "contents"
                }],
                "created_at": "2026-07-18T08:00:01.000Z"
            }
        ]))
        .unwrap();
        let index = ToolIndex::from_messages(&messages);

        let row = TranscriptRow::from_message(&messages[0], &index).unwrap();
        let TranscriptBlock::Tool(tool) = &row.blocks[0] else {
            panic!("expected a tool card");
        };
        assert_eq!(tool.summary, "/tmp/main.rs");
        assert_eq!(tool.output.as_deref(), Some("contents"));
        assert!(TranscriptRow::from_message(&messages[1], &index).is_none());
    }
}
