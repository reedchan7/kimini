use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TerminalStatus {
    Running,
    Exited,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Terminal {
    pub id: String,
    pub session_id: String,
    pub cwd: String,
    pub shell: String,
    pub cols: usize,
    pub rows: usize,
    pub status: TerminalStatus,
    pub created_at: String,
    #[serde(default)]
    pub exited_at: Option<String>,
    #[serde(default)]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct TerminalList {
    pub items: Vec<Terminal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateTerminal {
    pub cols: usize,
    pub rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalOutput {
    pub session_id: String,
    pub terminal_id: String,
    pub seq: u64,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalExit {
    pub session_id: String,
    pub terminal_id: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TerminalOutputFrame {
    pub seq: u64,
    pub session_id: String,
    pub terminal_id: String,
    pub payload: TerminalOutputPayload,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TerminalOutputPayload {
    pub data: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TerminalExitFrame {
    pub session_id: String,
    pub terminal_id: String,
    pub payload: TerminalExitPayload,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TerminalExitPayload {
    #[serde(default)]
    pub exit_code: Option<i32>,
}

impl From<TerminalOutputFrame> for TerminalOutput {
    fn from(frame: TerminalOutputFrame) -> Self {
        Self {
            session_id: frame.session_id,
            terminal_id: frame.terminal_id,
            seq: frame.seq,
            data: frame.payload.data,
        }
    }
}

impl From<TerminalExitFrame> for TerminalExit {
    fn from(frame: TerminalExitFrame) -> Self {
        Self {
            session_id: frame.session_id,
            terminal_id: frame.terminal_id,
            exit_code: frame.payload.exit_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_metadata_accepts_running_and_exited_sessions() {
        let terminal = serde_json::from_value::<Terminal>(serde_json::json!({
            "id": "term_01", "session_id": "sess_01", "cwd": "/workspace",
            "shell": "/bin/zsh", "cols": 120, "rows": 36, "status": "exited",
            "created_at": "2026-07-18T08:00:00.000Z", "exit_code": 0
        }))
        .unwrap();
        assert_eq!(terminal.status, TerminalStatus::Exited);
        assert_eq!(terminal.exit_code, Some(0));
    }

    #[test]
    fn terminal_wire_frames_project_only_the_native_fields() {
        let output: TerminalOutputFrame = serde_json::from_value(serde_json::json!({
            "seq": 7, "session_id": "sess_01", "terminal_id": "term_01",
            "payload": { "data": "hello\n" }
        }))
        .unwrap();
        let exit: TerminalExitFrame = serde_json::from_value(serde_json::json!({
            "session_id": "sess_01", "terminal_id": "term_01",
            "payload": { "exit_code": 2 }
        }))
        .unwrap();

        assert_eq!(
            TerminalOutput::from(output),
            TerminalOutput {
                session_id: "sess_01".into(),
                terminal_id: "term_01".into(),
                seq: 7,
                data: "hello\n".into(),
            }
        );
        assert_eq!(
            TerminalExit::from(exit),
            TerminalExit {
                session_id: "sess_01".into(),
                terminal_id: "term_01".into(),
                exit_code: Some(2),
            }
        );
    }

    #[test]
    fn terminal_collections_and_create_dimensions_match_rest_shapes() {
        let empty =
            serde_json::from_value::<TerminalList>(serde_json::json!({ "items": [] })).unwrap();
        assert_eq!(empty, TerminalList::default());
        let dimensions = serde_json::to_value(CreateTerminal {
            cols: 132,
            rows: 42,
        })
        .unwrap();

        assert_eq!(dimensions, serde_json::json!({ "cols": 132, "rows": 42 }));
    }
}
