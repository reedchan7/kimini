mod frame;
mod worker;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;

use crate::daemon::Connection;
use crate::protocol::{ClientControl, SessionCursor, TerminalExit, TerminalOutput, WireEvent};

#[derive(Debug)]
pub enum SocketEvent {
    Connected,
    Event(WireEvent),
    ResyncRequired { session_id: String, reason: String },
    Error { message: String, fatal: bool },
    TerminalOutput(TerminalOutput),
    TerminalExit(TerminalExit),
    Closed,
}

pub struct EventSocket {
    events: async_channel::Receiver<SocketEvent>,
    commands: async_channel::Sender<ClientControl>,
    next_id: AtomicU64,
    stop: Arc<AtomicBool>,
}

impl EventSocket {
    pub fn connect(
        connection: Connection,
        client_id: impl Into<String>,
        cursors: BTreeMap<String, SessionCursor>,
    ) -> Self {
        let (event_tx, event_rx) = async_channel::bounded(1024);
        let (command_tx, command_rx) = async_channel::bounded(256);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        let client_id = client_id.into();
        thread::spawn(move || {
            if let Err(error) = worker::run(
                &connection,
                &client_id,
                cursors,
                &event_tx,
                &command_rx,
                &thread_stop,
            ) {
                let _ = event_tx.send_blocking(SocketEvent::Error {
                    message: error,
                    fatal: false,
                });
            }
            let _ = event_tx.send_blocking(SocketEvent::Closed);
        });
        Self {
            events: event_rx,
            commands: command_tx,
            next_id: AtomicU64::new(1),
            stop,
        }
    }

    pub fn events(&self) -> async_channel::Receiver<SocketEvent> {
        self.events.clone()
    }

    pub fn terminal_attach(
        &self,
        session_id: &str,
        terminal_id: &str,
        since_seq: Option<u64>,
    ) -> Result<(), String> {
        self.send(ClientControl::terminal_attach(
            self.next_control_id(),
            session_id,
            terminal_id,
            since_seq,
        ))
    }

    pub fn terminal_detach(&self, session_id: &str, terminal_id: &str) -> Result<(), String> {
        self.send(ClientControl::terminal_detach(
            self.next_control_id(),
            session_id,
            terminal_id,
        ))
    }

    pub fn terminal_input(
        &self,
        session_id: &str,
        terminal_id: &str,
        data: &str,
    ) -> Result<(), String> {
        self.send(ClientControl::terminal_input(
            self.next_control_id(),
            session_id,
            terminal_id,
            data,
        ))
    }

    pub fn terminal_resize(
        &self,
        session_id: &str,
        terminal_id: &str,
        cols: usize,
        rows: usize,
    ) -> Result<(), String> {
        self.send(ClientControl::terminal_resize(
            self.next_control_id(),
            session_id,
            terminal_id,
            cols,
            rows,
        ))
    }

    pub fn terminal_close(&self, session_id: &str, terminal_id: &str) -> Result<(), String> {
        self.send(ClientControl::terminal_close(
            self.next_control_id(),
            session_id,
            terminal_id,
        ))
    }

    fn next_control_id(&self) -> String {
        format!("terminal_{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    fn send(&self, control: ClientControl) -> Result<(), String> {
        self.commands
            .try_send(control)
            .map_err(|error| format!("Kimi daemon WebSocket command failed: {error}"))
    }
}

impl Drop for EventSocket {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
