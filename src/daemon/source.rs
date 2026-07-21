use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_ORIGIN: &str = "http://127.0.0.1:58627";

pub(super) fn candidate_origins() -> Vec<String> {
    let mut origins = Vec::new();
    for origin in instance_origins() {
        push_unique(&mut origins, origin);
    }
    // Pre-0.28 Kimi Code wrote a single lock file; keep reading it for mixed installs.
    if let Some(origin) = read_legacy_lock_origin() {
        push_unique(&mut origins, origin);
    }
    push_unique(&mut origins, DEFAULT_ORIGIN.into());
    origins
}

pub(super) fn read_token() -> Option<String> {
    let raw = fs::read_to_string(kimi_home()?.join("server.token")).ok()?;
    let token = raw.trim();
    (!token.is_empty()).then(|| token.to_owned())
}

/// kimi-code 0.28+ records each foreground server under `server/instances/*.json`.
fn instance_origins() -> Vec<String> {
    let dir = match kimi_home() {
        Some(home) => home.join("server").join("instances"),
        None => return Vec::new(),
    };
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut records = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Ok(raw) = fs::read_to_string(path) else {
            continue;
        };
        if let Some(record) = parse_instance_record(&raw) {
            records.push(record);
        }
    }

    // Prefer the instance with the freshest heartbeat so a live multi-port
    // setup probes the active server first.
    records.sort_by(|left, right| right.heartbeat_at.cmp(&left.heartbeat_at));
    records.into_iter().map(|record| record.origin).collect()
}

struct InstanceRecord {
    origin: String,
    heartbeat_at: u64,
}

fn parse_instance_record(raw: &str) -> Option<InstanceRecord> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    let port = u16::try_from(value.get("port")?.as_u64()?).ok()?;
    let host = match value.get("host").and_then(serde_json::Value::as_str) {
        Some(host) if !host.is_empty() && host != "0.0.0.0" => host,
        _ => "127.0.0.1",
    };
    let heartbeat_at = value
        .get("heartbeat_at")
        .and_then(serde_json::Value::as_u64)
        .or_else(|| value.get("started_at").and_then(serde_json::Value::as_u64))
        .unwrap_or(0);
    Some(InstanceRecord {
        origin: format!("http://{host}:{port}"),
        heartbeat_at,
    })
}

fn read_legacy_lock_origin() -> Option<String> {
    let raw = fs::read_to_string(kimi_home()?.join("server").join("lock")).ok()?;
    parse_legacy_lock_origin(&raw)
}

fn kimi_home() -> Option<PathBuf> {
    if let Some(directory) = env::var_os("KIMI_CODE_HOME")
        && !directory.is_empty()
    {
        return Some(PathBuf::from(directory));
    }
    Some(dirs::home_dir()?.join(".kimi-code"))
}

fn parse_legacy_lock_origin(raw: &str) -> Option<String> {
    let lock: serde_json::Value = serde_json::from_str(raw).ok()?;
    let port = u16::try_from(lock.get("port")?.as_u64()?).ok()?;
    let host = match lock.get("host").and_then(serde_json::Value::as_str) {
        Some(host) if !host.is_empty() && host != "0.0.0.0" => host,
        _ => "127.0.0.1",
    };
    Some(format!("http://{host}:{port}"))
}

fn push_unique(origins: &mut Vec<String>, origin: String) {
    if !origins.iter().any(|existing| existing == &origin) {
        origins.push(origin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_record_uses_host_port_and_heartbeat() {
        let raw = r#"{"server_id":"abc","pid":1,"host":"localhost","port":58628,"started_at":10,"heartbeat_at":20,"host_version":"0.28.1"}"#;
        let record = parse_instance_record(raw).expect("instance");
        assert_eq!(record.origin, "http://localhost:58628");
        assert_eq!(record.heartbeat_at, 20);
    }

    #[test]
    fn instance_wildcard_host_uses_loopback() {
        let raw = r#"{"port":58627,"host":"0.0.0.0","started_at":1}"#;
        let record = parse_instance_record(raw).expect("instance");
        assert_eq!(record.origin, DEFAULT_ORIGIN);
        assert_eq!(record.heartbeat_at, 1);
    }

    #[test]
    fn invalid_instance_data_is_rejected() {
        for raw in ["", "{}", r#"{"port":"58627"}"#, r#"{"host":"127.0.0.1"}"#] {
            assert_eq!(parse_instance_record(raw).map(|r| r.origin), None);
        }
    }

    #[test]
    fn lock_origin_uses_recorded_host_and_port() {
        let raw = r#"{"pid":1,"host":"localhost","port":58628}"#;
        assert_eq!(
            parse_legacy_lock_origin(raw).as_deref(),
            Some("http://localhost:58628")
        );
    }

    #[test]
    fn wildcard_and_missing_hosts_use_loopback() {
        for raw in [r#"{"host":"0.0.0.0","port":58627}"#, r#"{"port":58627}"#] {
            assert_eq!(
                parse_legacy_lock_origin(raw).as_deref(),
                Some(DEFAULT_ORIGIN)
            );
        }
    }

    #[test]
    fn invalid_lock_data_is_rejected() {
        for raw in ["", "{}", r#"{"port":"58627"}"#] {
            assert_eq!(parse_legacy_lock_origin(raw), None);
        }
    }

    #[test]
    fn push_unique_preserves_order() {
        let mut origins = Vec::new();
        push_unique(&mut origins, "http://127.0.0.1:58628".into());
        push_unique(&mut origins, "http://127.0.0.1:58627".into());
        push_unique(&mut origins, "http://127.0.0.1:58628".into());
        assert_eq!(
            origins,
            vec![
                "http://127.0.0.1:58628".to_string(),
                "http://127.0.0.1:58627".to_string(),
            ]
        );
    }
}
