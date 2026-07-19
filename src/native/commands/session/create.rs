use gpui::{Context, PathPromptOptions, Window};

use crate::native::app::{NewSessionDraft, Shell};
use crate::native::prompt_runtime::thinking_segments;

impl Shell {
    pub(in crate::native) fn begin_new_session(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.client.is_none() {
            return;
        }
        if self
            .new_session_draft
            .as_ref()
            .is_some_and(|draft| draft.submitting)
        {
            return;
        }
        self.store_active_composer_draft(cx);
        let previous_draft_key = self.new_session_draft.as_ref().map(NewSessionDraft::key);
        let cwd = self
            .new_session_draft
            .as_ref()
            .map(|draft| draft.cwd.clone())
            .or_else(|| {
                self.model
                    .active_session()
                    .map(|session| session.metadata.cwd.clone())
            })
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .map(|path| path.to_string_lossy().into_owned())
            })
            .unwrap_or_default();
        let model = self
            .daemon_config
            .as_ref()
            .and_then(|config| config.default_model.clone())
            .or_else(|| self.preferred_model())
            .unwrap_or_default();
        let catalog = self.models.iter().find(|item| item.model == model);
        let efforts = thinking_segments(catalog);
        let thinking = catalog
            .and_then(|item| item.default_effort.clone())
            .filter(|effort| efforts.contains(effort))
            .or_else(|| efforts.first().cloned())
            .unwrap_or_else(|| "off".into());
        if let Some(previous_draft_key) = previous_draft_key {
            self.attachments.discard_session(&previous_draft_key);
            self.drafts.remove(&previous_draft_key);
        }
        self.new_session_generation = self.new_session_generation.wrapping_add(1);
        self.new_session_draft = Some(NewSessionDraft {
            id: self.new_session_generation,
            cwd,
            model,
            thinking,
            permission: self.preferences.composer_permission.as_mode().into(),
            plan_mode: false,
            swarm_mode: false,
            submitting: false,
        });
        self.utility_panel = None;
        self.browser = None;
        self.renaming_session = false;
        self.composer_menu = None;
        self.draft_workspace_menu_open = false;
        self.draft_workspace_show_all = false;
        self.composer_session_id = None;
        self.composer
            .update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    pub(in crate::native) fn set_draft_workspace(&mut self, cwd: String, cx: &mut Context<Self>) {
        if self
            .new_session_draft
            .as_ref()
            .is_none_or(|draft| draft.submitting)
        {
            return;
        }
        self.store_active_composer_draft(cx);
        if let Some(draft) = self.new_session_draft.as_mut() {
            draft.cwd = cwd;
        }
        cx.notify();
    }

    pub(in crate::native) fn choose_draft_workspace(&mut self, cx: &mut Context<Self>) {
        if self.new_session_draft.is_none() {
            return;
        }
        let selection = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(self.strings.native.choose_folder.into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = selection.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let cwd = path.to_string_lossy().into_owned();
            let _ = this.update(cx, |this, cx| this.set_draft_workspace(cwd, cx));
        })
        .detach();
    }
}
