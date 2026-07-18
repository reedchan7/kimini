use std::path::PathBuf;

use gpui::{AppContext, Context, Window};

use crate::api::ApiError;
use crate::protocol::{CreateTerminal, TerminalExit, TerminalStatus};

use super::super::app::{Shell, UtilityPanel};
use super::super::terminal::LocalTerminalEvent;

const DEFAULT_COLS: usize = 120;
const DEFAULT_ROWS: usize = 36;

impl Shell {
    pub(in crate::native) fn start_local_terminal_events(&mut self, cx: &mut Context<Self>) {
        let events = self.local_terminals.events();
        cx.spawn(async move |this, cx| {
            while let Ok(event) = events.recv().await {
                if this
                    .update(cx, |this, cx| {
                        match event {
                            LocalTerminalEvent::Output {
                                session_id,
                                terminal_id,
                                data,
                            } => {
                                if this.terminals.apply_local_output(
                                    &session_id,
                                    &terminal_id,
                                    &data,
                                ) {
                                    this.terminal_scroll.scroll_to_bottom();
                                }
                            }
                            LocalTerminalEvent::Exit {
                                session_id,
                                terminal_id,
                                exit_code,
                            } => {
                                this.terminals.apply_exit(TerminalExit {
                                    session_id,
                                    terminal_id: terminal_id.clone(),
                                    exit_code,
                                });
                                this.local_terminals.reap(&terminal_id);
                            }
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }

    pub(in crate::native) fn toggle_terminal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.utility_panel == Some(UtilityPanel::Terminal) {
            self.detach_active_terminal();
            self.utility_panel = None;
            cx.notify();
            return;
        }
        self.utility_panel = Some(UtilityPanel::Terminal);
        self.terminal_input
            .update(cx, |input, cx| input.focus(window, cx));
        self.refresh_terminals(cx);
        cx.notify();
    }

    pub(in crate::native) fn refresh_terminals(&mut self, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        if !self.terminals.begin_load(&session_id) {
            return;
        }
        let request_session_id = session_id.clone();
        let task = cx.background_spawn(async move { client.list_terminals(&session_id) });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session_id) {
                    return;
                }
                match result {
                    Ok(list) => {
                        this.terminals
                            .install(request_session_id.clone(), list.items);
                        if this.terminals.has_running(&request_session_id) {
                            this.attach_active_terminal();
                        } else {
                            this.create_terminal(cx);
                        }
                    }
                    Err(error) => this.terminals.fail(&request_session_id, error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn create_terminal(&mut self, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        if !self.terminals.begin_load(&session_id) {
            return;
        }
        let request_session_id = session_id.clone();
        let task = cx.background_spawn(async move {
            client.create_terminal(
                &session_id,
                &CreateTerminal {
                    cols: DEFAULT_COLS,
                    rows: DEFAULT_ROWS,
                },
            )
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session_id) {
                    return;
                }
                match result {
                    Ok(terminal) => {
                        this.terminals
                            .add_daemon(request_session_id.clone(), terminal);
                        this.attach_active_terminal();
                    }
                    Err(error) if terminal_backend_unavailable(&error) => {
                        this.create_local_terminal(&request_session_id, error.to_string());
                    }
                    Err(error) => this.terminals.fail(&request_session_id, error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn select_terminal(
        &mut self,
        terminal_id: String,
        cx: &mut Context<Self>,
    ) {
        let Some(session_id) = self
            .model
            .active_session()
            .map(|session| session.id.clone())
        else {
            return;
        };
        if self
            .terminals
            .active(&session_id)
            .is_some_and(|tab| tab.terminal.id == terminal_id)
        {
            return;
        }
        self.detach_active_terminal();
        if self.terminals.select(&session_id, &terminal_id) {
            self.attach_active_terminal();
            self.terminal_scroll.scroll_to_bottom();
            cx.notify();
        }
    }

    pub(in crate::native) fn send_terminal_command(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let command = self.terminal_input.read(cx).value().trim().to_owned();
        if command.is_empty() {
            return;
        }
        let Some((session_id, terminal_id)) = self.active_running_terminal() else {
            return;
        };
        let data = format!("{command}\n");
        let result = if self.terminals.is_local(&session_id, &terminal_id) {
            self.local_terminals.write(&terminal_id, data.as_bytes())
        } else {
            self.socket.as_ref().map_or_else(
                || Err("Terminal connection is offline".into()),
                |socket| socket.terminal_input(&session_id, &terminal_id, &data),
            )
        };
        match result {
            Ok(()) => {
                self.terminal_input
                    .update(cx, |input, cx| input.set_value("", window, cx));
            }
            Err(error) => self.terminals.fail(&session_id, error),
        }
        cx.notify();
    }

    pub(in crate::native) fn close_active_terminal(&mut self, cx: &mut Context<Self>) {
        let Some(session_id) = self
            .model
            .active_session()
            .map(|session| session.id.clone())
        else {
            return;
        };
        let Some((terminal_id, is_local)) = self
            .terminals
            .active(&session_id)
            .map(|tab| (tab.terminal.id.clone(), tab.is_local()))
        else {
            return;
        };
        if is_local {
            if self.local_terminals.contains(&terminal_id)
                && let Err(error) = self.local_terminals.close(&terminal_id)
            {
                self.terminals.fail(&session_id, error);
                cx.notify();
                return;
            }
            self.terminals.remove(&session_id, &terminal_id);
            self.attach_active_terminal();
            cx.notify();
            return;
        }
        let Some(client) = self.client.clone() else {
            return;
        };
        if let Some(socket) = self.socket.as_ref() {
            let _ = socket.terminal_close(&session_id, &terminal_id);
        }
        let request_session_id = session_id.clone();
        let request_terminal_id = terminal_id.clone();
        let task = cx.background_spawn(async move {
            client.close_terminal(&session_id, &terminal_id)?;
            Ok::<_, ApiError>(())
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.map_err(|error| error.to_string());
            let _ = this.update(cx, |this, cx| {
                if !this.is_active_session(&request_session_id) {
                    return;
                }
                match result {
                    Ok(()) => {
                        this.terminals
                            .remove(&request_session_id, &request_terminal_id);
                        this.attach_active_terminal();
                    }
                    Err(error) => this.terminals.fail(&request_session_id, error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::native) fn attach_active_terminal(&mut self) {
        let Some(session_id) = self
            .model
            .active_session()
            .map(|session| session.id.clone())
        else {
            return;
        };
        let Some((terminal_id, last_seq, cols, rows, running, is_local)) =
            self.terminals.active(&session_id).map(|tab| {
                (
                    tab.terminal.id.clone(),
                    tab.last_seq,
                    tab.terminal.cols,
                    tab.terminal.rows,
                    tab.terminal.status == TerminalStatus::Running,
                    tab.is_local(),
                )
            })
        else {
            return;
        };
        if !running {
            return;
        }
        let result = if is_local {
            self.local_terminals.resize(&terminal_id, cols, rows)
        } else {
            self.socket.as_ref().map_or_else(
                || Err("Terminal connection is offline".into()),
                |socket| {
                    socket.terminal_attach(&session_id, &terminal_id, Some(last_seq))?;
                    socket.terminal_resize(&session_id, &terminal_id, cols, rows)
                },
            )
        };
        if let Err(error) = result {
            self.terminals.fail(&session_id, error);
        }
    }

    fn detach_active_terminal(&mut self) {
        let Some(session_id) = self
            .model
            .active_session()
            .map(|session| session.id.clone())
        else {
            return;
        };
        let Some((terminal_id, is_local)) = self
            .terminals
            .active(&session_id)
            .map(|tab| (tab.terminal.id.clone(), tab.is_local()))
        else {
            return;
        };
        if is_local {
            return;
        }
        if let Some(socket) = self.socket.as_ref() {
            let _ = socket.terminal_detach(&session_id, &terminal_id);
        }
    }

    fn active_running_terminal(&self) -> Option<(String, String)> {
        let session_id = self.model.active_session()?.id.clone();
        let terminal = self.terminals.active(&session_id)?;
        (terminal.terminal.status == TerminalStatus::Running)
            .then(|| (session_id, terminal.terminal.id.clone()))
    }

    fn create_local_terminal(&mut self, session_id: &str, daemon_error: String) {
        let Some(cwd) = self
            .model
            .active_session()
            .filter(|session| session.id == session_id)
            .map(|session| PathBuf::from(&session.metadata.cwd))
        else {
            self.terminals.fail(
                session_id,
                format!("{daemon_error}; active session is unavailable"),
            );
            return;
        };
        match self
            .local_terminals
            .spawn(session_id, &cwd, DEFAULT_COLS, DEFAULT_ROWS)
        {
            Ok(terminal) => {
                self.terminals.add_local(session_id.into(), terminal);
                self.terminals.set_notice(
                    session_id,
                    self.strings.native.terminal_local_fallback.into(),
                );
                self.attach_active_terminal();
            }
            Err(local_error) => self.terminals.fail(
                session_id,
                format!("{daemon_error}; local terminal fallback failed: {local_error}"),
            ),
        }
    }
}

fn terminal_backend_unavailable(error: &ApiError) -> bool {
    let ApiError::Daemon { message, .. } = error else {
        return false;
    };
    [
        "spawn is not a function",
        "Failed to load native module: pty.node",
        "ERR_UNKNOWN_BUILTIN_MODULE",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_known_packaged_daemon_pty_failures() {
        for message in [
            "(intermediate value).spawn is not a function",
            "Failed to load native module: pty.node (ERR_UNKNOWN_BUILTIN_MODULE)",
        ] {
            assert!(terminal_backend_unavailable(&ApiError::Daemon {
                code: 50_001,
                message: message.into(),
            }));
        }
        assert!(!terminal_backend_unavailable(&ApiError::Daemon {
            code: 50_001,
            message: "permission denied".into(),
        }));
    }
}
