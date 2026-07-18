use url::Url;

pub fn explicit_url(arg: Option<&str>, env_url: Option<&str>) -> Option<String> {
    arg.filter(|url| !url.is_empty())
        .or_else(|| env_url.filter(|url| !url.is_empty()))
        .map(str::to_owned)
}

pub fn is_loopback(raw: &str) -> bool {
    let Ok(url) = Url::parse(raw) else {
        return false;
    };
    if url.scheme() == "about" {
        return true;
    }
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain(domain)) => {
            domain.eq_ignore_ascii_case("localhost")
                || domain.to_ascii_lowercase().ends_with(".localhost")
        }
        None => false,
    }
}

pub fn origin_for_log(raw: &str) -> String {
    Url::parse(raw)
        .ok()
        .and_then(|url| {
            url.host_str().map(|host| {
                format!(
                    "{}://{}:{}",
                    url.scheme(),
                    host,
                    url.port_or_known_default().unwrap_or(0)
                )
            })
        })
        .unwrap_or_else(|| "<invalid url>".to_string())
}
