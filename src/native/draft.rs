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
    pub(super) fn store_active_composer_draft(&mut self, cx: &mut Context<Self>) {
        let Some(session_id) = self
            .model
            .active_session()
            .map(|session| session.id.clone())
        else {
            return;
        };
        self.drafts
            .set(&session_id, self.composer.read(cx).value().to_string());
    }

    pub(super) fn sync_composer_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let active_session_id = self
            .model
            .active_session()
            .map(|session| session.id.clone());
        if self.composer_session_id == active_session_id {
            return;
        }
        self.composer_session_id = active_session_id.clone();
        let value = active_session_id
            .as_deref()
            .map(|session_id| self.drafts.get(session_id))
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
