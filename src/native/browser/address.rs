use url::Url;

pub(in crate::native) fn normalize_address(input: &str) -> Result<String, &'static str> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Enter a web address");
    }
    if input == "about:blank" {
        return Ok(input.into());
    }

    if let Ok(url) = Url::parse(input) {
        return supported_url(url);
    }

    let url = Url::parse(&format!("https://{input}"))
        .map_err(|_| "Enter a valid HTTP or HTTPS address")?;
    supported_url(url)
}

fn supported_url(url: Url) -> Result<String, &'static str> {
    if matches!(url.scheme(), "http" | "https") && url.host().is_some() {
        Ok(url.into())
    } else {
        Err("Only HTTP and HTTPS addresses are supported")
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_address;

    #[test]
    fn preserves_supported_urls() {
        assert_eq!(
            normalize_address("https://example.com/path?q=1").unwrap(),
            "https://example.com/path?q=1"
        );
        assert_eq!(normalize_address("about:blank").unwrap(), "about:blank");
    }

    #[test]
    fn adds_https_to_host_names() {
        assert_eq!(
            normalize_address("example.com/docs").unwrap(),
            "https://example.com/docs"
        );
    }

    #[test]
    fn rejects_empty_and_privileged_schemes() {
        assert!(normalize_address("  ").is_err());
        assert!(normalize_address("file:///etc/passwd").is_err());
        assert!(normalize_address("javascript:alert(1)").is_err());
    }
}
