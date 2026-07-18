use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedProviderSummary {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSummary {
    pub ready: bool,
    pub providers_count: u64,
    pub default_model: Option<String>,
    pub managed_provider: Option<ManagedProviderSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthFlowStatus {
    Pending,
    Authenticated,
    Denied,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum OAuthFlowStart {
    Pending {
        flow_id: String,
        provider: String,
        verification_uri: String,
        verification_uri_complete: String,
        user_code: String,
        expires_in: u64,
        interval: u64,
        expires_at: String,
    },
    Authenticated {
        flow_id: String,
        provider: String,
    },
}

impl OAuthFlowStart {
    pub fn pending_details(&self) -> Option<(&str, &str, u64)> {
        match self {
            Self::Pending {
                verification_uri_complete,
                user_code,
                interval,
                ..
            } => Some((verification_uri_complete, user_code, *interval)),
            Self::Authenticated { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuthFlowSnapshot {
    pub flow_id: String,
    pub provider: String,
    pub status: OAuthFlowStatus,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub user_code: String,
    pub expires_in: u64,
    pub expires_at: String,
    pub interval: u64,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuthCancelResult {
    pub cancelled: bool,
    pub status: OAuthFlowStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuthLogoutResult {
    pub logged_out: bool,
    pub provider: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_login_exposes_only_the_user_facing_handoff() {
        let flow: OAuthFlowStart = serde_json::from_value(serde_json::json!({
            "flow_id": "f1", "provider": "kimi", "status": "pending",
            "verification_uri": "https://example.test/device",
            "verification_uri_complete": "https://example.test/device?code=ABCD",
            "user_code": "ABCD", "expires_in": 600, "interval": 5,
            "expires_at": "2026-07-18T09:00:00.000Z"
        }))
        .unwrap();

        assert_eq!(
            flow.pending_details(),
            Some(("https://example.test/device?code=ABCD", "ABCD", 5))
        );
    }
}
