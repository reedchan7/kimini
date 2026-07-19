use std::collections::HashMap;

use gpui::{Context, Window};

use super::app::Shell;

#[derive(Debug, Default)]
pub(super) struct ComposerDrafts {
    text_by_session: HashMap<String, String>,
}

impl ComposerDrafts {
    pub fn set(&mut self, session_id: &str, text: String) {
        if text.is_empty() {
            self.text_by_session.remove(session_id);
        } else {
            self.text_by_session.insert(session_id.into(), text);
        }
    }

    pub fn get(&self, session_id: &str) -> &str {
        self.text_by_session
            .get(session_id)
            .map(String::as_str)
            .unwrap_or_default()
    }

    pub fn remove(&mut self, session_id: &str) {
        self.text_by_session.remove(session_id);
    }
}

impl Shell {
    pub(super) fn active_composer_key(&self) -> Option<String> {
        self.new_session_draft
            .as_ref()
            .map(super::app::NewSessionDraft::key)
            .or_else(|| {
                self.model
                    .active_session()
                    .map(|session| session.id.clone())
            })
    }

    pub(super) fn store_active_composer_draft(&mut self, cx: &mut Context<Self>) {
        let Some(key) = self.active_composer_key() else {
            return;
        };
        self.drafts
            .set(&key, self.composer.read(cx).value().to_string());
    }

    pub(super) fn sync_composer_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let active_key = self.active_composer_key();
        if self.composer_session_id == active_key {
            return;
        }
        self.composer_session_id = active_key.clone();
        let value = active_key
            .as_deref()
            .map(|key| self.drafts.get(key))
            .unwrap_or_default()
            .to_owned();
        self.composer
            .update(cx, |input, cx| input.set_value(value, window, cx));
    }

    pub(super) fn restore_composer_draft(&mut self, session_id: &str, text: String) {
        self.drafts.set(session_id, text);
        if self
            .model
            .active_session()
            .is_some_and(|session| session.id == session_id)
        {
            self.composer_session_id = None;
        }
    }

    pub(super) fn restore_new_session_draft(&mut self, key: &str, text: String) {
        self.drafts.set(key, text);
        if self.active_composer_key().as_deref() == Some(key) {
            self.composer_session_id = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_session_draft(id: u64, cwd: &str) -> super::super::app::NewSessionDraft {
        super::super::app::NewSessionDraft {
            id,
            cwd: cwd.into(),
            model: "kimi-code/k3".into(),
            thinking: "high".into(),
            permission: "manual".into(),
            plan_mode: false,
            swarm_mode: false,
            submitting: false,
        }
    }

    #[test]
    fn changing_workspace_does_not_change_a_draft_key() {
        let mut draft = new_session_draft(7, "/workspace/one");
        let key = draft.key();

        draft.cwd = "/workspace/two".into();

        assert_eq!(draft.key(), key);
    }

    #[test]
    fn drafts_are_isolated_and_empty_text_clears_storage() {
        let mut drafts = ComposerDrafts::default();
        drafts.set("a", "alpha".into());
        drafts.set("b", "beta".into());

        assert_eq!(drafts.get("a"), "alpha");
        assert_eq!(drafts.get("b"), "beta");

        drafts.set("a", String::new());
        assert_eq!(drafts.get("a"), "");
        assert_eq!(drafts.get("b"), "beta");
    }
}
