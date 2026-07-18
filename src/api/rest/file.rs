use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::api::multipart;
use crate::protocol::FileMeta;

use super::{ApiError, KimiClient, decode, transport_error};

const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;
static MULTIPART_ID: AtomicU64 = AtomicU64::new(1);

impl KimiClient {
    pub fn upload_file(&self, path: &Path) -> Result<FileMeta, ApiError> {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .ok_or_else(|| ApiError::LocalFile("file name is missing or invalid".into()))?;
        let bytes = fs::read(path).map_err(|error| ApiError::LocalFile(error.to_string()))?;
        if bytes.len() > MAX_UPLOAD_BYTES {
            return Err(ApiError::LocalFile(
                "file exceeds the 50 MB upload limit".into(),
            ));
        }
        let media_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .essence_str()
            .to_owned();
        let boundary = format!(
            "kimini-{}-{}",
            std::process::id(),
            MULTIPART_ID.fetch_add(1, Ordering::Relaxed)
        );
        let body = multipart::file_body(&boundary, name, &media_type, &bytes);
        let request = self.authorize(self.agent.post(&self.url("/files"))).set(
            "Content-Type",
            &format!("multipart/form-data; boundary={boundary}"),
        );
        let response = request.send_bytes(&body).map_err(transport_error)?;
        decode(response)
    }
}
