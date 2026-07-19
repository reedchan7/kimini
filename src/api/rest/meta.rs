use serde::Deserialize;

use super::{ApiError, KimiClient};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ServerMeta {
    pub server_version: String,
    #[serde(default = "legacy_backend")]
    pub backend: String,
}

fn legacy_backend() -> String {
    "v1".into()
}

impl KimiClient {
    pub fn server_meta(&self) -> Result<ServerMeta, ApiError> {
        self.get("/meta")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_backend_is_treated_as_legacy() {
        let meta: ServerMeta = serde_json::from_str(r#"{"server_version":"0.26.0"}"#).unwrap();
        assert_eq!(meta.backend, "v1");
    }
}
