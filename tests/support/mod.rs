use kimini::protocol::{SessionSnapshot, WireEvent};

pub fn snapshot(seq: u64, user_text: &str, assistant_text: &str) -> SessionSnapshot {
    serde_json::from_value(serde_json::json!({
      "as_of_seq": seq, "epoch": "ep_01",
      "session": {
        "id": "sess_01", "workspace_id": "ws_01", "title": "Test",
        "created_at": "2026-07-18T08:00:00.000Z", "updated_at": "2026-07-18T08:00:00.000Z",
        "busy": true, "archived": false, "metadata": { "cwd": "/tmp/project" },
        "agent_config": { "model": "kimi-k2" },
        "usage": { "input_tokens": 0, "output_tokens": 0, "cache_read_tokens": 0,
          "cache_creation_tokens": 0, "total_cost_usd": 0, "context_tokens": 0,
          "context_limit": 131072, "turn_count": 0 },
        "permission_rules": [], "message_count": 1, "last_seq": seq
      },
      "messages": { "items": [{
        "id": "msg_01", "session_id": "sess_01", "role": "user",
        "content": [{ "type": "text", "text": user_text }],
        "created_at": "2026-07-18T08:00:00.000Z"
      }], "has_more": false },
      "in_flight_turn": if assistant_text.is_empty() { serde_json::Value::Null } else {
        serde_json::json!({ "turn_id": 1, "assistant_text": assistant_text,
          "thinking_text": "", "running_tools": [] })
      },
      "pending_approvals": [], "pending_questions": []
    }))
    .unwrap()
}

pub fn event(
    kind: &str,
    seq: u64,
    volatile: bool,
    offset: Option<usize>,
    payload: serde_json::Value,
) -> WireEvent {
    WireEvent {
        kind: kind.into(),
        seq,
        epoch: Some("ep_01".into()),
        volatile,
        offset,
        session_id: Some("sess_01".into()),
        timestamp: "2026-07-18T08:01:00.000Z".into(),
        payload,
    }
}
