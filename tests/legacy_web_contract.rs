#![cfg(feature = "legacy-web")]

use kimini::legacy_web::navigation::{explicit_url, is_loopback, origin_for_log};

#[test]
fn command_line_url_wins_and_empty_values_fall_back() {
    assert_eq!(
        explicit_url(
            Some("http://localhost:58628"),
            Some("http://localhost:58627")
        ),
        Some("http://localhost:58628".to_string())
    );
    assert_eq!(
        explicit_url(Some(""), Some("http://localhost:58627")),
        Some("http://localhost:58627".to_string())
    );
    assert_eq!(explicit_url(Some(""), Some("")), None);
}

#[test]
fn embedded_navigation_accepts_only_loopback_http_origins() {
    for url in [
        "about:blank",
        "http://127.0.0.1:58627/",
        "http://[::1]:58627/",
        "https://localhost/",
        "http://api.localhost/",
    ] {
        assert!(is_loopback(url), "expected loopback URL: {url}");
    }

    for url in [
        "https://kimi.com/",
        "http://localhost.example/",
        "file:///tmp/index.html",
        "not a url",
    ] {
        assert!(!is_loopback(url), "expected external URL: {url}");
    }
}

#[test]
fn logged_origin_never_contains_path_or_token_fragment() {
    assert_eq!(
        origin_for_log("http://127.0.0.1:58627/chat#token=secret"),
        "http://127.0.0.1:58627"
    );
    assert_eq!(origin_for_log("not a url"), "<invalid url>");
}
