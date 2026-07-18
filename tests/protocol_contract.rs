use std::collections::BTreeMap;

use kimini::protocol::{
    ClientControl, MediaSource, Message, MessageContent, ModelCatalogItem, PromptQueue,
    PromptStatus, SessionCursor, SessionSnapshot, TaskKind, TaskStatus, WireEvent,
};

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
fn serializes_terminal_controls_with_replay_and_native_dimensions() {
    let attach = serde_json::to_value(ClientControl::terminal_attach(
        "term_1",
        "sess_01",
        "pty_01",
        Some(42),
    ))
    .unwrap();
    let resize = serde_json::to_value(ClientControl::terminal_resize(
        "term_2", "sess_01", "pty_01", 120, 36,
    ))
    .unwrap();

    assert_eq!(attach["type"], "terminal_attach");
    assert_eq!(attach["payload"]["since_seq"], 42);
    assert_eq!(resize["type"], "terminal_resize");
    assert_eq!(resize["payload"]["cols"], 120);
    assert_eq!(resize["payload"]["rows"], 36);
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

#[test]
fn decodes_uploaded_media_and_file_parts_without_falling_back_to_unknown() {
    let message: Message = serde_json::from_value(serde_json::json!({
      "id": "msg", "session_id": "sess", "role": "user",
      "created_at": "2026-07-18T08:00:00.000Z",
      "content": [
        { "type": "image", "source": { "kind": "file", "file_id": "f_image" } },
        { "type": "video", "source": { "kind": "url", "url": "https://example.test/video" } },
        { "type": "file", "file_id": "f_file", "name": "notes.pdf", "media_type": "application/pdf", "size": 42 }
      ]
    }))
    .unwrap();

    assert!(matches!(message.content[0], MessageContent::Image(_)));
    assert!(matches!(message.content[1], MessageContent::Video(_)));
    assert!(matches!(message.content[2], MessageContent::File(_)));
}

#[test]
fn media_references_and_model_labels_are_stable_for_native_presentation() {
    assert_eq!(
        MediaSource::Url {
            url: "https://example.test/image".into()
        }
        .display_reference(),
        "https://example.test/image"
    );
    assert_eq!(
        MediaSource::Base64 {
            media_type: "image/png".into(),
            data: "AA==".into()
        }
        .display_reference(),
        "image/png"
    );
    assert_eq!(
        MediaSource::File {
            file_id: "file_01".into()
        }
        .display_reference(),
        "file_01"
    );

    let model = |display_name| ModelCatalogItem {
        provider: "kimi".into(),
        model: "kimi-k2".into(),
        display_name,
        max_context_size: 131_072,
        capabilities: Vec::new(),
        support_efforts: Vec::new(),
        default_effort: None,
    };
    assert_eq!(model(Some("K2".into())).label(), "K2");
    assert_eq!(model(None).label(), "kimi-k2");
}

#[test]
fn decodes_the_authoritative_prompt_queue_shape() {
    let queue: PromptQueue = serde_json::from_value(serde_json::json!({
        "active": null,
        "queued": [{
            "prompt_id": "prompt_01",
            "user_message_id": "msg_01",
            "status": "queued",
            "content": [{ "type": "text", "text": "next" }],
            "created_at": "2026-07-18T08:00:00.000Z"
        }]
    }))
    .unwrap();

    assert_eq!(queue.queued[0].status, PromptStatus::Queued);
    assert!(matches!(
        queue.queued[0].content[0],
        MessageContent::Text(_)
    ));
}

#[test]
fn decodes_snapshot_subagents_with_runtime_identity() {
    let mut value: serde_json::Value = serde_json::from_str(SNAPSHOT).unwrap();
    value["subagents"] = serde_json::json!([{
        "id": "agent_01", "session_id": "sess_01", "kind": "subagent",
        "description": "Review the implementation", "status": "running",
        "created_at": "2026-07-18T08:00:00.000Z", "subagent_phase": "working",
        "subagent_type": "reviewer", "swarm_index": 0
    }]);

    let snapshot: SessionSnapshot = serde_json::from_value(value).unwrap();

    assert_eq!(snapshot.subagents[0].kind, TaskKind::Subagent);
    assert_eq!(snapshot.subagents[0].status, TaskStatus::Running);
    assert_eq!(
        snapshot.subagents[0].subagent_type.as_deref(),
        Some("reviewer")
    );
}
