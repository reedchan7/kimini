use std::collections::{BTreeMap, VecDeque};
use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tungstenite::client::IntoClientRequest;
use tungstenite::http::HeaderValue;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message as WsMessage, WebSocket};

use crate::daemon::Connection;
use crate::protocol::{ClientControl, SessionCursor};

use super::SocketEvent;
use super::frame::{self, IncomingFrame};

const READ_TICK: Duration = Duration::from_millis(100);
const BEARER_PROTOCOL: &str = "kimi-code.bearer.";

pub(super) fn run(
    connection: &Connection,
    client_id: &str,
    cursors: BTreeMap<String, SessionCursor>,
    events: &async_channel::Sender<SocketEvent>,
    commands: &async_channel::Receiver<ClientControl>,
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
    let mut ready = false;
    let mut pending = VecDeque::new();

    while !stop.load(Ordering::Relaxed) {
        drain_controls(&mut socket, commands, ready, &mut pending)?;
        match socket.read() {
            Ok(WsMessage::Text(text)) => match frame::parse(&text)? {
                IncomingFrame::ServerHello if !hello_sent => {
                    write_control(
                        &mut socket,
                        ClientControl::hello("hello_1", client_id, cursors.clone()),
                    )?;
                    hello_sent = true;
                }
                IncomingFrame::Ping(nonce) => {
                    write_control(&mut socket, ClientControl::pong(&nonce))?;
                }
                IncomingFrame::ResyncRequired { session_id, reason } => {
                    send(events, SocketEvent::ResyncRequired { session_id, reason });
                }
                IncomingFrame::ControlError { message, fatal } => {
                    send(events, SocketEvent::Error { message, fatal });
                    if fatal {
                        return Ok(());
                    }
                }
                IncomingFrame::HelloAck(Ok(resync_sessions)) => {
                    ready = true;
                    while let Some(control) = pending.pop_front() {
                        write_control(&mut socket, control)?;
                    }
                    for session_id in resync_sessions {
                        send(
                            events,
                            SocketEvent::ResyncRequired {
                                session_id,
                                reason: "client_hello".into(),
                            },
                        );
                    }
                    send(events, SocketEvent::Connected);
                }
                IncomingFrame::HelloAck(Err(message)) => {
                    send(
                        events,
                        SocketEvent::Error {
                            message,
                            fatal: true,
                        },
                    );
                    return Ok(());
                }
                IncomingFrame::TerminalOutput(output) => {
                    send(events, SocketEvent::TerminalOutput(output));
                }
                IncomingFrame::TerminalExit(exit) => {
                    send(events, SocketEvent::TerminalExit(exit));
                }
                IncomingFrame::Event(event) => send(events, SocketEvent::Event(event)),
                IncomingFrame::ServerHello | IncomingFrame::Ignore => {}
            },
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

fn send(events: &async_channel::Sender<SocketEvent>, event: SocketEvent) {
    let _ = events.send_blocking(event);
}

fn drain_controls(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    commands: &async_channel::Receiver<ClientControl>,
    ready: bool,
    pending: &mut VecDeque<ClientControl>,
) -> Result<(), String> {
    while let Ok(control) = commands.try_recv() {
        if ready {
            write_control(socket, control)?;
        } else {
            pending.push_back(control);
        }
    }
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
}
