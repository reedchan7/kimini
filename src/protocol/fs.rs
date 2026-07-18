use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsKind {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsGitStatus {
    Clean,
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Ignored,
    Conflicted,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct FsEntry {
    pub path: String,
    pub name: String,
    pub kind: FsKind,
    #[serde(default)]
    pub size: Option<u64>,
    pub modified_at: String,
    #[serde(default)]
    pub mime: Option<String>,
    #[serde(default)]
    pub language_id: Option<String>,
    #[serde(default)]
    pub is_binary: Option<bool>,
    #[serde(default)]
    pub git_status: Option<FsGitStatus>,
    #[serde(default)]
    pub child_count: Option<usize>,
}

impl FsEntry {
    pub fn is_directory(&self) -> bool {
        self.kind == FsKind::Directory
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
pub struct FsList {
    #[serde(default)]
    pub items: Vec<FsEntry>,
    #[serde(default)]
    pub children_by_path: HashMap<String, Vec<FsEntry>>,
    #[serde(default)]
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct FsPreview {
    pub path: String,
    pub content: String,
    pub encoding: String,
    pub size: u64,
    pub truncated: bool,
    pub mime: String,
    #[serde(default)]
    pub language_id: Option<String>,
    #[serde(default)]
    pub line_count: Option<usize>,
    pub is_binary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct FsDiff {
    pub path: String,
    pub diff: String,
    #[serde(default)]
    pub truncated: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
pub struct FsGitStatusSummary {
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub ahead: usize,
    #[serde(default)]
    pub behind: usize,
    #[serde(default)]
    pub entries: HashMap<String, FsGitStatus>,
    #[serde(default)]
    pub additions: usize,
    #[serde(default)]
    pub deletions: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct FsSearchHit {
    pub path: String,
    pub name: String,
    pub kind: FsKind,
    pub score: f64,
    #[serde(default)]
    pub match_positions: Vec<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize)]
pub struct FsSearchResults {
    #[serde(default)]
    pub items: Vec<FsSearchHit>,
    #[serde(default)]
    pub truncated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_forward_compatible_file_and_git_payloads() {
        let list: FsList = serde_json::from_value(serde_json::json!({
            "items": [{
                "path": "src", "name": "src", "kind": "directory",
                "modified_at": "2026-07-18T08:00:00.000Z", "child_count": 12,
                "future_field": true
            }],
            "truncated": false
        }))
        .unwrap();
        assert!(list.items[0].is_directory());
        assert_eq!(list.items[0].child_count, Some(12));

        let status: FsGitStatusSummary = serde_json::from_value(serde_json::json!({
            "branch": "main", "ahead": 1, "behind": 2,
            "entries": {"src/main.rs": "modified"},
            "additions": 7, "deletions": 3, "pullRequest": null
        }))
        .unwrap();
        assert_eq!(status.entries["src/main.rs"], FsGitStatus::Modified);
    }
}
