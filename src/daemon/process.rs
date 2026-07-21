use std::env;
#[cfg(unix)]
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

pub(super) fn resolve_kimi() -> Option<PathBuf> {
    which::which("kimi")
        .ok()
        .or_else(resolve_known_location)
        .or_else(login_shell_which)
}

fn resolve_known_location() -> Option<PathBuf> {
    let home = dirs::home_dir();
    let executable = format!("kimi{}", env::consts::EXE_SUFFIX);
    let candidates = home
        .iter()
        .flat_map(|home| {
            [
                home.join(".kimi-code/bin").join(&executable),
                home.join(".local/bin").join(&executable),
                home.join(".bun/bin").join(&executable),
                home.join(".volta/bin").join(&executable),
            ]
        })
        .collect::<Vec<_>>();
    #[cfg(unix)]
    let candidates = candidates
        .into_iter()
        .chain(["/opt/homebrew/bin/kimi", "/usr/local/bin/kimi"].map(PathBuf::from))
        .collect::<Vec<_>>();
    candidates
        .into_iter()
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(unix)]
fn login_shell_which() -> Option<PathBuf> {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let output = Command::new(shell)
        .args(["-lc", "command -v kimi"])
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let path = stdout
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| line.starts_with('/'))
        .map(PathBuf::from)?;
    is_executable(&path).then_some(path)
}

#[cfg(not(unix))]
fn login_shell_which() -> Option<PathBuf> {
    None
}

/// Start a shared local Kimi Code server when none is healthy.
///
/// kimi-code 0.28+ replaced `kimi server run` with foreground-only
/// `kimi web`. We pass `--no-open` so Kimini owns the UI (Native or Web shell)
/// and put the child in its own process group so it can outlive either app —
/// Native and Web share the same REST/WS origin and token.
pub(super) fn spawn_daemon(kimi: &Path) -> std::io::Result<()> {
    let mut command = Command::new(kimi);
    command
        .args(["web", "--no-open", "--log-level", "error"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // Detach from Kimini's process group so quitting Kimini.app / Kimini Web.app
    // does not SIGHUP the shared server the other client may still be using.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    let mut child = command.spawn()?;
    thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
