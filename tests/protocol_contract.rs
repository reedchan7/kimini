use std::collections::BTreeMap;

use kimini::protocol::{ClientControl, Message, SessionCursor, SessionSnapshot, WireEvent};

const SNAPSHOT: &str = r#"
{
  "as_of_seq": 7,
  "epoch": "ep_01",
  "session": {
    "id": "sess_01",
    "workspace_id": "ws_01",
    "title": "Native GUI",
    "created_at": "2026-07-18T08:00:00.000Z",
    "updated_at": "2026-07-18T08:01:00.000Z",
    "busy": true,
    "archived": false,
    "metadata": { "cwd": "/tmp/project" },
    "agent_config": { "model": "kimi-k2" },
    "usage": {
      "input_tokens": 10, "output_tokens": 20,
      "cache_read_tokens": 0, "cache_creation_tokens": 0,
      "total_cost_usd": 0, "context_tokens": 30,
      "context_limit": 131072, "turn_count": 1
    },
    "permission_rules": [], "message_count": 2, "last_seq": 7
  },
  "messages": {
    "items": [{
      "id": "msg_01", "session_id": "sess_01", "role": "user",
      "content": [{ "type": "text", "text": "hello" }],
      "created_at": "2026-07-18T08:00:00.000Z"
    }],
    "has_more": false
  },
  "in_flight_turn": {
    "turn_id": 1, "assistant_text": "work", "thinking_text": "",
    "running_tools": [], "current_prompt_id": "prompt_01"
  },
  "pending_approvals": [], "pending_questions": []
}
"#;

#[test]
fn decodes_atomic_session_snapshot_without_discarding_unknown_fields() {
    let snapshot: SessionSnapshot = serde_json::from_str(SNAPSHOT).unwrap();

    assert_eq!(
        snapshot.cursor(),
        SessionCursor::new(7, Some("ep_01".into()))
    );
    assert_eq!(snapshot.session.metadata.cwd, "/tmp/project");
    assert_eq!(snapshot.messages.items[0].plain_text(), "hello");
    assert_eq!(snapshot.in_flight_turn.unwrap().assistant_text, "work");
}

#[test]
fn serializes_subscribe_with_the_snapshot_cursor() {
    let control = ClientControl::subscribe(
        "req_01",
        "sess_01",
        SessionCursor::new(7, Some("ep_01".into())),
    );
    let value = serde_json::to_value(control).unwrap();

    assert_eq!(value["type"], "subscribe");
    assert_eq!(value["payload"]["session_ids"][0], "sess_01");
    assert_eq!(value["payload"]["cursors"]["sess_01"]["seq"], 7);
    assert_eq!(value["payload"]["cursors"]["sess_01"]["epoch"], "ep_01");
}

#[test]
fn accepts_future_events_as_typed_envelopes() {
    let event: WireEvent = serde_json::from_str(
        r#"{"type":"event.future.capability","seq":8,"session_id":"sess_01","timestamp":"2026-07-18T08:02:00.000Z","payload":{"enabled":true}}"#,
    )
    .unwrap();

    assert_eq!(event.kind, "event.future.capability");
    assert_eq!(event.seq, 8);
    assert_eq!(event.payload["enabled"], true);
}

#[test]
fn serializes_handshake_and_heartbeat_controls() {
    let cursors = BTreeMap::from([("sess_01".into(), SessionCursor::new(4, None))]);
    let hello =
        serde_json::to_value(ClientControl::hello("hello_01", "native_01", cursors)).unwrap();
    let pong = serde_json::to_value(ClientControl::pong("nonce_01")).unwrap();

    assert_eq!(hello["type"], "client_hello");
    assert_eq!(hello["payload"]["subscriptions"][0], "sess_01");
    assert_eq!(pong["type"], "pong");
    assert_eq!(pong["payload"]["nonce"], "nonce_01");
}

#[test]
fn plain_text_includes_readable_parts_and_ignores_tool_payloads() {
    let message: Message = serde_json::from_value(serde_json::json!({
      "id": "msg", "session_id": "sess", "role": "assistant",
      "created_at": "2026-07-18T08:00:00.000Z",
      "content": [
        { "type": "thinking", "thinking": "plan " },
        { "type": "tool_use", "tool_call_id": "tool", "tool_name": "Read", "input": {} },
        { "type": "text", "text": "answer" },
        { "type": "future", "payload": true }
      ]
    }))
    .unwrap();

    assert_eq!(message.plain_text(), "plan answer");
}
