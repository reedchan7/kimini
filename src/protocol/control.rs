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
}
