use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCursor {
    pub seq: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epoch: Option<String>,
}

impl SessionCursor {
    pub const fn new(seq: u64, epoch: Option<String>) -> Self {
        Self { seq, epoch }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientControl {
    ClientHello {
        id: String,
        payload: ClientHelloPayload,
    },
    Subscribe {
        id: String,
        payload: SubscribePayload,
    },
    Unsubscribe {
        id: String,
        payload: SessionIdsPayload,
    },
    Abort {
        id: String,
        payload: AbortPayload,
    },
    TerminalAttach {
        id: String,
        payload: TerminalAttachPayload,
    },
    TerminalDetach {
        id: String,
        payload: TerminalTargetPayload,
    },
    TerminalInput {
        id: String,
        payload: TerminalInputPayload,
    },
    TerminalResize {
        id: String,
        payload: TerminalResizePayload,
    },
    TerminalClose {
        id: String,
        payload: TerminalTargetPayload,
    },
    Pong {
        payload: PongPayload,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClientHelloPayload {
    pub client_id: String,
    pub subscriptions: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub cursors: BTreeMap<String, SessionCursor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubscribePayload {
    pub session_ids: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub cursors: BTreeMap<String, SessionCursor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionIdsPayload {
    pub session_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AbortPayload {
    pub session_id: String,
    pub prompt_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TerminalAttachPayload {
    pub session_id: String,
    pub terminal_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_seq: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TerminalTargetPayload {
    pub session_id: String,
    pub terminal_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TerminalInputPayload {
    pub session_id: String,
    pub terminal_id: String,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TerminalResizePayload {
    pub session_id: String,
    pub terminal_id: String,
    pub cols: usize,
    pub rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PongPayload {
    pub nonce: String,
}

impl ClientControl {
    pub fn hello(
        id: impl Into<String>,
        client_id: impl Into<String>,
        cursors: BTreeMap<String, SessionCursor>,
    ) -> Self {
        let subscriptions = cursors.keys().cloned().collect();
        Self::ClientHello {
            id: id.into(),
            payload: ClientHelloPayload {
                client_id: client_id.into(),
                subscriptions,
                cursors,
            },
        }
    }

    pub fn subscribe(
        id: impl Into<String>,
        session_id: impl Into<String>,
        cursor: SessionCursor,
    ) -> Self {
        let session_id = session_id.into();
        let cursors = BTreeMap::from([(session_id.clone(), cursor)]);
        Self::Subscribe {
            id: id.into(),
            payload: SubscribePayload {
                session_ids: vec![session_id],
                cursors,
            },
        }
    }

    pub fn pong(nonce: impl Into<String>) -> Self {
        Self::Pong {
            payload: PongPayload {
                nonce: nonce.into(),
            },
        }
    }

    pub fn terminal_attach(
        id: impl Into<String>,
        session_id: impl Into<String>,
        terminal_id: impl Into<String>,
        since_seq: Option<u64>,
    ) -> Self {
        Self::TerminalAttach {
            id: id.into(),
            payload: TerminalAttachPayload {
                session_id: session_id.into(),
                terminal_id: terminal_id.into(),
                since_seq,
            },
        }
    }

    pub fn terminal_detach(
        id: impl Into<String>,
        session_id: impl Into<String>,
        terminal_id: impl Into<String>,
    ) -> Self {
        Self::TerminalDetach {
            id: id.into(),
            payload: TerminalTargetPayload {
                session_id: session_id.into(),
                terminal_id: terminal_id.into(),
            },
        }
    }

    pub fn terminal_input(
        id: impl Into<String>,
        session_id: impl Into<String>,
        terminal_id: impl Into<String>,
        data: impl Into<String>,
    ) -> Self {
        Self::TerminalInput {
            id: id.into(),
            payload: TerminalInputPayload {
                session_id: session_id.into(),
                terminal_id: terminal_id.into(),
                data: data.into(),
            },
        }
    }

    pub fn terminal_resize(
        id: impl Into<String>,
        session_id: impl Into<String>,
        terminal_id: impl Into<String>,
        cols: usize,
        rows: usize,
    ) -> Self {
        Self::TerminalResize {
            id: id.into(),
            payload: TerminalResizePayload {
                session_id: session_id.into(),
                terminal_id: terminal_id.into(),
                cols,
                rows,
            },
        }
    }

    pub fn terminal_close(
        id: impl Into<String>,
        session_id: impl Into<String>,
        terminal_id: impl Into<String>,
    ) -> Self {
        Self::TerminalClose {
            id: id.into(),
            payload: TerminalTargetPayload {
                session_id: session_id.into(),
                terminal_id: terminal_id.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(control: ClientControl) -> serde_json::Value {
        serde_json::to_value(control).unwrap()
    }

    #[test]
    fn every_control_variant_keeps_its_protocol_payload() {
        let unsubscribe = encoded(ClientControl::Unsubscribe {
            id: "u1".into(),
            payload: SessionIdsPayload {
                session_ids: vec!["session".into()],
            },
        });
        let abort = encoded(ClientControl::Abort {
            id: "a1".into(),
            payload: AbortPayload {
                session_id: "session".into(),
                prompt_id: "prompt".into(),
            },
        });
        let detach = encoded(ClientControl::terminal_detach("d1", "session", "terminal"));
        let input = encoded(ClientControl::terminal_input(
            "i1", "session", "terminal", "pwd\n",
        ));
        let close = encoded(ClientControl::terminal_close("c1", "session", "terminal"));

        assert_eq!(unsubscribe["type"], "unsubscribe");
        assert_eq!(unsubscribe["payload"]["session_ids"][0], "session");
        assert_eq!(abort["type"], "abort");
        assert_eq!(abort["payload"]["prompt_id"], "prompt");
        assert_eq!(detach["type"], "terminal_detach");
        assert_eq!(input["payload"]["data"], "pwd\n");
        assert_eq!(close["type"], "terminal_close");
    }

    #[test]
    fn optional_resume_fields_are_omitted_when_no_cursor_exists() {
        let hello = encoded(ClientControl::hello("h1", "native", BTreeMap::new()));
        let attach = encoded(ClientControl::terminal_attach(
            "t1", "session", "terminal", None,
        ));

        assert!(hello["payload"].get("cursors").is_none());
        assert!(attach["payload"].get("since_seq").is_none());
    }
}
