use gpui::{AppContext, Context, Window};

use crate::api::ApiError;
use crate::protocol::PromptPart;

use super::super::app::{Shell, UtilityPanel};

impl Shell {
    pub(in crate::native) fn toggle_side_chat(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.utility_panel == Some(UtilityPanel::SideChat) {
            self.utility_panel = None;
            cx.notify();
            return;
        }
        self.open_side_chat(None, window, cx);
    }

    pub(in crate::native) fn open_side_chat(
        &mut self,
        initial: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.utility_panel = Some(UtilityPanel::SideChat);
        self.side_chat_input
            .update(cx, |input, cx| input.focus(window, cx));

        if self.side_chats.get(&session_id).is_some() {
            if let Some(text) = non_empty(initial) {
                self.send_side_chat_text(text, cx);
            }
            cx.notify();
            return;
        }
        if !self.side_chats.begin_open(&session_id) {
            cx.notify();
            return;
        }

        let request_session_id = session_id.clone();
        let task = cx.background_spawn(async move { client.start_side_chat(&session_id) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session_id) {
                    return;
                }
                match result {
                    Ok(started) => {
                        this.side_chats
                            .begin(request_session_id.clone(), started.agent_id);
                        if let Some(text) = non_empty(initial) {
                            this.send_side_chat_text(text, cx);
                        }
                    }
                    Err(error) => this.side_chats.fail_open(&request_session_id, error),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub(in crate::native) fn send_side_chat_prompt(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = self.side_chat_input.read(cx).value().trim().to_owned();
        if text.is_empty() {
            return;
        }
        let Some(session_id) = self
            .model
            .active_session()
            .map(|session| session.id.clone())
        else {
            return;
        };
        if self.side_chats.get(&session_id).is_none() {
            return;
        }
        self.side_chat_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.send_side_chat_text(text, cx);
    }

    fn send_side_chat_text(&mut self, text: String, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        let Some(agent_id) = self
            .side_chats
            .get(&session_id)
            .map(|chat| chat.agent_id.clone())
        else {
            return;
        };
        let Some(runtime) = self.active_prompt_runtime() else {
            self.side_chats
                .set_error(&session_id, self.strings.native.model_required.into());
            cx.notify();
            return;
        };
        let options = runtime.options(Some(agent_id));
        if !self.side_chats.start_turn(&session_id, text.clone()) {
            return;
        }

        let request_session_id = session_id.clone();
        let content = [PromptPart::text(text)];
        let task = cx.background_spawn(async move {
            client.submit_prompt_with_options(&session_id, &content, &options)
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session_id) {
                    return;
                }
                if let Err(error) = result {
                    this.side_chats.fail_turn(&request_session_id, error);
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub(in crate::native) fn stop_side_chat(&mut self, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        let Some(agent_id) = self
            .side_chats
            .get(&session_id)
            .filter(|chat| chat.running)
            .map(|chat| chat.agent_id.clone())
        else {
            return;
        };
        let request_session_id = session_id.clone();
        let task = cx.background_spawn(async move {
            client.cancel_task(&session_id, &agent_id)?;
            Ok::<_, ApiError>(())
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session_id) {
                    return;
                }
                match result {
                    Ok(()) => this.side_chats.finish_turn(&request_session_id),
                    Err(error) => this.side_chats.set_error(&request_session_id, error),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

fn non_empty(text: Option<String>) -> Option<String> {
    text.map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_initial_questions_ignore_whitespace() {
        assert_eq!(
            non_empty(Some("  hello  ".into())).as_deref(),
            Some("hello")
        );
        assert!(non_empty(Some("  ".into())).is_none());
        assert!(non_empty(None).is_none());
    }
}
