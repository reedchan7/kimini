use std::collections::HashMap;
use std::path::PathBuf;

use gpui::{AppContext, Context, PathPromptOptions};

use crate::protocol::{FileMeta, PromptPart};

use super::app::Shell;

#[derive(Debug, Clone)]
pub(super) struct AttachmentDraft {
    pub id: u64,
    pub name: String,
    pub state: AttachmentState,
}

#[derive(Debug, Clone)]
pub(super) enum AttachmentState {
    Uploading,
    Ready(FileMeta),
    Failed(String),
}

#[derive(Debug, Default)]
pub(super) struct Attachments {
    by_session: HashMap<String, Vec<AttachmentDraft>>,
    next_id: u64,
}

impl Attachments {
    pub fn for_session(&self, session_id: &str) -> &[AttachmentDraft] {
        self.by_session
            .get(session_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn add_upload(&mut self, session_id: &str, name: String) -> u64 {
        self.next_id = self.next_id.wrapping_add(1);
        let id = self.next_id;
        self.by_session
            .entry(session_id.into())
            .or_default()
            .push(AttachmentDraft {
                id,
                name,
                state: AttachmentState::Uploading,
            });
        id
    }

    pub fn finish(&mut self, session_id: &str, id: u64, result: Result<FileMeta, String>) {
        let Some(draft) = self
            .by_session
            .get_mut(session_id)
            .and_then(|drafts| drafts.iter_mut().find(|draft| draft.id == id))
        else {
            return;
        };
        draft.state = match result {
            Ok(file) => AttachmentState::Ready(file),
            Err(error) => AttachmentState::Failed(error),
        };
    }

    pub fn remove(&mut self, session_id: &str, id: u64) {
        if let Some(drafts) = self.by_session.get_mut(session_id) {
            drafts.retain(|draft| draft.id != id);
        }
    }

    pub fn has_uploads(&self, session_id: &str) -> bool {
        self.for_session(session_id)
            .iter()
            .any(|draft| matches!(draft.state, AttachmentState::Uploading))
    }

    pub fn prompt_parts(&self, session_id: &str, text: &str) -> Vec<PromptPart> {
        let mut parts = Vec::new();
        if !text.is_empty() {
            parts.push(PromptPart::text(text));
        }
        parts.extend(
            self.for_session(session_id)
                .iter()
                .filter_map(|draft| match &draft.state {
                    AttachmentState::Ready(file) => Some(PromptPart::uploaded(file)),
                    AttachmentState::Uploading | AttachmentState::Failed(_) => None,
                }),
        );
        parts
    }

    pub fn clear_sent(&mut self, session_id: &str) {
        if let Some(drafts) = self.by_session.get_mut(session_id) {
            drafts.retain(|draft| !matches!(draft.state, AttachmentState::Ready(_)));
        }
    }
}

impl Shell {
    pub(super) fn choose_attachments(&mut self, cx: &mut Context<Self>) {
        if self.model.active_session().is_none() {
            return;
        }
        let selection = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some(self.strings.native.attach_file.into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = selection.await else {
                return;
            };
            let _ = this.update(cx, |this, cx| this.add_attachment_paths(paths, cx));
        })
        .detach();
    }

    pub(super) fn add_attachment_paths(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let Some(session_id) = self
            .model
            .active_session()
            .map(|session| session.id.clone())
        else {
            return;
        };
        for path in paths.into_iter().filter(|path| path.is_file()) {
            let Some(name) = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
            else {
                continue;
            };
            let id = self.attachments.add_upload(&session_id, name);
            let task = cx.background_spawn({
                let client = client.clone();
                async move { client.upload_file(&path).map_err(|error| error.to_string()) }
            });
            let request_session_id = session_id.clone();
            cx.spawn(async move |this, cx| {
                let result = task.await;
                let _ = this.update(cx, |this, cx| {
                    this.attachments.finish(&request_session_id, id, result);
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    pub(super) fn remove_attachment(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(session_id) = self
            .model
            .active_session()
            .map(|session| session.id.clone())
        {
            self.attachments.remove(&session_id, id);
            cx.notify();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(id: &str) -> FileMeta {
        FileMeta {
            id: id.into(),
            name: "notes.pdf".into(),
            media_type: "application/pdf".into(),
            size: 12,
            created_at: "2026-07-18T08:00:00.000Z".into(),
            expires_at: None,
        }
    }

    #[test]
    fn drafts_are_isolated_by_session_and_only_ready_files_are_submitted() {
        let mut attachments = Attachments::default();
        let first = attachments.add_upload("one", "notes.pdf".into());
        attachments.add_upload("two", "other.pdf".into());
        attachments.finish("one", first, Ok(file("f_01")));

        assert_eq!(attachments.for_session("one").len(), 1);
        assert_eq!(attachments.for_session("two").len(), 1);
        assert_eq!(attachments.prompt_parts("one", "review").len(), 2);
        assert!(attachments.prompt_parts("two", "").is_empty());

        attachments.clear_sent("one");
        assert!(attachments.for_session("one").is_empty());
        assert_eq!(attachments.for_session("two").len(), 1);
    }
}
