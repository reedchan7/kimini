use std::collections::{BTreeSet, HashMap, HashSet};

use gpui::{Entity, Subscription};
use gpui_component::input::InputState;

use crate::protocol::{QuestionAnswer, QuestionAnswers, QuestionRequest};

pub(super) type DraftKey = (String, String);

pub(super) struct OtherInput {
    pub input: Entity<InputState>,
    pub _subscription: Subscription,
}

#[derive(Default)]
pub(in crate::native) struct QuestionDrafts {
    pub(super) selections: HashMap<DraftKey, BTreeSet<String>>,
    pub(super) other_selected: HashSet<DraftKey>,
    pub(super) other_text: HashMap<DraftKey, String>,
    pub(super) other_inputs: HashMap<DraftKey, OtherInput>,
}

impl QuestionDrafts {
    pub fn toggle(&mut self, request_id: &str, item_id: &str, option_id: &str, multi: bool) {
        let key = (request_id.into(), item_id.into());
        let selected = self.selections.entry(key).or_default();
        if !multi {
            selected.clear();
            selected.insert(option_id.into());
            self.other_selected
                .remove(&(request_id.into(), item_id.into()));
        } else if !selected.remove(option_id) {
            selected.insert(option_id.into());
        }
    }

    pub fn is_selected(&self, request_id: &str, item_id: &str, option_id: &str) -> bool {
        self.selections
            .get(&(request_id.into(), item_id.into()))
            .is_some_and(|selected| selected.contains(option_id))
    }

    pub fn answers(&self, request: &QuestionRequest) -> Option<QuestionAnswers> {
        request
            .questions
            .iter()
            .map(|item| {
                let key = (request.question_id.clone(), item.id.clone());
                let option_ids = self
                    .selections
                    .get(&key)
                    .map(|selected| selected.iter().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                let other_text = self
                    .other_text
                    .get(&key)
                    .map(|text| text.trim())
                    .unwrap_or_default();
                let other_selected = item.allow_other && self.other_selected.contains(&key);
                let answer = if item.multi_select && other_selected && !other_text.is_empty() {
                    QuestionAnswer::MultiWithOther {
                        option_ids,
                        other_text: other_text.into(),
                    }
                } else if item.multi_select && !option_ids.is_empty() {
                    QuestionAnswer::Multi { option_ids }
                } else if !item.multi_select && other_selected && !other_text.is_empty() {
                    QuestionAnswer::Other {
                        text: other_text.into(),
                    }
                } else if !item.multi_select {
                    QuestionAnswer::Single {
                        option_id: option_ids.into_iter().next()?,
                    }
                } else {
                    return None;
                };
                Some((item.id.clone(), answer))
            })
            .collect()
    }

    pub fn is_other_selected(&self, request_id: &str, item_id: &str) -> bool {
        self.other_selected
            .contains(&(request_id.into(), item_id.into()))
    }

    pub fn other_input(&self, request_id: &str, item_id: &str) -> Option<Entity<InputState>> {
        self.other_inputs
            .get(&(request_id.into(), item_id.into()))
            .map(|draft| draft.input.clone())
    }

    pub(super) fn set_other_text(
        &mut self,
        request_id: &str,
        item_id: &str,
        text: String,
        multi: bool,
    ) {
        let key = (request_id.into(), item_id.into());
        if !text.trim().is_empty() {
            self.other_selected.insert(key.clone());
            if !multi {
                self.selections.remove(&key);
            }
        }
        self.other_text.insert(key, text);
    }

    pub(super) fn seed_recommended(&mut self, request: &QuestionRequest) {
        for item in &request.questions {
            let key = (request.question_id.clone(), item.id.clone());
            if self.selections.contains_key(&key) || self.other_selected.contains(&key) {
                continue;
            }
            let recommended = item
                .options
                .iter()
                .filter(|option| option.recommended)
                .map(|option| option.id.clone())
                .collect::<BTreeSet<_>>();
            if recommended.is_empty() {
                continue;
            }
            self.selections.insert(
                key,
                if item.multi_select {
                    recommended
                } else {
                    recommended.into_iter().take(1).collect()
                },
            );
        }
    }

    pub(super) fn retain_active(&mut self, active: &HashSet<DraftKey>) {
        self.selections.retain(|key, _| active.contains(key));
        self.other_selected.retain(|key| active.contains(key));
        self.other_text.retain(|key, _| active.contains(key));
        self.other_inputs.retain(|key, _| active.contains(key));
    }

    pub(super) fn insert_other_input(&mut self, key: DraftKey, draft: OtherInput) {
        self.other_inputs.insert(key, draft);
    }

    pub fn remove(&mut self, request_id: &str) {
        self.selections
            .retain(|(candidate, _), _| candidate != request_id);
        self.other_selected
            .retain(|(candidate, _)| candidate != request_id);
        self.other_text
            .retain(|(candidate, _), _| candidate != request_id);
        self.other_inputs
            .retain(|(candidate, _), _| candidate != request_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> QuestionRequest {
        serde_json::from_value(serde_json::json!({
            "question_id": "q", "session_id": "s", "created_at": "now",
            "questions": [
                { "id": "single", "question": "One?", "options": [
                    { "id": "a", "label": "A" }, { "id": "b", "label": "B" }
                ]},
                { "id": "multi", "question": "Many?", "multi_select": true, "options": [
                    { "id": "x", "label": "X" }, { "id": "y", "label": "Y" }
                ]}
            ]
        }))
        .unwrap()
    }

    #[test]
    fn builds_one_atomic_answer_map_for_single_and_multi_items() {
        let request = request();
        let mut drafts = QuestionDrafts::default();
        drafts.toggle("q", "single", "a", false);
        assert!(drafts.answers(&request).is_none());
        drafts.toggle("q", "multi", "x", true);
        drafts.toggle("q", "multi", "y", true);

        let answers = drafts.answers(&request).unwrap();
        assert!(matches!(answers["single"], QuestionAnswer::Single { .. }));
        assert_eq!(
            answers["multi"],
            QuestionAnswer::Multi {
                option_ids: vec!["x".into(), "y".into()]
            }
        );
    }

    #[test]
    fn free_text_answers_replace_single_choices_and_extend_multi_choices() {
        let request: QuestionRequest = serde_json::from_value(serde_json::json!({
            "question_id": "q", "session_id": "s", "created_at": "now",
            "questions": [
                { "id": "single", "question": "One?", "allow_other": true,
                  "options": [{ "id": "a", "label": "A" }] },
                { "id": "multi", "question": "Many?", "multi_select": true,
                  "allow_other": true, "options": [{ "id": "x", "label": "X" }] }
            ]
        }))
        .unwrap();
        let mut drafts = QuestionDrafts::default();
        drafts.toggle("q", "single", "a", false);
        drafts.set_other_text("q", "single", "custom".into(), false);
        drafts.toggle("q", "multi", "x", true);
        drafts.set_other_text("q", "multi", "tail".into(), true);

        let answers = drafts.answers(&request).unwrap();
        assert_eq!(
            answers["single"],
            QuestionAnswer::Other {
                text: "custom".into()
            }
        );
        assert_eq!(
            answers["multi"],
            QuestionAnswer::MultiWithOther {
                option_ids: vec!["x".into()],
                other_text: "tail".into()
            }
        );
    }

    #[test]
    fn recommended_options_are_seeded_once() {
        let request: QuestionRequest = serde_json::from_value(serde_json::json!({
            "question_id": "q", "session_id": "s", "created_at": "now",
            "questions": [{ "id": "single", "question": "One?", "options": [
                { "id": "a", "label": "A", "recommended": true },
                { "id": "b", "label": "B", "recommended": true }
            ]}]
        }))
        .unwrap();
        let mut drafts = QuestionDrafts::default();

        drafts.seed_recommended(&request);

        assert_eq!(
            drafts.answers(&request).unwrap()["single"],
            QuestionAnswer::Single {
                option_id: "a".into()
            }
        );
    }
}
