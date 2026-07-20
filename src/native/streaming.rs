//! Streaming markdown state for the active conversation.
//!
//! The reference Web UI (`markstream-vue`) keeps a live incremental AST
//! during `assistant.delta` / `thinking.delta` events so tokens stream in
//! without reparsing the whole document. The Rust native shell historically
//! rebuilt every `TextView::markdown(.., full_text)` on each delta, which
//! forces [`TextViewState::set_text`] to reparse the entire markdown string
//! synchronously on the UI thread — the source of the choppy chat output.
//!
//! This module owns a pair of [`TextViewState`] entities for the active
//! conversation's assistant + thinking streams. Instead of replacing the
//! whole text on every event, it diffs against what the state already holds
//! and forwards only the new suffix via [`TextViewState::push_str`]. The
//! gpui-component parse pipeline coalesces up to 64 pending appends per
//! background reparse, which is what makes streaming feel smooth on the web
//! side and now here too.
//!
//! State lifecycle:
//! - [`start_streaming`] creates the entities when a turn starts.
//! - [`Streaming::sync_assistant`] / [`Streaming::sync_thinking`] run on each
//!   delta; no-op if unchanged.
//! - Clearing `Shell::streaming` drops both entities when the turn completes
//!   or the active session changes, so the next turn starts from a clean parse.

use gpui::{AppContext, Context, Entity};
use gpui_component::text::{TextView, TextViewState};

use crate::native::app::Shell;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Slot {
    Assistant,
    Thinking,
}

/// Incremental streaming state for the active conversation.
///
/// Held on [`Shell`] as `Option<Streaming>` so it can be torn down between
/// turns without touching the rest of the shell.
pub(in crate::native) struct Streaming {
    assistant: Entity<TextViewState>,
    thinking: Entity<TextViewState>,
    /// The full source string the assistant entity currently holds. Tracked
    /// locally so we can compute the new suffix without a read-modify-write
    /// through the entity on every event.
    assistant_text: String,
    thinking_text: String,
}

impl Streaming {
    /// Push any new suffix of `target` into the assistant entity. Returns
    /// true if the view should be remeasured (the text actually grew).
    pub fn sync_assistant(&mut self, target: &str, cx: &mut Context<Shell>) -> bool {
        self.sync_slot(Slot::Assistant, target, cx)
    }

    /// Push any new suffix of `target` into the thinking entity.
    pub fn sync_thinking(&mut self, target: &str, cx: &mut Context<Shell>) -> bool {
        self.sync_slot(Slot::Thinking, target, cx)
    }

    /// Read-only access for the view layer to mount the managed entity.
    pub fn assistant_entity(&self) -> &Entity<TextViewState> {
        &self.assistant
    }

    pub fn thinking_entity(&self) -> &Entity<TextViewState> {
        &self.thinking
    }

    fn sync_slot(
        &mut self,
        slot: Slot,
        target: &str,
        cx: &mut Context<Shell>,
    ) -> bool {
        let current = match slot {
            Slot::Assistant => &mut self.assistant_text,
            Slot::Thinking => &mut self.thinking_text,
        };
        // Fast path: the model snapshot equals what the entity already holds.
        // This fires on every redraw between deltas and must stay cheap.
        if current.as_str() == target {
            return false;
        }
        let entity = match slot {
            Slot::Assistant => &self.assistant,
            Slot::Thinking => &self.thinking,
        };
        // Append-only delta path. If the daemon ever rewinds (offset 0 with a
        // shorter string), fall back to a full replace so we never show stale
        // content spliced behind the new turn.
        if let Some(suffix) = diff_suffix(current, target) {
            entity.update(cx, |state, cx| state.push_str(suffix, cx));
        } else {
            entity.update(cx, |state, cx| state.set_text(target, cx));
        }
        current.clear();
        current.push_str(target);
        true
    }
}

/// Return the new suffix when `next` is `current` with bytes appended, or
/// `None` when the two diverge (truncation / rewrite) and the caller should
/// fall back to a full replace.
fn diff_suffix<'a>(current: &str, next: &'a str) -> Option<&'a str> {
    if next.len() < current.len() {
        return None;
    }
    if !next.starts_with(current) {
        return None;
    }
    Some(&next[current.len()..])
}

/// Create a fresh streaming pair bound to the shell context.
pub(in crate::native) fn start_streaming(cx: &mut Context<Shell>) -> Streaming {
    let assistant = cx.new(|cx| TextViewState::markdown("", cx));
    let thinking = cx.new(|cx| TextViewState::markdown("", cx));
    Streaming {
        assistant,
        thinking,
        assistant_text: String::new(),
        thinking_text: String::new(),
    }
}

/// Build a selectable markdown [`TextView`] bound to a managed entity.
///
/// Used in place of the stateless `TextView::markdown(id, full_text)` form
/// for streaming rows so the parsed AST persists across renders.
pub(in crate::native) fn streaming_text_view(entity: &Entity<TextViewState>) -> TextView {
    TextView::new(entity).selectable(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_suffix_returns_only_appended_bytes() {
        assert_eq!(diff_suffix("hello", "hello world"), Some(" world"));
        assert_eq!(diff_suffix("", "abc"), Some("abc"));
        assert_eq!(diff_suffix("abc", "abc"), Some(""));
    }

    #[test]
    fn diff_suffix_rejects_rewrites_and_truncations() {
        assert_eq!(diff_suffix("abc", "ab"), None);
        assert_eq!(diff_suffix("abc", "abX"), None);
        assert_eq!(diff_suffix("hello", "hi world"), None);
    }

    #[test]
    fn diff_suffix_handles_multibyte_boundaries() {
        // 中 = 3 bytes; ensure we slice at the right boundary when the new
        // text extends the existing CJK run.
        assert_eq!(diff_suffix("你好", "你好吗"), Some("吗"));
        // Mixing a multibyte suffix after ascii prefix.
        assert_eq!(diff_suffix("a", "a中"), Some("中"));
        // Truncating mid-codepoint is impossible because &str[start..] is
        // always char-aligned when start is a valid boundary; the helper
        // relies on next.len()/current.len() in bytes but only ever returns
        // a &str slice of `next`, which is itself valid UTF-8.
    }
}
