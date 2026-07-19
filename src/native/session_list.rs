use std::collections::HashSet;
use std::path::Path;

use chrono::{DateTime, Utc};
use gpui::{ListAlignment, ListState, px};

use crate::protocol::Session;

const INITIAL_VISIBLE_SESSIONS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SidebarSession {
    pub id: String,
    pub title: String,
    pub cwd: String,
    pub updated_at: String,
    pub busy: bool,
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SessionListRow {
    Workspace {
        key: String,
        label: String,
        collapsed: bool,
    },
    Session(SidebarSession),
    ShowMore {
        workspace_key: String,
        remaining: usize,
        expanded: bool,
    },
}

impl SessionListRow {
    fn key(&self) -> String {
        match self {
            Self::Workspace { key, .. } => format!("workspace:{key}"),
            Self::Session(session) => format!("session:{}", session.id),
            Self::ShowMore { workspace_key, .. } => format!("more:{workspace_key}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SessionGroup {
    label: String,
    sessions: Vec<SidebarSession>,
    key: String,
}

pub(super) struct SessionList {
    pub list: ListState,
    rows: Vec<SessionListRow>,
    session_count: usize,
    expanded_workspaces: HashSet<String>,
    collapsed_workspaces: HashSet<String>,
}

impl Default for SessionList {
    fn default() -> Self {
        Self {
            list: ListState::new(0, ListAlignment::Top, px(40.0)),
            rows: Vec::new(),
            session_count: 0,
            expanded_workspaces: HashSet::new(),
            collapsed_workspaces: HashSet::new(),
        }
    }
}

impl SessionList {
    pub fn sync(&mut self, sessions: &[Session], query: &str, active_session_id: Option<&str>) {
        let next = flattened_rows(
            sessions,
            query,
            active_session_id,
            &self.expanded_workspaces,
            &self.collapsed_workspaces,
        );
        let common = self
            .rows
            .iter()
            .zip(&next)
            .take_while(|(left, right)| left.key() == right.key())
            .count();
        let changed = self
            .rows
            .iter()
            .zip(&next)
            .take(common)
            .enumerate()
            .filter_map(|(index, (left, right))| (left != right).then_some(index))
            .collect::<Vec<_>>();
        let old_len = self.rows.len();
        self.rows = next;
        self.session_count = self
            .rows
            .iter()
            .filter(|row| matches!(row, SessionListRow::Session(_)))
            .count();
        if old_len != self.rows.len() || common < old_len.min(self.rows.len()) {
            self.list
                .splice(common..old_len, self.rows.len().saturating_sub(common));
        }
        for index in changed {
            self.list.remeasure_items(index..index + 1);
        }
    }

    pub fn row(&self, index: usize) -> Option<&SessionListRow> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.session_count == 0
    }

    pub fn session_count(&self) -> usize {
        self.session_count
    }

    pub fn toggle_workspace(&mut self, key: &str) {
        if !self.collapsed_workspaces.insert(key.to_owned()) {
            self.collapsed_workspaces.remove(key);
        }
    }

    pub fn toggle_expanded(&mut self, key: &str) {
        if !self.expanded_workspaces.insert(key.to_owned()) {
            self.expanded_workspaces.remove(key);
        }
    }
}

pub(super) fn workspace_label(cwd: &str) -> String {
    Path::new(cwd)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(cwd)
        .to_owned()
}

pub(super) fn display_title(title: &str) -> String {
    let title = title.trim();
    let title = title.strip_prefix("```").unwrap_or(title).trim();
    title
        .lines()
        .next()
        .unwrap_or(title)
        .trim_matches('`')
        .trim()
        .to_owned()
}

pub(super) fn relative_time(updated_at: &str, just_now: &str) -> String {
    relative_time_at(updated_at, Utc::now(), just_now)
}

fn relative_time_at(updated_at: &str, now: DateTime<Utc>, just_now: &str) -> String {
    let Ok(updated_at) = DateTime::parse_from_rfc3339(updated_at) else {
        return String::new();
    };
    let seconds = now
        .signed_duration_since(updated_at.with_timezone(&Utc))
        .num_seconds()
        .max(0) as f64;
    if seconds < 60.0 {
        return just_now.to_owned();
    }
    let hours = seconds / 3_600.0;
    if hours < 1.0 {
        return format!("{}m", (seconds / 60.0).round() as u64);
    }
    if hours < 24.0 {
        return format!("{}h", hours.round() as u64);
    }
    let days = seconds / 86_400.0;
    if days < 7.0 {
        return format!("{}d", days.round() as u64);
    }
    if days < 30.0 {
        return format!("{}w", (days / 7.0).round() as u64);
    }
    if days < 365.0 {
        return format!("{}mo", (days / 30.0).round() as u64);
    }
    format!("{}y", (days / 365.0).round() as u64)
}

fn flattened_rows(
    sessions: &[Session],
    query: &str,
    active_session_id: Option<&str>,
    expanded_workspaces: &HashSet<String>,
    collapsed_workspaces: &HashSet<String>,
) -> Vec<SessionListRow> {
    let searching = !query.is_empty();
    let mut rows = Vec::new();
    let mut position = 0;

    for group in grouped_sessions(sessions, query) {
        let collapsed = !searching && collapsed_workspaces.contains(&group.key);
        rows.push(SessionListRow::Workspace {
            key: group.key.clone(),
            label: group.label,
            collapsed,
        });
        if collapsed {
            continue;
        }

        let expanded = searching || expanded_workspaces.contains(&group.key);
        let total = group.sessions.len();
        let active_tail = (!expanded).then(|| {
            group
                .sessions
                .iter()
                .skip(INITIAL_VISIBLE_SESSIONS)
                .find(|session| Some(session.id.as_str()) == active_session_id)
                .cloned()
        });
        let mut visible = if expanded {
            group.sessions
        } else {
            group
                .sessions
                .into_iter()
                .take(INITIAL_VISIBLE_SESSIONS)
                .collect()
        };
        if let Some(active) = active_tail.flatten() {
            visible.push(active);
        }
        for mut session in visible {
            session.position = position;
            position += 1;
            rows.push(SessionListRow::Session(session));
        }
        if !searching && total > INITIAL_VISIBLE_SESSIONS {
            rows.push(SessionListRow::ShowMore {
                workspace_key: group.key,
                remaining: total - INITIAL_VISIBLE_SESSIONS,
                expanded,
            });
        }
    }
    rows
}

fn grouped_sessions(sessions: &[Session], query: &str) -> Vec<SessionGroup> {
    let mut groups: Vec<SessionGroup> = Vec::new();
    for session in sessions
        .iter()
        .filter(|session| session_matches(query, &session.title, &session.metadata.cwd))
    {
        let key = if session.workspace_id.is_empty() {
            session.metadata.cwd.clone()
        } else {
            session.workspace_id.clone()
        };
        let group_index = groups
            .iter()
            .position(|group| group.key == key)
            .unwrap_or_else(|| {
                groups.push(SessionGroup {
                    key,
                    label: workspace_label(&session.metadata.cwd),
                    sessions: Vec::new(),
                });
                groups.len() - 1
            });
        groups[group_index].sessions.push(SidebarSession {
            id: session.id.clone(),
            title: display_title(&session.title),
            cwd: session.metadata.cwd.clone(),
            updated_at: session.updated_at.clone(),
            busy: session.busy,
            position: 0,
        });
    }
    groups
}

fn session_matches(query: &str, title: &str, cwd: &str) -> bool {
    query.is_empty() || title.to_lowercase().contains(query) || cwd.to_lowercase().contains(query)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(id: &str, workspace_id: &str, cwd: &str, title: &str) -> Session {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "workspace_id": workspace_id,
            "title": title,
            "created_at": "2026-07-18T08:00:00.000Z",
            "updated_at": "2026-07-18T08:00:00.000Z",
            "busy": false,
            "archived": false,
            "metadata": { "cwd": cwd },
            "agent_config": { "model": "k3" },
            "usage": {
                "input_tokens": 0,
                "output_tokens": 0,
                "cache_read_tokens": 0,
                "cache_creation_tokens": 0,
                "total_cost_usd": 0,
                "context_tokens": 0,
                "context_limit": 100000,
                "turn_count": 0
            },
            "permission_rules": [],
            "message_count": 0,
            "last_seq": 0
        }))
        .unwrap()
    }

    fn rows(sessions: &[Session], active_session_id: Option<&str>) -> Vec<SessionListRow> {
        flattened_rows(
            sessions,
            "",
            active_session_id,
            &HashSet::new(),
            &HashSet::new(),
        )
    }

    #[test]
    fn search_matches_titles_and_workspace_paths_case_insensitively() {
        assert!(session_matches("timezone", "Fix TimeZone", "/tmp/pollo"));
        assert!(session_matches("pollo", "Other", "/tmp/pollo"));
        assert!(!session_matches("kimini", "Other", "/tmp/pollo"));
    }

    #[test]
    fn display_helpers_keep_navigation_labels_compact() {
        assert_eq!(
            workspace_label("/Users/reedchan/Workspaces/github/kimini"),
            "kimini"
        );
        assert_eq!(
            display_title("``` Request changes\nmore"),
            "Request changes"
        );
    }

    #[test]
    fn relative_times_follow_the_web_sidebar_buckets() {
        let now = DateTime::parse_from_rfc3339("2026-07-19T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(relative_time_at("2026-07-19T11:59:30Z", now, "now"), "now");
        assert_eq!(relative_time_at("2026-07-19T11:30:00Z", now, "now"), "30m");
        assert_eq!(relative_time_at("2026-07-18T17:00:00Z", now, "now"), "19h");
        assert_eq!(relative_time_at("2026-07-15T12:00:00Z", now, "now"), "4d");
        assert_eq!(relative_time_at("2026-07-05T12:00:00Z", now, "now"), "2w");
    }

    #[test]
    fn rows_keep_workspace_order_and_global_session_positions() {
        let sessions = vec![
            session("a", "workspace-a", "/tmp/alpha", "Newest alpha"),
            session("b", "workspace-b", "/tmp/beta", "Beta"),
            session("c", "workspace-a", "/tmp/alpha", "Older alpha"),
        ];
        let rows = rows(&sessions, None);

        assert_eq!(rows.len(), 5);
        assert!(matches!(
            &rows[0],
            SessionListRow::Workspace { label, .. } if label == "alpha"
        ));
        assert!(matches!(
            &rows[2],
            SessionListRow::Session(session) if session.position == 1
        ));
        assert!(matches!(
            &rows[4],
            SessionListRow::Session(session) if session.position == 2
        ));
    }

    #[test]
    fn sync_updates_the_virtual_list_count_after_search_and_pagination() {
        let sessions = vec![
            session("a", "workspace-a", "/tmp/alpha", "First"),
            session("b", "workspace-b", "/tmp/beta", "Second"),
        ];
        let mut list = SessionList::default();
        list.sync(&sessions[..1], "", None);
        assert_eq!(list.list.item_count(), 2);

        list.sync(&sessions, "", None);
        assert_eq!(list.list.item_count(), 4);
        assert_eq!(list.session_count(), 2);

        list.sync(&sessions, "beta", None);
        assert_eq!(list.list.item_count(), 2);
        assert_eq!(list.session_count(), 1);
    }

    #[test]
    fn workspace_rows_match_the_web_first_page_and_keep_a_deep_link_visible() {
        let sessions = (0..7)
            .map(|index| {
                session(
                    &format!("session-{index}"),
                    "workspace-a",
                    "/tmp/alpha",
                    &format!("Session {index}"),
                )
            })
            .collect::<Vec<_>>();

        let initial = rows(&sessions, None);
        assert_eq!(initial.len(), 7);
        assert!(matches!(
            initial.last(),
            Some(SessionListRow::ShowMore {
                remaining: 2,
                expanded: false,
                ..
            })
        ));

        let with_active_tail = rows(&sessions, Some("session-6"));
        assert_eq!(with_active_tail.len(), 8);
        assert!(with_active_tail.iter().any(
            |row| matches!(row, SessionListRow::Session(session) if session.id == "session-6")
        ));
    }

    #[test]
    fn workspace_controls_expand_collapse_and_leave_search_untrimmed() {
        let sessions = (0..7)
            .map(|index| {
                session(
                    &format!("session-{index}"),
                    "workspace-a",
                    "/tmp/alpha",
                    &format!("Session {index}"),
                )
            })
            .collect::<Vec<_>>();
        let mut list = SessionList::default();

        list.toggle_expanded("workspace-a");
        list.sync(&sessions, "", None);
        assert_eq!(list.session_count(), 7);
        assert!(matches!(
            list.rows.last(),
            Some(SessionListRow::ShowMore { expanded: true, .. })
        ));

        list.toggle_workspace("workspace-a");
        list.sync(&sessions, "", None);
        assert_eq!(list.rows.len(), 1);
        assert_eq!(list.session_count(), 0);

        list.sync(&sessions, "session", None);
        assert_eq!(list.rows.len(), 8);
        assert_eq!(list.session_count(), 7);
    }
}
