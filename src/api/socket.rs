use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use tungstenite::client::IntoClientRequest;
use tungstenite::http::HeaderValue;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message as WsMessage, WebSocket};

use crate::daemon::Connection;
use crate::protocol::{ClientControl, SessionCursor, WireEvent};

const READ_TICK: Duration = Duration::from_millis(100);
const BEARER_PROTOCOL: &str = "kimi-code.bearer.";

#[derive(Debug)]
pub enum SocketEvent {
    Connected,
    Event(WireEvent),
    ResyncRequired { session_id: String, reason: String },
    Error { message: String, fatal: bool },
    Closed,
}

pub struct EventSocket {
    events: async_channel::Receiver<SocketEvent>,
    stop: Arc<AtomicBool>,
}

impl EventSocket {
    pub fn connect(
        connection: Connection,
        client_id: impl Into<String>,
        cursors: BTreeMap<String, SessionCursor>,
    ) -> Self {
        let (event_tx, event_rx) = async_channel::bounded(1024);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        let client_id = client_id.into();
        thread::spawn(move || {
            if let Err(error) =
                run_socket(&connection, &client_id, cursors, &event_tx, &thread_stop)
            {
                let _ = event_tx.send_blocking(SocketEvent::Error {
                    message: error,
                    fatal: false,
                });
            }
            let _ = event_tx.send_blocking(SocketEvent::Closed);
        });
        Self {
            events: event_rx,
            stop,
        }
    }

    pub fn events(&self) -> async_channel::Receiver<SocketEvent> {
        self.events.clone()
    }
}

impl Drop for EventSocket {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn run_socket(
    connection: &Connection,
    client_id: &str,
    cursors: BTreeMap<String, SessionCursor>,
    events: &async_channel::Sender<SocketEvent>,
    stop: &AtomicBool,
) -> Result<(), String> {
    let mut request = socket_url(connection, client_id)
        .into_client_request()
        .map_err(|error| error.to_string())?;
    if let Some(token) = connection.token() {
        let protocol = HeaderValue::from_str(&format!("{BEARER_PROTOCOL}{token}"))
            .map_err(|error| error.to_string())?;
        request
            .headers_mut()
            .insert("Sec-WebSocket-Protocol", protocol);
    }
    let (mut socket, _) = tungstenite::connect(request).map_err(|error| error.to_string())?;
    configure_read_tick(&mut socket);
    let mut hello_sent = false;

    while !stop.load(Ordering::Relaxed) {
        match socket.read() {
            Ok(WsMessage::Text(text)) => {
                let frame: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|error| format!("invalid WebSocket frame: {error}"))?;
                let kind = frame
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                match kind {
                    "server_hello" if !hello_sent => {
                        write_control(
                            &mut socket,
                            ClientControl::hello("hello_1", client_id, cursors.clone()),
                        )?;
                        hello_sent = true;
                    }
                    "ping" => {
                        if let Some(nonce) = frame
                            .pointer("/payload/nonce")
                            .and_then(serde_json::Value::as_str)
                        {
                            write_control(&mut socket, ClientControl::pong(nonce))?;
                        }
                    }
                    "resync_required" => {
                        let session_id = string_at(&frame, "/payload/session_id");
                        let reason = string_at(&frame, "/payload/reason");
                        let _ = events
                            .send_blocking(SocketEvent::ResyncRequired { session_id, reason });
                    }
                    "error" if frame.get("session_id").is_none() => {
                        let (message, fatal) = control_error(&frame);
                        let _ = events.send_blocking(SocketEvent::Error { message, fatal });
                        if fatal {
                            return Ok(());
                        }
                    }
                    "ack" => match hello_ack(&frame) {
                        Some(Ok(resync_sessions)) => {
                            for session_id in resync_sessions {
                                let _ = events.send_blocking(SocketEvent::ResyncRequired {
                                    session_id,
                                    reason: "client_hello".into(),
                                });
                            }
                            let _ = events.send_blocking(SocketEvent::Connected);
                        }
                        Some(Err(error)) => {
                            let _ = events.send_blocking(SocketEvent::Error {
                                message: error,
                                fatal: true,
                            });
                            return Ok(());
                        }
                        None => {}
                    },
                    _ if frame.get("seq").is_some() => {
                        let event = serde_json::from_value(frame)
                            .map_err(|error| format!("invalid event frame: {error}"))?;
                        let _ = events.send_blocking(SocketEvent::Event(event));
                    }
                    _ => {}
                }
            }
            Ok(WsMessage::Close(_)) => return Ok(()),
            Ok(WsMessage::Ping(payload)) => socket
                .send(WsMessage::Pong(payload))
                .map_err(|error| error.to_string())?,
            Ok(_) => {}
            Err(tungstenite::Error::Io(error))
                if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    let _ = socket.close(None);
    Ok(())
}

fn socket_url(connection: &Connection, client_id: &str) -> String {
    let scheme = if connection.origin().starts_with("https://") {
        "wss"
    } else {
        "ws"
    };
    let origin = connection
        .origin()
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let client_id: String = url::form_urlencoded::byte_serialize(client_id.as_bytes()).collect();
    format!("{scheme}://{origin}/api/v1/ws?client_id={client_id}")
}

fn configure_read_tick(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
    if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
        let _ = stream.set_read_timeout(Some(READ_TICK));
    }
}

fn write_control(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    control: ClientControl,
) -> Result<(), String> {
    let message = serde_json::to_string(&control).map_err(|error| error.to_string())?;
    socket
        .send(WsMessage::Text(message.into()))
        .map_err(|error| error.to_string())
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
        let resync_sessions = frame
            .pointer("/payload/resync_required")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
            .map(str::to_owned)
            .collect();
        return Some(Ok(resync_sessions));
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
    fn websocket_url_uses_the_daemon_origin_and_escaped_client_id() {
        let connection = Connection::new("http://127.0.0.1:58627", None);
        assert_eq!(
            socket_url(&connection, "kimini native"),
            "ws://127.0.0.1:58627/api/v1/ws?client_id=kimini+native"
        );
    }

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

        assert_eq!(hello_ack(&success), Some(Ok(Vec::new())));
        assert_eq!(hello_ack(&needs_resync), Some(Ok(vec!["sess_02".into()])));
        assert_eq!(
            hello_ack(&rejected),
            Some(Err(
                "Kimi daemon WebSocket hello failed 40112: unauthorized".into()
            ))
        );
        assert_eq!(hello_ack(&unrelated), None);
    }
}
