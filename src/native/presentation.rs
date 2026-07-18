use std::rc::Rc;

use gpui::{Pixels, Size, px, size};

use crate::model::AppModel;
use crate::protocol::MessageRole;

use super::theme::CONTENT_WIDTH;

#[derive(Debug, Clone)]
pub(super) struct TranscriptRow {
    pub role: MessageRole,
    pub text: String,
}

#[derive(Debug, Clone)]
pub(super) struct Transcript {
    pub rows: Vec<TranscriptRow>,
    pub sizes: Rc<Vec<Size<Pixels>>>,
}

impl Default for Transcript {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            sizes: Rc::new(Vec::new()),
        }
    }
}

impl Transcript {
    pub fn rebuild(&mut self, model: &AppModel) {
        let Some(conversation) = model.active_conversation() else {
            self.rows.clear();
            self.sizes = Rc::new(Vec::new());
            return;
        };
        self.rows = conversation
            .messages
            .iter()
            .map(|message| TranscriptRow {
                role: message.role,
                text: message.plain_text(),
            })
            .chain(
                conversation
                    .assistant_stream
                    .iter()
                    .map(|text| TranscriptRow {
                        role: MessageRole::Assistant,
                        text: text.clone(),
                    }),
            )
            .collect();
        self.sizes = Rc::new(self.rows.iter().map(|row| row_size(&row.text)).collect());
    }

    pub fn sync_stream(&mut self, model: &AppModel) {
        let Some(conversation) = model.active_conversation() else {
            self.rebuild(model);
            return;
        };
        let message_count = conversation.messages.len();
        let Some(text) = conversation.assistant_stream.as_ref() else {
            if self.rows.len() == message_count + 1 {
                self.rows.pop();
                Rc::make_mut(&mut self.sizes).pop();
            } else if self.rows.len() != message_count {
                self.rebuild(model);
            }
            return;
        };

        if self.rows.len() == message_count {
            self.rows.push(TranscriptRow {
                role: MessageRole::Assistant,
                text: text.clone(),
            });
            Rc::make_mut(&mut self.sizes).push(row_size(text));
        } else if self.rows.len() == message_count + 1
            && self.rows[message_count].role == MessageRole::Assistant
        {
            self.rows[message_count].text.clone_from(text);
            Rc::make_mut(&mut self.sizes)[message_count] = row_size(text);
        } else {
            self.rebuild(model);
        }
    }
}

fn row_size(text: &str) -> Size<Pixels> {
    size(px(CONTENT_WIDTH), px(estimated_height(text)))
}

fn estimated_height(text: &str) -> f32 {
    let lines: usize = text
        .split('\n')
        .map(|line| line.chars().count().div_ceil(84).max(1))
        .sum();
    54.0 + lines as f32 * 20.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimated_rows_cover_explicit_and_long_wrapped_lines() {
        assert_eq!(estimated_height("a\nb"), 94.0);
        assert!(estimated_height(&"x".repeat(84 * 30)) > 54.0 + 24.0 * 20.0);
    }
}
