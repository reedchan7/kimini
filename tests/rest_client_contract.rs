use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use kimini::api::KimiClient;
use kimini::daemon::Connection;

#[test]
fn unwraps_the_daemon_envelope_and_sends_native_identity() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"items":[],"has_more":false},"request_id":"req_01"}"#,
    );
    let client = KimiClient::new(Connection::new(origin, Some("secret".into())));

    let page = client.list_sessions().unwrap();
    let request = request.join().unwrap();

    assert!(page.items.is_empty());
    assert!(request.starts_with("GET /api/v1/sessions?page_size=100&include_archive=false"));
    assert!(request.contains("Authorization: Bearer secret"));
    assert!(request.contains("X-Kimi-Client-Ui-Mode: native"));
}

#[test]
fn posts_prompt_content_to_the_session_route() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"prompt_id":"prompt_01","user_message_id":"msg_01","status":"running"},"request_id":"req_01"}"#,
    );
    let client = KimiClient::new(Connection::new(origin, None));

    let result = client.submit_prompt("sess/a", "hello").unwrap();
    let request = request.join().unwrap();

    assert_eq!(result.prompt_id, "prompt_01");
    assert!(request.starts_with("POST /api/v1/sessions/sess%2Fa/prompts"));
    assert!(request.contains(r#"{"content":[{"type":"text","text":"hello"}]}"#));
}

#[test]
fn surfaces_daemon_and_shape_errors_without_credentials() {
    let (origin, _) =
        one_response(r#"{"code":40901,"msg":"busy","data":null,"request_id":"req_01"}"#);
    let error = KimiClient::new(Connection::new(origin, None))
        .list_sessions()
        .unwrap_err();
    assert_eq!(error.to_string(), "Kimi daemon error 40901: busy");

    let (origin, _) = one_response(r#"{"code":0,"msg":"","data":null,"request_id":"req_02"}"#);
    let error = KimiClient::new(Connection::new(origin, None))
        .list_sessions()
        .unwrap_err();
    assert_eq!(error.to_string(), "Kimi daemon returned no data");

    let (origin, _) = one_response("not-json");
    let error = KimiClient::new(Connection::new(origin, None))
        .list_sessions()
        .unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("Invalid Kimi daemon response:")
    );
}

#[test]
fn question_answers_use_the_real_item_identifier_as_the_json_key() {
    let (origin, request) = one_response(r#"{"code":0,"msg":"","data":{},"request_id":"req_01"}"#);
    let client = KimiClient::new(Connection::new(origin, None));

    client
        .resolve_question("sess_01", "question_01", "item/1", "option_01")
        .unwrap();
    let request = request.join().unwrap();

    assert!(request.contains(
        r#"{"answers":{"item/1":{"kind":"single","option_id":"option_01"}},"method":"click"}"#
    ));
}

fn one_response(body: &'static str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        loop {
            let mut buffer = [0_u8; 4096];
            let length = stream.read(&mut buffer).unwrap();
            if length == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..length]);
            if request_complete(&request) {
                break;
            }
        }
        let request = String::from_utf8_lossy(&request).into_owned();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
        request
    });
    (format!("http://{address}"), handle)
}

fn request_complete(request: &[u8]) -> bool {
    let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
        return false;
    };
    let headers = String::from_utf8_lossy(&request[..header_end]).to_ascii_lowercase();
    let content_length = headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length:"))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    request.len() >= header_end + 4 + content_length
}
