use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use kimini::api::{EventSocket, KimiClient, SocketEvent};
use kimini::daemon::discover_connection;

#[test]
#[ignore = "requires the locally installed Kimi Code daemon"]
fn loads_sessions_and_an_atomic_snapshot_from_the_real_daemon() {
    let stop = AtomicBool::new(false);
    let connection = discover_connection(&stop, &|_| {}).expect("local Kimi Code daemon");
    let client = KimiClient::new(connection.clone());
    let sessions = client.list_sessions().expect("session list");

    if let Some(session) = sessions.items.first() {
        let snapshot = client.snapshot(&session.id).expect("session snapshot");
        assert_eq!(snapshot.session.id, session.id);
        assert!(!snapshot.epoch.is_empty());
        assert_eq!(snapshot.cursor().seq, snapshot.as_of_seq);

        let socket = EventSocket::connect(
            connection,
            "kimini-live-test",
            [(session.id.clone(), snapshot.cursor())].into(),
        );
        let events = socket.events();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match events.try_recv() {
                Ok(SocketEvent::Connected) => break,
                Ok(SocketEvent::Error { message, .. }) => {
                    panic!("WebSocket connection: {message}")
                }
                Ok(_) | Err(async_channel::TryRecvError::Empty) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Ok(_) | Err(async_channel::TryRecvError::Empty) => {
                    panic!("WebSocket handshake timed out")
                }
                Err(async_channel::TryRecvError::Closed) => {
                    panic!("WebSocket closed during handshake")
                }
            }
        }
    }
}
