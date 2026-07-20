use std::collections::BTreeMap;

use gpui::{AppContext, Context};

use crate::api::{EventSocket, KimiClient, SocketEvent};
use crate::model::ApplyOutcome;

use super::app::{LoadState, Shell, UtilityPanel};
use super::bootstrap::{self, Bootstrap, LoadedSession};
use super::streaming::start_streaming;

impl Shell {
    pub(super) fn start_bootstrap(&mut self, cx: &mut Context<Self>) {
        self.bootstrap_generation = self.bootstrap_generation.wrapping_add(1);
        let generation = self.bootstrap_generation;
        let task = cx.background_spawn(async { bootstrap::load() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                if generation == this.bootstrap_generation {
                    this.finish_bootstrap(result, cx);
                }
            });
        })
        .detach();
    }

    fn finish_bootstrap(&mut self, result: Result<Bootstrap, String>, cx: &mut Context<Self>) {
        match result {
            Ok(bootstrap) => {
                self.client = Some(KimiClient::new(bootstrap.connection.clone()));
                self.connection = Some(bootstrap.connection);
                self.server_meta = bootstrap.meta;
                self.auth.summary = bootstrap.auth;
                match bootstrap.config {
                    Ok(config) => self.install_daemon_config(config),
                    Err(error) => {
                        self.daemon_config = None;
                        self.config_error = Some(error);
                    }
                }
                self.model.replace_workspaces(bootstrap.workspaces);
                self.model.replace_session_page(bootstrap.sessions);
                self.models = bootstrap.models;
                if let Some(active) = bootstrap.active {
                    self.install_snapshot(active, cx);
                }
                self.state = LoadState::Ready;
            }
            Err(error) => self.state = LoadState::Failed(error),
        }
        cx.notify();
    }

    fn install_snapshot(&mut self, loaded: LoadedSession, cx: &mut Context<Self>) {
        self.history_loading = false;
        let cursor = loaded.snapshot.cursor();
        let session_id = loaded.snapshot.session.id.clone();
        let snapshot_subagents = loaded.snapshot.subagents.clone();
        self.model.seed(loaded.snapshot);
        if let Some(status) = loaded.status {
            self.model.set_runtime(&session_id, status);
        }
        self.model.set_goal(&session_id, loaded.goal);
        if let Some(prompts) = loaded.prompts {
            self.prompt_queues.replace(session_id.clone(), prompts);
        }
        self.tasks.install(
            session_id.clone(),
            snapshot_subagents,
            loaded.tasks.map(|tasks| tasks.items).unwrap_or_default(),
        );
        if let Some(skills) = loaded.skills {
            self.skills.begin_load(session_id.clone());
            self.skills.install(&session_id, skills.skills);
        }
        self.transcript.rebuild(&self.model);
        if self.utility_panel == Some(UtilityPanel::Files) {
            self.refresh_workspace_files(cx);
        } else {
            self.refresh_workspace_git_status(cx);
        }
        if self.utility_panel == Some(UtilityPanel::Skills) {
            self.refresh_skills(cx);
        }
        if self.utility_panel == Some(UtilityPanel::Terminal) {
            self.refresh_terminals(cx);
        }
        if let Some(connection) = self.connection.clone() {
            let socket = EventSocket::connect(
                connection,
                "kimini-native",
                BTreeMap::from([(session_id.clone(), cursor)]),
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
        // Daemon journals often omit assistant.delta/message.created. If the
        // turn is still running after this snapshot, poll until idle and reload
        // so the final assistant message is not lost when socket catch-up fails.
        let turn_active = self.model.active_session().is_some_and(|session| {
            session.busy || session.main_turn_active.unwrap_or(false)
        }) || self.model.active_conversation().is_some_and(|conversation| {
            conversation.assistant_stream.is_some()
                && !conversation
                    .messages
                    .iter()
                    .any(|message| message.role == crate::protocol::MessageRole::Assistant)
        });
        if turn_active {
            self.watch_turn_until_idle(session_id, cx);
        }
    }

    /// Poll session status until the main turn settles, then reload the snapshot.
    /// Acts as a backup when WebSocket streaming events are missing or delayed.
    pub(super) fn watch_turn_until_idle(&mut self, session_id: String, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        self.turn_watch_generation = self.turn_watch_generation.wrapping_add(1);
        let generation = self.turn_watch_generation;
        let task = cx.background_spawn(async move {
            use std::time::Duration;
            for _ in 0..90 {
                match client.session_status(&session_id) {
                    Ok(status) if !status.busy => break,
                    _ => std::thread::sleep(Duration::from_millis(400)),
                }
            }
            Ok::<_, crate::api::ApiError>(session_id)
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                if generation != this.turn_watch_generation {
                    return;
                }
                match result {
                    Ok(session_id) if this.is_active_session(&session_id) => {
                        this.load_snapshot(session_id, cx);
                    }
                    Ok(_) => {}
                    Err(error) => {
                        if matches!(this.state, LoadState::Working(_)) {
                            this.state = LoadState::Failed(error.to_string());
                            cx.notify();
                        }
                    }
                }
            });
        })
        .detach();
    }

    fn handle_socket_event(&mut self, generation: u64, event: SocketEvent, cx: &mut Context<Self>) {
        if generation != self.socket_generation {
            return;
        }
        let mut reload = false;
        match event {
            SocketEvent::Event(event) => {
                let kind = event.kind.clone();
                let side_chat_event = self.side_chats.owns_event(&event);
                self.side_chats.apply_event(&event);
                let task_changed = !side_chat_event && self.tasks.apply_event(&event);
                let snapshot_boundary = !side_chat_event
                    && matches!(
                        kind.as_str(),
                        "turn.step.completed"
                            | "event.turn.step.completed"
                            | "turn.ended"
                            | "event.turn.ended"
                            | "prompt.completed"
                            | "event.prompt.completed"
                            | "prompt.aborted"
                            | "event.prompt.aborted"
                            | "prompt.steered"
                            | "event.prompt.steered"
                    );
                let outcome = self.model.apply(event);
                reload = outcome == ApplyOutcome::ResyncRequired || snapshot_boundary;
                if outcome == ApplyOutcome::Applied && !side_chat_event {
                    match kind.as_str() {
                        "assistant.delta"
                        | "event.assistant.delta"
                        | "thinking.delta"
                        | "event.thinking.delta"
                        | "turn.started"
                        | "event.turn.started"
                        | "turn.step.started"
                        | "event.turn.step.started"
                        | "turn.step.retrying"
                        | "event.turn.step.retrying" => {
                            self.sync_streaming(cx);
                            self.transcript.sync_stream(&self.model);
                        }
                        "event.message.created" => {
                            // The assistant turn has been promoted to a real
                            // message: drop the streaming entity so the next
                            // turn starts from a fresh parse.
                            self.streaming = None;
                            self.transcript.rebuild(&self.model);
                        }
                        _ => {}
                    }
                }
                if task_changed || (!side_chat_event && is_task_boundary(&kind)) {
                    self.schedule_task_poll(cx);
                }
            }
            SocketEvent::ResyncRequired { .. } => reload = true,
            SocketEvent::TerminalOutput(output) => {
                if self.terminals.apply_output(output) {
                    self.terminal_scroll.scroll_to_bottom();
                }
            }
            SocketEvent::TerminalExit(exit) => {
                self.terminals.apply_exit(exit);
            }
            SocketEvent::Error { message, fatal } => {
                self.state = LoadState::Failed(message);
                if fatal {
                    self.socket_generation = self.socket_generation.wrapping_add(1);
                    self.socket = None;
                }
            }
            SocketEvent::Closed => reload = true,
            SocketEvent::Connected => self.attach_active_terminal(),
        }
        if reload {
            self.reload_active(cx);
        }
        cx.notify();
    }

    pub(super) fn select_session(
        &mut self,
        session_id: String,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.store_active_composer_draft(cx);
        if let Some(draft) = self.new_session_draft.take()
            && !draft.submitting
        {
            let key = draft.key();
            self.attachments.discard_session(&key);
            self.drafts.remove(&key);
        }
        self.composer_menu = None;
        self.draft_workspace_menu_open = false;
        self.draft_workspace_show_all = false;
        self.renaming_session_id = None;
        self.streaming = None;
        self.load_snapshot(session_id, cx);
        self.composer
            .update(cx, |input, cx| input.focus(window, cx));
    }

    pub(in crate::native) fn reload_active(&mut self, cx: &mut Context<Self>) {
        // Snapshot boundary reached (turn ended / prompt completed / socket
        // resync): the journal snapshot now owns the turn's content, so the
        // streaming entity is stale. Drop it so the next turn starts clean.
        self.streaming = None;
        if let Some(session) = self.model.active_session() {
            self.load_snapshot(session.id.clone(), cx);
        }
    }

    /// Diff the active conversation's assistant + thinking snapshots into the
    /// persistent streaming entities. Creates the pair lazily on the first
    /// delta of a new turn, and no-ops when the snapshot hasn't changed.
    pub(super) fn sync_streaming(&mut self, cx: &mut Context<Self>) {
        let Some(conversation) = self.model.active_conversation() else {
            return;
        };
        let assistant = conversation.assistant_stream.as_deref().unwrap_or("");
        let thinking = conversation.thinking_stream.as_deref().unwrap_or("");
        let streaming = self
            .streaming
            .get_or_insert_with(|| start_streaming(cx));
        streaming.sync_assistant(assistant, cx);
        streaming.sync_thinking(thinking, cx);
    }

    pub(super) fn load_snapshot(&mut self, session_id: String, cx: &mut Context<Self>) {
        self.load_snapshot_with_notice(session_id, None, cx);
    }

    pub(super) fn load_snapshot_with_notice(
        &mut self,
        session_id: String,
        notice: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(client) = self.client.clone() else {
            return;
        };
        self.snapshot_generation = self.snapshot_generation.wrapping_add(1);
        self.preview_thinking = None;
        let generation = self.snapshot_generation;
        self.state = LoadState::Working(self.strings.native.working.into());
        let task =
            cx.background_spawn(async move { bootstrap::load_session(&client, &session_id) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if generation != this.snapshot_generation {
                    return;
                }
                match result {
                    Ok(loaded) => {
                        this.install_snapshot(loaded, cx);
                        this.state = notice
                            .clone()
                            .map(LoadState::Failed)
                            .unwrap_or(LoadState::Ready);
                    }
                    Err(error) => this.state = LoadState::Failed(error),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

fn is_task_boundary(kind: &str) -> bool {
    matches!(
        kind.strip_prefix("event.").unwrap_or(kind),
        "task.started"
            | "task.terminated"
            | "background.task.started"
            | "background.task.terminated"
            | "subagent.spawned"
            | "subagent.started"
            | "subagent.suspended"
            | "subagent.completed"
            | "subagent.failed"
    )
}
