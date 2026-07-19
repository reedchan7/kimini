mod auth;
mod catalog;
mod config;
mod file;
mod fs;
mod goals;
mod interaction;
mod meta;
mod prompt;
mod session;
mod side_chat;
mod skills;
mod tasks;
mod terminals;
mod workspace;

use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::daemon::Connection;

use super::ApiError;

pub use config::{KimiConfig, ThinkingConfig};
pub use meta::ServerMeta;
pub use prompt::PromptResult;

const API_PREFIX: &str = "/api/v1";

#[derive(Clone)]
pub struct KimiClient {
    connection: Connection,
    agent: ureq::Agent,
}

impl KimiClient {
    pub fn new(connection: Connection) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(30))
            .build();
        Self { connection, agent }
    }

    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let request = self.authorize(self.agent.get(&self.url(path)));
        let response = request.call().map_err(transport_error)?;
        decode(response)
    }

    fn get_optional<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>, ApiError> {
        let request = self.authorize(self.agent.get(&self.url(path)));
        let response = request.call().map_err(transport_error)?;
        decode_optional(response)
    }

    fn post<T: DeserializeOwned>(&self, path: &str, body: &impl Serialize) -> Result<T, ApiError> {
        let request = self
            .post_request(path)
            .set("Content-Type", "application/json");
        let response = request.send_json(body).map_err(transport_error)?;
        decode(response)
    }

    fn patch<T: DeserializeOwned>(&self, path: &str, body: &impl Serialize) -> Result<T, ApiError> {
        let request = self
            .authorize(self.agent.request("PATCH", &self.url(path)))
            .set("Content-Type", "application/json");
        let response = request.send_json(body).map_err(transport_error)?;
        decode(response)
    }

    fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let request = self.authorize(self.agent.delete(&self.url(path)));
        let response = request.call().map_err(transport_error)?;
        decode(response)
    }

    pub(super) fn post_request(&self, path: &str) -> ureq::Request {
        self.authorize(self.agent.post(&self.url(path)))
    }

    fn authorize(&self, request: ureq::Request) -> ureq::Request {
        let request = request
            .set("X-Kimi-Client-Id", "kimini-native")
            .set("X-Kimi-Client-Name", "Kimini")
            .set("X-Kimi-Client-Version", env!("CARGO_PKG_VERSION"))
            .set("X-Kimi-Client-Ui-Mode", "native");
        match self.connection.token() {
            Some(token) => request.set("Authorization", &format!("Bearer {token}")),
            None => request,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}{}", self.connection.origin(), API_PREFIX, path)
    }
}

#[derive(Debug, serde::Deserialize)]
struct Envelope<T> {
    code: i64,
    msg: String,
    data: Option<T>,
}

fn decode<T: DeserializeOwned>(response: ureq::Response) -> Result<T, ApiError> {
    decode_with_allowed_codes(response, &[])
}

fn decode_optional<T: DeserializeOwned>(response: ureq::Response) -> Result<Option<T>, ApiError> {
    let envelope: Envelope<T> = response
        .into_json()
        .map_err(|error| ApiError::InvalidResponse(error.to_string()))?;
    if envelope.code != 0 {
        return Err(ApiError::Daemon {
            code: envelope.code,
            message: envelope.msg,
        });
    }
    Ok(envelope.data)
}

fn decode_with_allowed_codes<T: DeserializeOwned>(
    response: ureq::Response,
    allowed_codes: &[i64],
) -> Result<T, ApiError> {
    let envelope: Envelope<T> = response
        .into_json()
        .map_err(|error| ApiError::InvalidResponse(error.to_string()))?;
    if envelope.code != 0 && !allowed_codes.contains(&envelope.code) {
        return Err(ApiError::Daemon {
            code: envelope.code,
            message: envelope.msg,
        });
    }
    envelope.data.ok_or(ApiError::MissingData)
}

fn transport_error(error: ureq::Error) -> ApiError {
    ApiError::Transport(error.to_string())
}

fn segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes())
        .collect::<String>()
        .replace('+', "%20")
}

#[cfg(test)]
mod tests {
    use super::segment;

    #[test]
    fn path_segments_are_encoded_without_touching_safe_ids() {
        assert_eq!(segment("sess_01"), "sess_01");
        assert_eq!(segment("a/b c"), "a%2Fb%20c");
    }
}
