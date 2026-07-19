use std::collections::{HashMap, HashSet};

use gpui::{ListAlignment, ListState, px};

use crate::protocol::{
    FsDiff, FsEntry, FsGitStatus, FsGitStatusSummary, FsKind, FsList, FsPreview, FsSearchResults,
};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct FilePreview {
    pub file: FsPreview,
    pub diff: Option<FsDiff>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum PreviewMode {
    #[default]
    Source,
    Diff,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FileRow {
    pub path: String,
    pub name: String,
    pub kind: FsKind,
    pub depth: usize,
    pub expanded: bool,
    pub git_status: Option<FsGitStatus>,
}

impl FileRow {
    pub fn is_directory(&self) -> bool {
        self.kind == FsKind::Directory
    }
}

#[derive(Debug)]
pub(super) struct WorkspaceFiles {
    pub list: ListState,
    pub rows: Vec<FileRow>,
    pub preview: Option<FilePreview>,
    pub preview_mode: PreviewMode,
    pub git: Option<FsGitStatusSummary>,
    pub loading: bool,
    pub preview_loading: bool,
    pub error: Option<String>,
    pub generation: u64,
    pub preview_request: u64,
    pub search_request: u64,
    session_id: Option<String>,
    roots: Vec<FsEntry>,
    children: HashMap<String, Vec<FsEntry>>,
    expanded: HashSet<String>,
    search_rows: Option<Vec<FileRow>>,
}

impl Default for WorkspaceFiles {
    fn default() -> Self {
        Self {
            list: ListState::new(0, ListAlignment::Top, px(30.0)),
            rows: Vec::new(),
            preview: None,
            preview_mode: PreviewMode::Source,
            git: None,
            loading: false,
            preview_loading: false,
            error: None,
            generation: 0,
            preview_request: 0,
            search_request: 0,
            session_id: None,
            roots: Vec::new(),
            children: HashMap::new(),
            expanded: HashSet::new(),
            search_rows: None,
        }
    }
}

impl WorkspaceFiles {
    pub fn ensure_session(&mut self, session_id: &str) -> bool {
        if self.session_id.as_deref() == Some(session_id) {
            return false;
        }
        self.session_id = Some(session_id.to_owned());
        self.roots.clear();
        self.children.clear();
        self.expanded.clear();
        self.search_rows = None;
        self.preview = None;
        self.preview_mode = PreviewMode::Source;
        self.git = None;
        self.error = None;
        self.loading = false;
        self.preview_loading = false;
        self.generation = self.generation.wrapping_add(1);
        self.preview_request = self.preview_request.wrapping_add(1);
        self.search_request = self.search_request.wrapping_add(1);
        self.rebuild();
        true
    }

    pub fn replace_root(&mut self, listing: FsList, git: Option<FsGitStatusSummary>) {
        self.roots = listing.items;
        self.children.extend(listing.children_by_path);
        self.git = git;
        self.loading = false;
        self.error = None;
        self.rebuild();
    }

    pub fn replace_git_status(&mut self, git: Option<FsGitStatusSummary>) {
        self.git = git;
        self.rebuild();
    }

    pub fn toggle_directory(&mut self, path: &str) -> bool {
        if !self.expanded.insert(path.to_owned()) {
            self.expanded.remove(path);
            self.rebuild();
            return false;
        }
        let needs_load = !self.children.contains_key(path);
        self.rebuild();
        needs_load
    }

    pub fn replace_children(&mut self, path: String, listing: FsList) {
        self.children.insert(path, listing.items);
        self.children.extend(listing.children_by_path);
        self.error = None;
        self.rebuild();
    }

    pub fn set_search_results(&mut self, results: FsSearchResults) {
        self.search_rows = Some(
            results
                .items
                .into_iter()
                .map(|hit| FileRow {
                    path: hit.path,
                    name: hit.name,
                    kind: hit.kind,
                    depth: 0,
                    expanded: false,
                    git_status: None,
                })
                .collect(),
        );
        self.loading = false;
        self.error = None;
        self.rebuild();
    }

    pub fn clear_search(&mut self) {
        if self.search_rows.take().is_some() {
            self.rebuild();
        }
    }

    pub fn set_preview(&mut self, file: FsPreview, diff: Option<FsDiff>) {
        self.preview_mode = if diff.as_ref().is_some_and(|diff| !diff.diff.is_empty()) {
            PreviewMode::Diff
        } else {
            PreviewMode::Source
        };
        self.preview = Some(FilePreview { file, diff });
        self.preview_loading = false;
        self.error = None;
    }

    pub fn set_preview_mode(&mut self, mode: PreviewMode) {
        if mode == PreviewMode::Source
            || self
                .preview
                .as_ref()
                .and_then(|preview| preview.diff.as_ref())
                .is_some()
        {
            self.preview_mode = mode;
        }
    }

    pub fn current_session(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    fn rebuild(&mut self) {
        let rows = if let Some(rows) = &self.search_rows {
            rows.clone()
        } else {
            let mut rows = Vec::new();
            append_rows(
                &self.roots,
                0,
                &self.children,
                &self.expanded,
                self.git.as_ref(),
                &mut rows,
            );
            rows
        };
        let old_len = self.rows.len();
        self.rows = rows;
        self.list.splice(0..old_len, self.rows.len());
    }
}

fn append_rows(
    entries: &[FsEntry],
    depth: usize,
    children: &HashMap<String, Vec<FsEntry>>,
    expanded: &HashSet<String>,
    git: Option<&FsGitStatusSummary>,
    rows: &mut Vec<FileRow>,
) {
    for entry in entries {
        let is_expanded = entry.is_directory() && expanded.contains(&entry.path);
        rows.push(FileRow {
            path: entry.path.clone(),
            name: entry.name.clone(),
            kind: entry.kind,
            depth,
            expanded: is_expanded,
            git_status: entry
                .git_status
                .or_else(|| git.and_then(|status| status.entries.get(&entry.path).copied())),
        });
        if is_expanded && let Some(child_entries) = children.get(&entry.path) {
            append_rows(child_entries, depth + 1, children, expanded, git, rows);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, kind: FsKind) -> FsEntry {
        FsEntry {
            path: path.into(),
            name: path.rsplit('/').next().unwrap_or(path).into(),
            kind,
            size: None,
            modified_at: "2026-07-18T08:00:00.000Z".into(),
            mime: None,
            language_id: None,
            is_binary: None,
            git_status: None,
            child_count: None,
        }
    }

    #[test]
    fn tree_expands_lazily_and_preserves_git_state() {
        let mut files = WorkspaceFiles::default();
        files.ensure_session("session");
        files.replace_root(
            FsList {
                items: vec![entry("src", FsKind::Directory)],
                ..Default::default()
            },
            Some(FsGitStatusSummary {
                entries: HashMap::from([("src/main.rs".into(), FsGitStatus::Modified)]),
                ..Default::default()
            }),
        );

        assert!(files.toggle_directory("src"));
        files.replace_children(
            "src".into(),
            FsList {
                items: vec![entry("src/main.rs", FsKind::File)],
                ..Default::default()
            },
        );

        assert_eq!(files.rows.len(), 2);
        assert_eq!(files.rows[1].depth, 1);
        assert_eq!(files.rows[1].git_status, Some(FsGitStatus::Modified));
        assert!(!files.toggle_directory("src"));
        assert_eq!(files.rows.len(), 1);
    }

    #[test]
    fn switching_sessions_clears_transient_file_state() {
        let mut files = WorkspaceFiles::default();
        assert!(files.ensure_session("one"));
        files.error = Some("old".into());
        files.preview_loading = true;

        assert!(files.ensure_session("two"));
        assert_eq!(files.current_session(), Some("two"));
        assert!(files.error.is_none());
        assert!(!files.preview_loading);
        assert!(!files.ensure_session("two"));
    }
}
