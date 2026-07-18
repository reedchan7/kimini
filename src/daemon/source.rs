use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_ORIGIN: &str = "http://127.0.0.1:58627";

pub(super) fn candidate_origins() -> Vec<String> {
    let lock_origin = read_lock_origin();
    let mut origins = lock_origin.into_iter().collect::<Vec<_>>();
    if !origins.iter().any(|origin| origin == DEFAULT_ORIGIN) {
        origins.push(DEFAULT_ORIGIN.into());
    }
    origins
}

pub(super) fn read_token() -> Option<String> {
    let raw = fs::read_to_string(kimi_home()?.join("server.token")).ok()?;
    let token = raw.trim();
    (!token.is_empty()).then(|| token.to_owned())
}

fn read_lock_origin() -> Option<String> {
    let raw = fs::read_to_string(kimi_home()?.join("server").join("lock")).ok()?;
    parse_lock_origin(&raw)
}

fn kimi_home() -> Option<PathBuf> {
    if let Some(directory) = env::var_os("KIMI_CODE_HOME")
        && !directory.is_empty()
    {
        return Some(PathBuf::from(directory));
    }
    Some(PathBuf::from(env::var_os("HOME")?).join(".kimi-code"))
}

fn parse_lock_origin(raw: &str) -> Option<String> {
    let lock: serde_json::Value = serde_json::from_str(raw).ok()?;
    let port = u16::try_from(lock.get("port")?.as_u64()?).ok()?;
    let host = match lock.get("host").and_then(serde_json::Value::as_str) {
        Some(host) if !host.is_empty() && host != "0.0.0.0" => host,
        _ => "127.0.0.1",
    };
    Some(format!("http://{host}:{port}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_origin_uses_recorded_host_and_port() {
        let raw = r#"{"pid":1,"host":"localhost","port":58628}"#;
        assert_eq!(
            parse_lock_origin(raw).as_deref(),
            Some("http://localhost:58628")
        );
    }

    #[test]
    fn wildcard_and_missing_hosts_use_loopback() {
        for raw in [r#"{"host":"0.0.0.0","port":58627}"#, r#"{"port":58627}"#] {
            assert_eq!(parse_lock_origin(raw).as_deref(), Some(DEFAULT_ORIGIN));
        }
    }

    #[test]
    fn invalid_lock_data_is_rejected() {
        for raw in ["", "{}", r#"{"port":"58627"}"#] {
            assert_eq!(parse_lock_origin(raw), None);
        }
    }
}
