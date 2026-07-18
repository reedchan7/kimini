use crate::protocol::{
    TerminalExit, TerminalExitFrame, TerminalOutput, TerminalOutputFrame, WireEvent,
};

pub(super) enum IncomingFrame {
    ServerHello,
    Ping(String),
    ResyncRequired { session_id: String, reason: String },
    ControlError { message: String, fatal: bool },
    HelloAck(Result<Vec<String>, String>),
    TerminalOutput(TerminalOutput),
    TerminalExit(TerminalExit),
    Event(WireEvent),
    Ignore,
}

pub(super) fn parse(text: &str) -> Result<IncomingFrame, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|error| format!("invalid WebSocket frame: {error}"))?;
    let kind = value
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    match kind {
        "server_hello" => Ok(IncomingFrame::ServerHello),
        "ping" => Ok(value
            .pointer("/payload/nonce")
            .and_then(serde_json::Value::as_str)
            .map(|nonce| IncomingFrame::Ping(nonce.to_owned()))
            .unwrap_or(IncomingFrame::Ignore)),
        "resync_required" => Ok(IncomingFrame::ResyncRequired {
            session_id: string_at(&value, "/payload/session_id"),
            reason: string_at(&value, "/payload/reason"),
        }),
        "error" if value.get("session_id").is_none() => {
            let (message, fatal) = control_error(&value);
            Ok(IncomingFrame::ControlError { message, fatal })
        }
        "ack" => Ok(hello_ack(&value)
            .map(IncomingFrame::HelloAck)
            .unwrap_or(IncomingFrame::Ignore)),
        "terminal_output" => serde_json::from_value::<TerminalOutputFrame>(value)
            .map(|frame| IncomingFrame::TerminalOutput(frame.into()))
            .map_err(|error| format!("invalid terminal output frame: {error}")),
        "terminal_exit" => serde_json::from_value::<TerminalExitFrame>(value)
            .map(|frame| IncomingFrame::TerminalExit(frame.into()))
            .map_err(|error| format!("invalid terminal exit frame: {error}")),
        _ if value.get("seq").is_some() => serde_json::from_value(value)
            .map(IncomingFrame::Event)
            .map_err(|error| format!("invalid event frame: {error}")),
        _ => Ok(IncomingFrame::Ignore),
    }
}

fn string_at(value: &serde_json::Value, pointer: &str) -> String {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn control_error(frame: &serde_json::Value) -> (String, bool) {
    let code = frame
        .pointer("/payload/code")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default();
    let message = string_at(frame, "/payload/msg");
    let fatal = frame
        .pointer("/payload/fatal")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    (
        format!("Kimi daemon WebSocket error {code}: {message}"),
        fatal,
    )
}

fn hello_ack(frame: &serde_json::Value) -> Option<Result<Vec<String>, String>> {
    if frame.get("id").and_then(serde_json::Value::as_str) != Some("hello_1") {
        return None;
    }
    let code = frame
        .get("code")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default();
    if code == 0 {
        let sessions = frame
            .pointer("/payload/resync_required")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
            .map(str::to_owned)
            .collect();
        return Some(Ok(sessions));
    }
    let message = frame
        .get("msg")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    Some(Err(format!(
        "Kimi daemon WebSocket hello failed {code}: {message}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_errors_surface_safe_fields_and_fatality() {
        let frame = serde_json::json!({
            "type": "error",
            "payload": { "code": 401, "msg": "unauthorized", "fatal": true }
        });
        assert_eq!(
            control_error(&frame),
            ("Kimi daemon WebSocket error 401: unauthorized".into(), true)
        );
    }

    #[test]
    fn connection_requires_a_successful_client_hello_ack() {
        let success = serde_json::json!({
            "type": "ack", "id": "hello_1", "code": 0, "msg": "success",
            "payload": { "accepted_subscriptions": ["sess_01"] }
        });
        let rejected = serde_json::json!({
            "type": "ack", "id": "hello_1", "code": 40112, "msg": "unauthorized",
            "payload": {}
        });
        let unrelated = serde_json::json!({
            "type": "ack", "id": "other", "code": 0, "msg": "success",
            "payload": {}
        });
        let needs_resync = serde_json::json!({
            "type": "ack", "id": "hello_1", "code": 0, "msg": "success",
            "payload": { "accepted_subscriptions": [], "resync_required": ["sess_02"] }
        });

        assert!(matches!(hello_ack(&success), Some(Ok(sessions)) if sessions.is_empty()));
        assert!(matches!(hello_ack(&needs_resync), Some(Ok(sessions)) if sessions == ["sess_02"]));
        assert!(
            matches!(hello_ack(&rejected), Some(Err(error)) if error == "Kimi daemon WebSocket hello failed 40112: unauthorized")
        );
        assert!(hello_ack(&unrelated).is_none());
    }
}
