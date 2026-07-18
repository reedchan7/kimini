use std::collections::HashSet;

use gpui::{AppContext, Context, Window};
use gpui_component::input::{InputEvent, InputState};

use super::drafts::OtherInput;
use crate::native::app::Shell;

impl Shell {
    pub(in crate::native) fn sync_question_inputs(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let requests = self
            .model
            .active_conversation()
            .map(|conversation| conversation.questions.clone())
            .unwrap_or_default();
        let active = requests
            .iter()
            .flat_map(|request| {
                request
                    .questions
                    .iter()
                    .map(|item| (request.question_id.clone(), item.id.clone()))
            })
            .collect::<HashSet<_>>();
        self.question_drafts.retain_active(&active);

        for request in requests {
            self.question_drafts.seed_recommended(&request);
            for item in request
                .questions
                .into_iter()
                .filter(|item| item.allow_other)
            {
                let key = (request.question_id.clone(), item.id.clone());
                if self.question_drafts.other_inputs.contains_key(&key) {
                    continue;
                }
                let placeholder = item
                    .other_label
                    .unwrap_or_else(|| self.strings.native.other_answer.into());
                let input = cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder(placeholder)
                        .default_value("")
                });
                let observed = input.clone();
                let request_id = request.question_id.clone();
                let item_id = item.id.clone();
                let multi = item.multi_select;
                let subscription = cx.subscribe(&input, move |this, _, event: &InputEvent, cx| {
                    if !matches!(event, InputEvent::Change) {
                        return;
                    }
                    let value = observed.read(cx).value().to_string();
                    this.question_drafts
                        .set_other_text(&request_id, &item_id, value, multi);
                    cx.notify();
                });
                self.question_drafts.insert_other_input(
                    key,
                    OtherInput {
                        input,
                        _subscription: subscription,
                    },
                );
            }
        }
    }

    pub(in crate::native) fn toggle_question_option(
        &mut self,
        question_id: String,
        item_id: String,
        option_id: String,
        multi: bool,
        cx: &mut Context<Self>,
    ) {
        self.question_drafts
            .toggle(&question_id, &item_id, &option_id, multi);
        cx.notify();
    }
}
