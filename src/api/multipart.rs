pub(super) fn file_body(boundary: &str, name: &str, media_type: &str, bytes: &[u8]) -> Vec<u8> {
    let safe_name = header_value(name);
    let mut body = Vec::with_capacity(bytes.len() + 512);
    append(
        &mut body,
        &format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{safe_name}\"\r\nContent-Type: {media_type}\r\n\r\n"
        ),
    );
    body.extend_from_slice(bytes);
    append(
        &mut body,
        &format!(
            "\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\n{name}\r\n--{boundary}--\r\n"
        ),
    );
    body
}

fn append(body: &mut Vec<u8>, value: &str) {
    body.extend_from_slice(value.as_bytes());
}

fn header_value(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\r' | '\n' => ' ',
            '"' | '\\' => '_',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_binary_bytes_and_neutralizes_header_injection() {
        let body = file_body("boundary", "bad\r\n\"name.png", "image/png", &[0, 1, 255]);
        let text = String::from_utf8_lossy(&body);

        assert!(text.contains("filename=\"bad  _name.png\""));
        assert!(text.contains("Content-Type: image/png"));
        assert!(body.windows(3).any(|window| window == [0, 1, 255]));
        assert!(text.ends_with("--boundary--\r\n"));
    }
}
