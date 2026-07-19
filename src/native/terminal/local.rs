use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use async_channel::{Receiver, Sender};
use portable_pty::{ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};

use crate::protocol::{Terminal, TerminalStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::native) enum LocalTerminalEvent {
    Output {
        session_id: String,
        terminal_id: String,
        data: Vec<u8>,
    },
    Exit {
        session_id: String,
        terminal_id: String,
        exit_code: Option<i32>,
    },
}

struct LocalProcess {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
}

pub(in crate::native) struct LocalTerminalHost {
    processes: HashMap<String, LocalProcess>,
    events_tx: Sender<LocalTerminalEvent>,
    events_rx: Receiver<LocalTerminalEvent>,
    next_id: u64,
}

impl Default for LocalTerminalHost {
    fn default() -> Self {
        let (events_tx, events_rx) = async_channel::unbounded();
        Self {
            processes: HashMap::new(),
            events_tx,
            events_rx,
            next_id: 0,
        }
    }
}

impl LocalTerminalHost {
    pub fn events(&self) -> Receiver<LocalTerminalEvent> {
        self.events_rx.clone()
    }

    pub fn spawn(
        &mut self,
        session_id: &str,
        cwd: &Path,
        cols: usize,
        rows: usize,
    ) -> Result<Terminal, String> {
        self.spawn_with_shell(session_id, cwd, &preferred_shell(), cols, rows)
    }

    fn spawn_with_shell(
        &mut self,
        session_id: &str,
        cwd: &Path,
        shell: &Path,
        cols: usize,
        rows: usize,
    ) -> Result<Terminal, String> {
        if !cwd.is_dir() {
            return Err(format!(
                "Terminal working directory does not exist: {}",
                cwd.display()
            ));
        }
        if !shell.is_file() {
            return Err(format!(
                "Terminal shell does not exist: {}",
                shell.display()
            ));
        }

        let size = pty_size(cols, rows);
        let pair = native_pty_system()
            .openpty(size)
            .map_err(|error| format!("Unable to open local terminal: {error}"))?;
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| format!("Unable to read local terminal: {error}"))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| format!("Unable to write local terminal: {error}"))?;

        let mut command = CommandBuilder::new(shell);
        command.cwd(cwd);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        let mut child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| format!("Unable to start local terminal: {error}"))?;
        let killer = child.clone_killer();
        drop(pair.slave);

        self.next_id = self.next_id.wrapping_add(1);
        let terminal_id = format!(
            "local_{}_{}_{}",
            std::process::id(),
            now_millis(),
            self.next_id
        );
        let terminal = Terminal {
            id: terminal_id.clone(),
            session_id: session_id.to_owned(),
            cwd: cwd.to_string_lossy().into_owned(),
            shell: shell.to_string_lossy().into_owned(),
            cols: usize::from(size.cols),
            rows: usize::from(size.rows),
            status: TerminalStatus::Running,
            created_at: format!("unix-ms:{}", now_millis()),
            exited_at: None,
            exit_code: None,
        };
        self.processes.insert(
            terminal_id.clone(),
            LocalProcess {
                master: pair.master,
                writer,
                killer,
            },
        );

        let output_tx = self.events_tx.clone();
        let output_session_id = session_id.to_owned();
        let output_terminal_id = terminal_id.clone();
        std::thread::spawn(move || {
            let mut buffer = [0_u8; 8 * 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(read) => {
                        if output_tx
                            .send_blocking(LocalTerminalEvent::Output {
                                session_id: output_session_id.clone(),
                                terminal_id: output_terminal_id.clone(),
                                data: buffer[..read].to_vec(),
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
        });

        let exit_tx = self.events_tx.clone();
        let exit_session_id = session_id.to_owned();
        let exit_terminal_id = terminal_id;
        std::thread::spawn(move || {
            let exit_code = child
                .wait()
                .ok()
                .and_then(|status| i32::try_from(status.exit_code()).ok());
            let _ = exit_tx.send_blocking(LocalTerminalEvent::Exit {
                session_id: exit_session_id,
                terminal_id: exit_terminal_id,
                exit_code,
            });
        });

        Ok(terminal)
    }

    pub fn contains(&self, terminal_id: &str) -> bool {
        self.processes.contains_key(terminal_id)
    }

    pub fn write(&mut self, terminal_id: &str, data: &[u8]) -> Result<(), String> {
        let process = self
            .processes
            .get_mut(terminal_id)
            .ok_or_else(|| "Local terminal is no longer running".to_owned())?;
        process
            .writer
            .write_all(data)
            .and_then(|()| process.writer.flush())
            .map_err(|error| format!("Unable to write local terminal: {error}"))
    }

    pub fn resize(&self, terminal_id: &str, cols: usize, rows: usize) -> Result<(), String> {
        let process = self
            .processes
            .get(terminal_id)
            .ok_or_else(|| "Local terminal is no longer running".to_owned())?;
        process
            .master
            .resize(pty_size(cols, rows))
            .map_err(|error| format!("Unable to resize local terminal: {error}"))
    }

    pub fn close(&mut self, terminal_id: &str) -> Result<(), String> {
        let mut process = self
            .processes
            .remove(terminal_id)
            .ok_or_else(|| "Local terminal is no longer running".to_owned())?;
        process
            .killer
            .kill()
            .map_err(|error| format!("Unable to close local terminal: {error}"))
    }

    pub fn reap(&mut self, terminal_id: &str) {
        self.processes.remove(terminal_id);
    }
}

impl Drop for LocalTerminalHost {
    fn drop(&mut self) {
        for process in self.processes.values_mut() {
            let _ = process.killer.kill();
        }
    }
}

fn pty_size(cols: usize, rows: usize) -> PtySize {
    PtySize {
        cols: u16::try_from(cols.max(1)).unwrap_or(u16::MAX),
        rows: u16::try_from(rows.max(1)).unwrap_or(u16::MAX),
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn preferred_shell() -> PathBuf {
    ["KIMINI_SHELL_PATH", "KIMI_SHELL_PATH"]
        .into_iter()
        .find_map(|name| {
            std::env::var_os(name)
                .map(PathBuf::from)
                .filter(|shell| shell.is_file())
        })
        .or_else(platform_shell)
        .unwrap_or_else(default_shell)
}

#[cfg(unix)]
fn platform_shell() -> Option<PathBuf> {
    std::env::var_os("SHELL")
        .map(PathBuf::from)
        .filter(|shell| shell.is_file())
}

#[cfg(windows)]
fn platform_shell() -> Option<PathBuf> {
    which::which("pwsh")
        .or_else(|_| which::which("powershell"))
        .ok()
        .or_else(|| {
            std::env::var_os("COMSPEC")
                .map(PathBuf::from)
                .filter(|shell| shell.is_file())
        })
}

#[cfg(unix)]
fn default_shell() -> PathBuf {
    ["/bin/zsh", "/bin/bash", "/bin/sh"]
        .into_iter()
        .map(PathBuf::from)
        .find(|shell| shell.is_file())
        .unwrap_or_else(|| PathBuf::from("/bin/sh"))
}

#[cfg(windows)]
fn default_shell() -> PathBuf {
    PathBuf::from(r"C:\Windows\System32\cmd.exe")
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::time::{Duration, Instant};

    use super::*;

    #[cfg(unix)]
    #[test]
    fn local_pty_executes_commands_and_reports_exit() {
        let mut host = LocalTerminalHost::default();
        let terminal = host
            .spawn_with_shell(
                "session",
                &std::env::temp_dir(),
                Path::new("/bin/sh"),
                80,
                24,
            )
            .unwrap();
        host.write(&terminal.id, b"printf 'KIMINI_LOCAL_PTY_OK\\n'\nexit\n")
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut output = Vec::new();
        let mut exited = false;
        while Instant::now() < deadline && !exited {
            match host.events_rx.try_recv() {
                Ok(LocalTerminalEvent::Output { data, .. }) => output.extend(data),
                Ok(LocalTerminalEvent::Exit { exit_code, .. }) => {
                    assert_eq!(exit_code, Some(0));
                    exited = true;
                }
                Err(async_channel::TryRecvError::Empty) => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(async_channel::TryRecvError::Closed) => break,
            }
        }

        assert!(exited, "local shell did not exit before the deadline");
        assert!(String::from_utf8_lossy(&output).contains("KIMINI_LOCAL_PTY_OK"));
        host.reap(&terminal.id);
    }

    #[test]
    fn pty_dimensions_are_never_zero_and_saturate_safely() {
        assert_eq!(pty_size(0, 0).cols, 1);
        assert_eq!(pty_size(usize::MAX, usize::MAX).rows, u16::MAX);
    }
}
