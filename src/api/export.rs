use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;

use super::{ApiError, KimiClient};

const MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;
static EXPORT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionExport {
    pub path: PathBuf,
    pub bytes: u64,
}

impl KimiClient {
    pub fn export_session_to(
        &self,
        session_id: &str,
        destination: &Path,
    ) -> Result<SessionExport, ApiError> {
        let response = self
            .post_request(&format!("/sessions/{}/export", encoded_segment(session_id)))
            .set("Content-Type", "application/json")
            .send_json(serde_json::json!({}))
            .map_err(|error| ApiError::Transport(error.to_string()))?;
        let content_type = response
            .header("content-type")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let mut reader = response.into_reader().take(MAX_ARCHIVE_BYTES + 1);
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .map_err(|error| ApiError::Transport(error.to_string()))?;

        if !content_type.starts_with("application/zip") {
            return Err(decode_export_error(&bytes));
        }
        if bytes.len() as u64 > MAX_ARCHIVE_BYTES {
            return Err(ApiError::InvalidResponse(
                "session archive exceeds the 64 MiB client limit".into(),
            ));
        }

        write_atomically(destination, &bytes)?;
        Ok(SessionExport {
            path: destination.to_owned(),
            bytes: bytes.len() as u64,
        })
    }
}

#[derive(Deserialize)]
struct ErrorEnvelope {
    code: i64,
    msg: String,
}

fn decode_export_error(bytes: &[u8]) -> ApiError {
    match serde_json::from_slice::<ErrorEnvelope>(bytes) {
        Ok(envelope) if envelope.code != 0 => ApiError::Daemon {
            code: envelope.code,
            message: envelope.msg,
        },
        Ok(_) => ApiError::InvalidResponse("expected a ZIP session archive".into()),
        Err(error) => ApiError::InvalidResponse(error.to_string()),
    }
}

fn write_atomically(destination: &Path, bytes: &[u8]) -> Result<(), ApiError> {
    let parent = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("session.zip");
    let temp = parent.join(format!(
        ".{name}.kimini-{}-{}.part",
        std::process::id(),
        EXPORT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&temp, bytes).map_err(|error| ApiError::LocalFile(error.to_string()))?;
    if let Err(error) = fs::rename(&temp, destination) {
        let _ = fs::remove_file(&temp);
        return Err(ApiError::LocalFile(error.to_string()));
    }
    Ok(())
}

fn encoded_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes())
        .collect::<String>()
        .replace('+', "%20")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_errors_remain_structured_for_binary_routes() {
        let error = decode_export_error(br#"{"code":40401,"msg":"missing","data":null}"#);
        assert_eq!(error.to_string(), "Kimi daemon error 40401: missing");
    }

    #[test]
    fn path_segments_match_rest_route_encoding() {
        assert_eq!(encoded_segment("session/a b"), "session%2Fa%20b");
    }
}
