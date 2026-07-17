//! Zero-config discovery of the local kimi-code daemon.
//!
//! Mirrors the official desktop app's protocol:
//! - `$KIMI_CODE_HOME` (default `~/.kimi-code`) is the daemon's home.
//! - `server/lock` — JSON `{pid, host?, port, …}` written by the live daemon.
//!   The recorded port is authoritative: when the default port is busy the
//!   server retries on port+1 and rewrites the lock.
//! - `server.token` — persistent bearer token (0600). It rides only in the
//!   URL fragment (`#token=…`), which never reaches the server or the logs.
//! - `GET /api/v1/healthz` (unauthenticated) answers `{"code":0}` when live.
//! - Cold start: `kimi server run` reuses or spawns the shared daemon and
//!   exits once it is healthy; the daemon self-exits ~60s after its last
//!   client disconnects, so the shell never manages its lifetime.

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use url::Url;

/// Fallback origin probed when no lock file exists (default daemon port).
const DEFAULT_ORIGIN: &str = "http://127.0.0.1:58627";
const HEALTHZ_TIMEOUT: Duration = Duration::from_millis(1500);
/// How long to fast-poll after `kimi server run` before assuming it failed.
const SPAWN_WINDOW: Duration = Duration::from_secs(30);
const FAST_POLL: Duration = Duration::from_millis(500);
const SLOW_POLL: Duration = Duration::from_secs(2);

/// Launch-page status updates emitted while discovering.
#[derive(Debug, Clone, Copy)]
pub enum Status {
    StartingDaemon,
    WaitingForDaemon,
    KimiNotFound,
}

impl Status {
    /// Key understood by the launch page's `__kiminiStatus` (see `i18n::launch_html`).
    pub const fn key(self) -> &'static str {
        match self {
            Self::StartingDaemon => "starting",
            Self::WaitingForDaemon => "waiting",
            Self::KimiNotFound => "noKimi",
        }
    }
}

/// Blocking discovery loop; run on a background thread.
///
/// Probes for a live daemon, starting one via `kimi server run` when none is
/// found, and returns the web UI URL to load. Never gives up on its own — the
/// user may install or start `kimi` at any time while the launch page shows —
/// but exits with `None` once `stop` is set (manual connect won the race).
pub fn discover(stop: &AtomicBool, notify: &dyn Fn(Status)) -> Option<String> {
    if let Some(url) = probe() {
        return Some(url);
    }

    let spawned = match resolve_kimi() {
        Some(kimi) => {
            notify(Status::StartingDaemon);
            spawn_daemon(&kimi)
                .map_err(|e| eprintln!("kimini: failed to run kimi server run: {e}"))
                .is_ok()
        }
        None => false,
    };
    if !spawned {
        notify(Status::KimiNotFound);
    }

    let fast_deadline = Instant::now() + SPAWN_WINDOW;
    let mut announced_wait = false;
    loop {
        if stop.load(Ordering::Relaxed) {
            return None;
        }
        if let Some(url) = probe() {
            return Some(url);
        }
        let fast = spawned && Instant::now() < fast_deadline;
        if spawned && !fast && !announced_wait {
            announced_wait = true;
            notify(Status::WaitingForDaemon);
        }
        thread::sleep(if fast { FAST_POLL } else { SLOW_POLL });
    }
}

/// One discovery pass: lock origin first (authoritative port), then the
/// default port as a backstop for an unreadable lock.
fn probe() -> Option<String> {
    let mut origins: Vec<String> = Vec::new();
    if let Some(origin) = read_lock_origin() {
        origins.push(origin);
    }
    if !origins.iter().any(|o| o == DEFAULT_ORIGIN) {
        origins.push(DEFAULT_ORIGIN.to_string());
    }
    origins
        .into_iter()
        .find(|origin| is_healthy(origin))
        .map(|origin| web_url(&origin, read_token().as_deref()))
}

/// `$KIMI_CODE_HOME` or `~/.kimi-code` — must match the daemon's own resolution.
fn kimi_home() -> Option<PathBuf> {
    if let Some(dir) = env::var_os("KIMI_CODE_HOME")
        && !dir.is_empty()
    {
        return Some(PathBuf::from(dir));
    }
    Some(PathBuf::from(env::var_os("HOME")?).join(".kimi-code"))
}

/// Origin recorded in `server/lock`, or `None` when the file is missing or
/// unparseable. Liveness is NOT checked here — that's `is_healthy`'s job.
fn read_lock_origin() -> Option<String> {
    let raw = fs::read_to_string(kimi_home()?.join("server").join("lock")).ok()?;
    parse_lock_origin(&raw)
}

fn parse_lock_origin(raw: &str) -> Option<String> {
    let lock: serde_json::Value = serde_json::from_str(raw).ok()?;
    let port = u16::try_from(lock.get("port")?.as_u64()?).ok()?;
    let host = match lock.get("host").and_then(|h| h.as_str()) {
        // A wildcard bind is reachable via loopback.
        Some(h) if !h.is_empty() && h != "0.0.0.0" => h,
        _ => "127.0.0.1",
    };
    Some(format!("http://{host}:{port}"))
}

/// The persistent bearer token, or `None` when unreadable (the web UI then
/// falls back to its own manual token dialog).
fn read_token() -> Option<String> {
    let raw = fs::read_to_string(kimi_home()?.join("server.token")).ok()?;
    let token = raw.trim();
    (!token.is_empty()).then(|| token.to_string())
}

/// Compose the web UI URL. The token (base64url, URL-safe) rides in the
/// fragment: kimi-web reads `location.hash` and persists it to localStorage.
fn web_url(origin: &str, token: Option<&str>) -> String {
    match token {
        Some(token) => format!("{origin}/#token={token}"),
        None => format!("{origin}/"),
    }
}

/// `GET /api/v1/healthz` over a raw loopback socket — expects `{"code":0}`.
/// HTTP/1.0 keeps the reply un-chunked so the body is plain JSON.
fn is_healthy(origin: &str) -> bool {
    let Ok(url) = Url::parse(origin) else {
        return false;
    };
    let Some(host) = url.host_str().map(str::to_string) else {
        return false;
    };
    let Some(port) = url.port_or_known_default() else {
        return false;
    };
    let Ok(mut addrs) = (host.as_str(), port).to_socket_addrs() else {
        return false;
    };
    let Some(addr) = addrs.next() else {
        return false;
    };
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, HEALTHZ_TIMEOUT) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(HEALTHZ_TIMEOUT));
    let _ = stream.set_write_timeout(Some(HEALTHZ_TIMEOUT));
    let request = format!(
        "GET /api/v1/healthz HTTP/1.0\r\nHost: {host}:{port}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let mut response = String::new();
    if stream.read_to_string(&mut response).is_err() {
        return false;
    }
    healthz_ok(&response)
}

fn healthz_ok(response: &str) -> bool {
    let status_line = response.lines().next().unwrap_or("");
    if !status_line.contains(" 200") {
        return false;
    }
    let body = response.split("\r\n\r\n").nth(1).unwrap_or("");
    serde_json::from_str::<serde_json::Value>(body.trim())
        .ok()
        .and_then(|v| v.get("code").and_then(serde_json::Value::as_i64))
        == Some(0)
}

/// Locate the `kimi` CLI. Finder-launched apps get a minimal `PATH`, so after
/// `$PATH` we try the standard install locations, then ask a login shell.
fn resolve_kimi() -> Option<PathBuf> {
    if let Some(paths) = env::var_os("PATH") {
        for dir in env::split_paths(&paths) {
            let candidate = dir.join("kimi");
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }

    let home = env::var_os("HOME").map(PathBuf::from);
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(home) = &home {
        // kimi's self-managed install lives inside its own home dir.
        candidates.push(home.join(".kimi-code").join("bin").join("kimi"));
        candidates.push(home.join(".local").join("bin").join("kimi"));
        candidates.push(home.join(".bun").join("bin").join("kimi"));
        candidates.push(home.join(".volta").join("bin").join("kimi"));
    }
    candidates.push(PathBuf::from("/opt/homebrew/bin/kimi"));
    candidates.push(PathBuf::from("/usr/local/bin/kimi"));
    if let Some(found) = candidates.into_iter().find(|c| is_executable(c)) {
        return Some(found);
    }

    login_shell_which()
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// Last resort: a login shell knows the user's full `PATH` (nvm, fnm, …).
#[cfg(unix)]
fn login_shell_which() -> Option<PathBuf> {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let output = Command::new(shell)
        .args(["-lc", "command -v kimi"])
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    // Profiles may echo noise before the result; the path is the last line.
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

/// Run `kimi server run` — it reuses a live shared daemon or spawns one, then
/// exits once it is healthy. Fire-and-forget: the discovery loop watches
/// healthz; a reaper thread prevents a zombie child.
fn spawn_daemon(kimi: &Path) -> std::io::Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_origin_uses_recorded_host_and_port() {
        let raw = r#"{"pid":1,"started_at":"t","host":"localhost","port":58628}"#;
        assert_eq!(
            parse_lock_origin(raw),
            Some("http://localhost:58628".to_string())
        );
    }

    #[test]
    fn lock_origin_maps_wildcard_and_missing_host_to_loopback() {
        let wildcard = r#"{"pid":1,"started_at":"t","host":"0.0.0.0","port":58627}"#;
        let missing = r#"{"pid":1,"started_at":"t","port":58627}"#;
        for raw in [wildcard, missing] {
            assert_eq!(
                parse_lock_origin(raw),
                Some("http://127.0.0.1:58627".to_string())
            );
        }
    }

    #[test]
    fn lock_origin_rejects_garbage() {
        assert_eq!(parse_lock_origin(""), None);
        assert_eq!(parse_lock_origin("{}"), None);
        assert_eq!(parse_lock_origin(r#"{"port":"58627"}"#), None);
    }

    #[test]
    fn web_url_places_token_in_fragment() {
        assert_eq!(
            web_url("http://127.0.0.1:58627", Some("abc")),
            "http://127.0.0.1:58627/#token=abc"
        );
        assert_eq!(web_url("http://127.0.0.1:58627", None), "http://127.0.0.1:58627/");
    }

    #[test]
    fn healthz_requires_200_and_code_zero() {
        let ok = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"code\":0,\"data\":{}}";
        let unauthorized = "HTTP/1.1 401 Unauthorized\r\n\r\n{\"code\":40101}";
        let bad_code = "HTTP/1.1 200 OK\r\n\r\n{\"code\":1}";
        assert!(healthz_ok(ok));
        assert!(!healthz_ok(unauthorized));
        assert!(!healthz_ok(bad_code));
        assert!(!healthz_ok(""));
    }
}
