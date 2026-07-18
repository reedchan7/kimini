mod diff;
mod summary;

use std::collections::{HashMap, HashSet};

use crate::protocol::{Message, MessageContent};

pub(super) use diff::ToolDiff;
use summary::{clip_chars, tool_detail, tool_summary};

pub(super) fn display_value(value: &serde_json::Value) -> String {
    summary::display_value(value)
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::native) struct ToolCard {
    pub id: String,
    pub name: Option<String>,
    pub summary: String,
    pub detail: Option<String>,
    pub output: Option<String>,
    pub is_error: bool,
    pub running: bool,
    pub diff: Option<ToolDiff>,
}

impl ToolCard {
    pub fn accessible_text(&self) -> String {
        self.name
            .as_ref()
            .map(|name| format!("{name}: {}", self.summary))
            .unwrap_or_else(|| self.summary.clone())
    }

    pub fn has_details(&self) -> bool {
        self.diff.is_some() || self.detail.is_some() || self.output.is_some()
    }

    pub fn detached_result(id: &str, output: String, is_error: bool) -> Self {
        let summary = output
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(str::trim)
            .unwrap_or("—")
            .to_owned();
        Self {
            id: id.to_owned(),
            name: None,
            summary: clip_chars(summary),
            detail: None,
            output: Some(output),
            is_error,
            running: false,
            diff: None,
        }
    }
}

#[derive(Debug, Clone)]
struct ToolOutcome {
    output: String,
    is_error: bool,
}

#[derive(Debug, Default)]
pub(super) struct ToolIndex {
    uses: HashSet<String>,
    outcomes: HashMap<String, ToolOutcome>,
}

impl ToolIndex {
    pub fn from_messages(messages: &[Message]) -> Self {
        let mut index = Self::default();
        for content in messages.iter().flat_map(|message| &message.content) {
            match content {
                MessageContent::ToolUse(tool) => {
                    index.uses.insert(tool.tool_call_id.clone());
                }
                MessageContent::ToolResult(result) => {
                    index.outcomes.insert(
                        result.tool_call_id.clone(),
                        ToolOutcome {
                            output: display_value(&result.output),
                            is_error: result.is_error,
                        },
                    );
                }
                _ => {}
            }
        }
        index
    }

    pub fn card(&self, id: &str, name: &str, input: &serde_json::Value) -> ToolCard {
        let outcome = self.outcomes.get(id);
        let diff = diff::edit_diff(name, input);
        ToolCard {
            id: id.to_owned(),
            name: Some(name.to_owned()),
            summary: tool_summary(name, input),
            detail: tool_detail(name, input, diff.is_some()),
            output: outcome.map(|outcome| outcome.output.clone()),
            is_error: outcome.is_some_and(|outcome| outcome.is_error),
            running: outcome.is_none(),
            diff,
        }
    }

    pub fn has_use(&self, id: &str) -> bool {
        self.uses.contains(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detached_results_use_a_bounded_summary_and_keep_details_collapsible() {
        let card = ToolCard::detached_result(
            "call",
            format!("\n{}\nrest", "你".repeat(summary::MAX_SUMMARY_CHARS + 20)),
            false,
        );

        assert!(card.name.is_none());
        assert_eq!(card.summary.chars().count(), summary::MAX_SUMMARY_CHARS + 1);
        assert!(card.has_details());
        assert!(!card.accessible_text().contains("rest"));
    }
}
