use gpui::{AppContext, Context};

use crate::api::ApiError;

use super::super::app::{Shell, UtilityPanel};
use super::super::files::PreviewMode;

impl Shell {
    pub(in crate::native) fn set_file_preview_mode(
        &mut self,
        mode: PreviewMode,
        cx: &mut Context<Self>,
    ) {
        self.files.set_preview_mode(mode);
        cx.notify();
    }

    pub(in crate::native) fn toggle_file_panel(&mut self, cx: &mut Context<Self>) {
        self.utility_panel = if self.utility_panel == Some(UtilityPanel::Files) {
            None
        } else {
            Some(UtilityPanel::Files)
        };
        if self.utility_panel == Some(UtilityPanel::Files) {
            self.refresh_workspace_files(cx);
        }
        cx.notify();
    }

    pub(in crate::native) fn refresh_workspace_files(&mut self, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.files.ensure_session(&session_id);
        if self.files.loading {
            return;
        }
        self.files.loading = true;
        self.files.error = None;
        let generation = self.files.generation;
        let request_session = session_id.clone();
        let task = cx.background_spawn(async move {
            let listing = client.list_files(&session_id, ".")?;
            let git = client.workspace_git_status(&session_id).ok();
            Ok::<_, ApiError>((listing, git))
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if generation != this.files.generation
                    || this.files.current_session() != Some(request_session.as_str())
                {
                    return;
                }
                match result {
                    Ok((listing, git)) => this.files.replace_root(listing, git),
                    Err(error) => {
                        this.files.loading = false;
                        this.files.error = Some(error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn activate_file_row(
        &mut self,
        path: String,
        directory: bool,
        cx: &mut Context<Self>,
    ) {
        if directory {
            if self.files.toggle_directory(&path) {
                self.load_workspace_directory(path, cx);
            } else {
                cx.notify();
            }
        } else {
            self.open_workspace_file(path, cx);
        }
    }

    fn load_workspace_directory(&mut self, path: String, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        let generation = self.files.generation;
        let request_session = session_id.clone();
        let request_path = path.clone();
        let task = cx.background_spawn(async move { client.list_files(&session_id, &path) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if generation != this.files.generation
                    || this.files.current_session() != Some(request_session.as_str())
                {
                    return;
                }
                match result {
                    Ok(listing) => this.files.replace_children(request_path, listing),
                    Err(error) => this.files.error = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn open_workspace_file(&mut self, path: String, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.files.preview_loading = true;
        self.files.error = None;
        self.files.preview_request = self.files.preview_request.wrapping_add(1);
        let request = self.files.preview_request;
        let generation = self.files.generation;
        let request_session = session_id.clone();
        let changed = self
            .files
            .git
            .as_ref()
            .is_some_and(|git| git.entries.contains_key(&path));
        let task = cx.background_spawn(async move {
            let file = client.read_workspace_file(&session_id, &path)?;
            let diff = changed
                .then(|| client.workspace_file_diff(&session_id, &path).ok())
                .flatten();
            Ok::<_, ApiError>((file, diff))
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if request != this.files.preview_request
                    || generation != this.files.generation
                    || this.files.current_session() != Some(request_session.as_str())
                {
                    return;
                }
                match result {
                    Ok((file, diff)) => this.files.set_preview(file, diff),
                    Err(error) => {
                        this.files.preview_loading = false;
                        this.files.error = Some(error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn search_workspace_files(&mut self, cx: &mut Context<Self>) {
        let query = self.file_search.read(cx).value().trim().to_owned();
        if query.is_empty() {
            self.files.clear_search();
            cx.notify();
            return;
        }
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        self.files.loading = true;
        self.files.error = None;
        self.files.search_request = self.files.search_request.wrapping_add(1);
        let request = self.files.search_request;
        let generation = self.files.generation;
        let request_session = session_id.clone();
        let task =
            cx.background_spawn(async move { client.search_workspace_files(&session_id, &query) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if request != this.files.search_request
                    || generation != this.files.generation
                    || this.files.current_session() != Some(request_session.as_str())
                {
                    return;
                }
                match result {
                    Ok(results) => this.files.set_search_results(results),
                    Err(error) => {
                        this.files.loading = false;
                        this.files.error = Some(error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}
