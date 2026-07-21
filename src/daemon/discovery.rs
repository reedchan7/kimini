use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use super::Connection;
use super::health::is_healthy;
use super::process::{resolve_kimi, spawn_daemon};
use super::source::{candidate_origins, read_token};

const SPAWN_WINDOW: Duration = Duration::from_secs(30);
const FAST_POLL: Duration = Duration::from_millis(500);
const SLOW_POLL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    StartingDaemon,
    WaitingForDaemon,
    KimiNotFound,
}

impl Status {
    pub const fn key(self) -> &'static str {
        match self {
            Self::StartingDaemon => "starting",
            Self::WaitingForDaemon => "waiting",
            Self::KimiNotFound => "noKimi",
        }
    }
}

/// Discover a healthy local Kimi Code server, starting one when needed.
///
/// Used by both Native and Web: attach to an existing healthy instance first;
/// only spawn when every candidate fails the health probe.
pub fn discover_connection(stop: &AtomicBool, notify: &dyn Fn(Status)) -> Option<Connection> {
    if let Some(connection) = probe() {
        return Some(connection);
    }

    let spawned = match resolve_kimi() {
        Some(kimi) => {
            notify(Status::StartingDaemon);
            spawn_daemon(&kimi)
                .map_err(|error| eprintln!("kimini: failed to start Kimi Code: {error}"))
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
        if let Some(connection) = probe() {
            return Some(connection);
        }
        let fast = spawned && Instant::now() < fast_deadline;
        if spawned && !fast && !announced_wait {
            announced_wait = true;
            notify(Status::WaitingForDaemon);
        }
        thread::sleep(if fast { FAST_POLL } else { SLOW_POLL });
    }
}

fn probe() -> Option<Connection> {
    candidate_origins()
        .into_iter()
        .find(|origin| is_healthy(origin))
        .map(|origin| Connection::new(origin, read_token()))
}
