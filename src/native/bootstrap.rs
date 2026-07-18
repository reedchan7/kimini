use std::sync::atomic::AtomicBool;

use crate::api::KimiClient;
use crate::daemon::{Connection, discover_connection};
use crate::protocol::{Session, SessionSnapshot};

pub(super) struct Bootstrap {
    pub connection: Connection,
    pub sessions: Vec<Session>,
    pub snapshot: Option<SessionSnapshot>,
}

pub(super) fn load() -> Result<Bootstrap, String> {
    let stop = AtomicBool::new(false);
    let connection = discover_connection(&stop, &|_| {})
        .ok_or_else(|| "Kimi daemon discovery was cancelled".to_owned())?;
    let client = KimiClient::new(connection.clone());
    let sessions = client
        .list_sessions()
        .map_err(|error| error.to_string())?
        .items;
    let snapshot = sessions
        .iter()
        .find(|session| !session.archived)
        .map(|session| client.snapshot(&session.id))
        .transpose()
        .map_err(|error| error.to_string())?;
    Ok(Bootstrap {
        connection,
        sessions,
        snapshot,
    })
}
