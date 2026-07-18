use std::collections::BTreeMap;

use gpui::{AppContext, Context};

use crate::api::{EventSocket, KimiClient, SocketEvent};
use crate::model::ApplyOutcome;
use crate::protocol::SessionSnapshot;

use super::app::{LoadState, Shell};
use super::bootstrap::{self, Bootstrap};

impl Shell {
    pub(super) fn start_bootstrap(&mut self, cx: &mut Context<Self>) {
        let task = cx.background_spawn(async { bootstrap::load() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| this.finish_bootstrap(result, cx));
        })
        .detach();
    }

    fn finish_bootstrap(&mut self, result: Result<Bootstrap, String>, cx: &mut Context<Self>) {
        match result {
            Ok(bootstrap) => {
                self.client = Some(KimiClient::new(bootstrap.connection.clone()));
                self.connection = Some(bootstrap.connection);
                self.model.replace_sessions(bootstrap.sessions);
                if let Some(snapshot) = bootstrap.snapshot {
                    self.install_snapshot(snapshot, cx);
                }
                self.state = LoadState::Ready;
            }
            Err(error) => self.state = LoadState::Failed(error),
        }
        cx.notify();
    }

    fn install_snapshot(&mut self, snapshot: SessionSnapshot, cx: &mut Context<Self>) {
        let cursor = snapshot.cursor();
        let session_id = snapshot.session.id.clone();
        self.model.seed(snapshot);
        self.transcript.rebuild(&self.model);
        if let Some(connection) = self.connection.clone() {
            let socket = EventSocket::connect(
                connection,
                "kimini-native",
                BTreeMap::from([(session_id, cursor)]),
            );
            self.socket_generation = self.socket_generation.wrapping_add(1);
            let generation = self.socket_generation;
            let events = socket.events();
            self.socket = Some(socket);
            cx.spawn(async move |this, cx| {
                while let Ok(event) = events.recv().await {
                    if this
                        .update(cx, |this, cx| {
                            this.handle_socket_event(generation, event, cx)
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .detach();
        }
    }

    fn handle_socket_event(&mut self, generation: u64, event: SocketEvent, cx: &mut Context<Self>) {
        if generation != self.socket_generation {
            return;
        }
        let mut reload = false;
        match event {
            SocketEvent::Event(event) => {
                let kind = event.kind.clone();
                let snapshot_boundary = matches!(
                    kind.as_str(),
                    "turn.step.completed"
                        | "event.turn.step.completed"
                        | "turn.ended"
                        | "event.turn.ended"
                );
                let outcome = self.model.apply(event);
                reload = outcome == ApplyOutcome::ResyncRequired || snapshot_boundary;
                if outcome == ApplyOutcome::Applied {
                    match kind.as_str() {
                        "assistant.delta"
                        | "event.assistant.delta"
                        | "turn.started"
                        | "event.turn.started"
                        | "turn.step.started"
                        | "event.turn.step.started"
                        | "turn.step.retrying"
                        | "event.turn.step.retrying" => self.transcript.sync_stream(&self.model),
                        "event.message.created" => self.transcript.rebuild(&self.model),
                        _ => {}
                    }
                }
            }
            SocketEvent::ResyncRequired { .. } => reload = true,
            SocketEvent::Error { message, fatal } => {
                self.state = LoadState::Failed(message);
                if fatal {
                    self.socket_generation = self.socket_generation.wrapping_add(1);
                    self.socket = None;
                }
            }
            SocketEvent::Closed => reload = true,
            SocketEvent::Connected => {}
        }
        if reload {
            self.reload_active(cx);
        }
        cx.notify();
    }

    pub(super) fn select_session(&mut self, session_id: String, cx: &mut Context<Self>) {
        self.load_snapshot(session_id, cx);
    }

    fn reload_active(&mut self, cx: &mut Context<Self>) {
        if let Some(session) = self.model.active_session() {
            self.load_snapshot(session.id.clone(), cx);
        }
    }

    pub(super) fn load_snapshot(&mut self, session_id: String, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        self.snapshot_generation = self.snapshot_generation.wrapping_add(1);
        let generation = self.snapshot_generation;
        self.state = LoadState::Working("Loading conversation…".into());
        let task = cx.background_spawn(async move { client.snapshot(&session_id) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if generation != this.snapshot_generation {
                    return;
                }
                match result {
                    Ok(snapshot) => {
                        this.install_snapshot(snapshot, cx);
                        this.state = LoadState::Ready;
                    }
                    Err(error) => this.state = LoadState::Failed(error),
                }
                cx.notify();
            });
        })
        .detach();
    }
}
