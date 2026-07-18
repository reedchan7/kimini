use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

pub(super) fn resolve_kimi() -> Option<PathBuf> {
    let from_path = env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|directory| directory.join("kimi"))
            .find(|candidate| is_executable(candidate))
    });
    from_path
        .or_else(resolve_known_location)
        .or_else(login_shell_which)
}

fn resolve_known_location() -> Option<PathBuf> {
    let home = env::var_os("HOME").map(PathBuf::from);
    let mut candidates = home
        .iter()
        .flat_map(|home| {
            [
                home.join(".kimi-code/bin/kimi"),
                home.join(".local/bin/kimi"),
                home.join(".bun/bin/kimi"),
                home.join(".volta/bin/kimi"),
            ]
        })
        .collect::<Vec<_>>();
    candidates.extend(["/opt/homebrew/bin/kimi", "/usr/local/bin/kimi"].map(PathBuf::from));
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

pub(super) fn spawn_daemon(kimi: &Path) -> std::io::Result<()> {
    let mut child = Command::new(kimi)
        .args(["server", "run", "--log-level", "error"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
