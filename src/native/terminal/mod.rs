mod buffer;
mod local;

use std::collections::{HashMap, HashSet};

use crate::protocol::{Terminal, TerminalExit, TerminalOutput, TerminalStatus};

use buffer::TerminalBuffer;
pub(super) use local::{LocalTerminalEvent, LocalTerminalHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalOrigin {
    Daemon,
    Local,
}

pub(super) struct TerminalTab {
    pub terminal: Terminal,
    pub last_seq: u64,
    buffer: TerminalBuffer,
    origin: TerminalOrigin,
}

impl TerminalTab {
    fn new(terminal: Terminal, origin: TerminalOrigin) -> Self {
        let buffer = TerminalBuffer::new(terminal.cols, terminal.rows);
        Self {
            terminal,
            last_seq: 0,
            buffer,
            origin,
        }
    }

    pub fn output(&self) -> String {
        self.buffer.text()
    }

    pub fn is_local(&self) -> bool {
        self.origin == TerminalOrigin::Local
    }
}

#[derive(Default)]
struct SessionTerminals {
    tabs: Vec<TerminalTab>,
    active_id: Option<String>,
    error: Option<String>,
    notice: Option<String>,
}

#[derive(Default)]
pub(super) struct Terminals {
    sessions: HashMap<String, SessionTerminals>,
    loading: HashSet<String>,
}

impl Terminals {
    pub fn begin_load(&mut self, session_id: &str) -> bool {
        if !self.loading.insert(session_id.into()) {
            return false;
        }
        self.sessions.entry(session_id.into()).or_default().error = None;
        true
    }

    pub fn install(&mut self, session_id: String, terminals: Vec<Terminal>) {
        self.loading.remove(&session_id);
        let state = self.sessions.entry(session_id).or_default();
        let mut local_tabs = Vec::new();
        let mut old = HashMap::new();
        for tab in std::mem::take(&mut state.tabs) {
            if tab.origin == TerminalOrigin::Local {
                local_tabs.push(tab);
            } else {
                old.insert(tab.terminal.id.clone(), tab);
            }
        }
        let mut daemon_tabs = terminals
            .into_iter()
            .map(|terminal| {
                if let Some(mut tab) = old.remove(&terminal.id) {
                    tab.buffer.resize(terminal.cols, terminal.rows);
                    tab.terminal = terminal;
                    tab
                } else {
                    TerminalTab::new(terminal, TerminalOrigin::Daemon)
                }
            })
            .collect::<Vec<_>>();
        if !daemon_tabs.is_empty() {
            state.notice = None;
        }
        local_tabs.append(&mut daemon_tabs);
        state.tabs = local_tabs;
        if !state
            .active_id
            .as_ref()
            .is_some_and(|id| state.tabs.iter().any(|tab| tab.terminal.id == *id))
        {
            state.active_id = state
                .tabs
                .iter()
                .find(|tab| tab.terminal.status == TerminalStatus::Running)
                .or_else(|| state.tabs.first())
                .map(|tab| tab.terminal.id.clone());
        }
        state.error = None;
    }

    pub fn add_daemon(&mut self, session_id: String, terminal: Terminal) {
        self.loading.remove(&session_id);
        let state = self.sessions.entry(session_id).or_default();
        let id = terminal.id.clone();
        state
            .tabs
            .push(TerminalTab::new(terminal, TerminalOrigin::Daemon));
        state.active_id = Some(id);
        state.error = None;
        state.notice = None;
    }

    pub fn add_local(&mut self, session_id: String, terminal: Terminal) {
        self.loading.remove(&session_id);
        let state = self.sessions.entry(session_id).or_default();
        let id = terminal.id.clone();
        state
            .tabs
            .push(TerminalTab::new(terminal, TerminalOrigin::Local));
        state.active_id = Some(id);
        state.error = None;
    }

    pub fn fail(&mut self, session_id: &str, error: String) {
        self.loading.remove(session_id);
        self.sessions.entry(session_id.into()).or_default().error = Some(error);
    }

    pub fn is_loading(&self, session_id: &str) -> bool {
        self.loading.contains(session_id)
    }

    pub fn error(&self, session_id: &str) -> Option<&str> {
        self.sessions
            .get(session_id)
            .and_then(|state| state.error.as_deref())
    }

    pub fn notice(&self, session_id: &str) -> Option<&str> {
        self.sessions
            .get(session_id)
            .and_then(|state| state.notice.as_deref())
    }

    pub fn set_notice(&mut self, session_id: &str, notice: String) {
        self.sessions.entry(session_id.into()).or_default().notice = Some(notice);
    }

    pub fn tabs(&self, session_id: &str) -> &[TerminalTab] {
        self.sessions
            .get(session_id)
            .map(|state| state.tabs.as_slice())
            .unwrap_or_default()
    }

    pub fn active(&self, session_id: &str) -> Option<&TerminalTab> {
        let state = self.sessions.get(session_id)?;
        let id = state.active_id.as_deref()?;
        state.tabs.iter().find(|tab| tab.terminal.id == id)
    }

    pub fn has_running(&self, session_id: &str) -> bool {
        self.tabs(session_id)
            .iter()
            .any(|tab| tab.terminal.status == TerminalStatus::Running)
    }

    pub fn is_local(&self, session_id: &str, terminal_id: &str) -> bool {
        self.tabs(session_id)
            .iter()
            .any(|tab| tab.terminal.id == terminal_id && tab.is_local())
    }

    pub fn select(&mut self, session_id: &str, terminal_id: &str) -> bool {
        let Some(state) = self.sessions.get_mut(session_id) else {
            return false;
        };
        if !state.tabs.iter().any(|tab| tab.terminal.id == terminal_id) {
            return false;
        }
        state.active_id = Some(terminal_id.into());
        true
    }

    pub fn apply_output(&mut self, output: TerminalOutput) -> bool {
        let Some(state) = self.sessions.get_mut(&output.session_id) else {
            return false;
        };
        let Some(tab) = state
            .tabs
            .iter_mut()
            .find(|tab| tab.terminal.id == output.terminal_id)
        else {
            return false;
        };
        if output.seq <= tab.last_seq {
            return false;
        }
        tab.last_seq = output.seq;
        tab.buffer.advance(output.data.as_bytes());
        true
    }

    pub fn apply_local_output(&mut self, session_id: &str, terminal_id: &str, data: &[u8]) -> bool {
        let Some(state) = self.sessions.get_mut(session_id) else {
            return false;
        };
        let Some(tab) = state
            .tabs
            .iter_mut()
            .find(|tab| tab.terminal.id == terminal_id && tab.is_local())
        else {
            return false;
        };
        tab.last_seq = tab.last_seq.wrapping_add(1);
        tab.buffer.advance(data);
        true
    }

    pub fn apply_exit(&mut self, exit: TerminalExit) -> bool {
        let Some(state) = self.sessions.get_mut(&exit.session_id) else {
            return false;
        };
        let Some(tab) = state
            .tabs
            .iter_mut()
            .find(|tab| tab.terminal.id == exit.terminal_id)
        else {
            return false;
        };
        tab.terminal.status = TerminalStatus::Exited;
        tab.terminal.exit_code = exit.exit_code;
        true
    }

    pub fn remove(&mut self, session_id: &str, terminal_id: &str) {
        let Some(state) = self.sessions.get_mut(session_id) else {
            return;
        };
        state.tabs.retain(|tab| tab.terminal.id != terminal_id);
        if state.active_id.as_deref() == Some(terminal_id) {
            state.active_id = state.tabs.first().map(|tab| tab.terminal.id.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terminal(id: &str) -> Terminal {
        serde_json::from_value(serde_json::json!({
            "id": id, "session_id": "session", "cwd": "/workspace",
            "shell": "/bin/zsh", "cols": 80, "rows": 24, "status": "running",
            "created_at": "2026-07-18T08:00:00.000Z"
        }))
        .unwrap()
    }

    #[test]
    fn refresh_preserves_output_and_active_terminal_by_identity() {
        let mut terminals = Terminals::default();
        terminals.install("session".into(), vec![terminal("one"), terminal("two")]);
        terminals.select("session", "two");
        terminals.apply_output(TerminalOutput {
            session_id: "session".into(),
            terminal_id: "two".into(),
            seq: 1,
            data: "hello".into(),
        });
        terminals.install("session".into(), vec![terminal("two")]);

        let active = terminals.active("session").unwrap();
        assert_eq!(active.terminal.id, "two");
        assert_eq!(active.output(), "hello");
    }

    #[test]
    fn terminal_output_deduplicates_replayed_sequences() {
        let mut terminals = Terminals::default();
        terminals.install("session".into(), vec![terminal("one")]);
        let output = TerminalOutput {
            session_id: "session".into(),
            terminal_id: "one".into(),
            seq: 1,
            data: "x".into(),
        };
        assert!(terminals.apply_output(output.clone()));
        assert!(!terminals.apply_output(output));
        assert_eq!(terminals.active("session").unwrap().output(), "x");
    }

    #[test]
    fn daemon_refresh_preserves_a_running_local_fallback() {
        let mut terminals = Terminals::default();
        terminals.add_local("session".into(), terminal("local"));

        terminals.install("session".into(), Vec::new());

        let active = terminals.active("session").unwrap();
        assert_eq!(active.terminal.id, "local");
        assert!(active.is_local());
        assert!(terminals.has_running("session"));
    }

    #[test]
    fn local_output_accepts_raw_utf8_across_chunks() {
        let mut terminals = Terminals::default();
        terminals.add_local("session".into(), terminal("local"));
        assert!(terminals.apply_local_output("session", "local", &[0xe4, 0xbd]));
        assert!(terminals.apply_local_output("session", "local", &[0xa0, 0xe5, 0xa5, 0xbd]));
        assert_eq!(terminals.active("session").unwrap().output(), "你好");
    }
}
