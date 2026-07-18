use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use url::Url;

const TIMEOUT: Duration = Duration::from_millis(1500);

pub(super) fn is_healthy(origin: &str) -> bool {
    let Some((host, port)) = endpoint(origin) else {
        return false;
    };
    let Ok(mut addrs) = (host.as_str(), port).to_socket_addrs() else {
        return false;
    };
    let Some(address) = addrs.next() else {
        return false;
    };
    let Ok(mut stream) = TcpStream::connect_timeout(&address, TIMEOUT) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(TIMEOUT));
    let _ = stream.set_write_timeout(Some(TIMEOUT));
    let request = format!(
        "GET /api/v1/healthz HTTP/1.0\r\nHost: {host}:{port}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
    );
    let mut response = String::new();
    stream.write_all(request.as_bytes()).is_ok()
        && stream.read_to_string(&mut response).is_ok()
        && healthz_ok(&response)
}

fn endpoint(origin: &str) -> Option<(String, u16)> {
    let url = Url::parse(origin).ok()?;
    let host = url.host_str()?.to_owned();
    let port = url.port_or_known_default()?;
    Some((host, port))
}

fn healthz_ok(response: &str) -> bool {
    let status_line = response.lines().next().unwrap_or_default();
    let body = response.split("\r\n\r\n").nth(1).unwrap_or_default();
    status_line.contains(" 200")
        && serde_json::from_str::<serde_json::Value>(body.trim())
            .ok()
            .and_then(|value| value.get("code")?.as_i64())
            == Some(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_requires_a_complete_http_origin() {
        assert_eq!(
            endpoint("http://localhost:58627"),
            Some(("localhost".into(), 58627))
        );
        assert_eq!(endpoint("not a url"), None);
    }

    #[test]
    fn response_requires_success_and_code_zero() {
        let ok = "HTTP/1.1 200 OK\r\n\r\n{\"code\":0}";
        let bad_status = "HTTP/1.1 401 Unauthorized\r\n\r\n{\"code\":0}";
        let bad_body = "HTTP/1.1 200 OK\r\n\r\n{\"code\":1}";

        assert!(healthz_ok(ok));
        assert!(!healthz_ok(bad_status));
        assert!(!healthz_ok(bad_body));
        assert!(!healthz_ok(""));
    }
}
