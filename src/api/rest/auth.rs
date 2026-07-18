use crate::protocol::{
    AuthSummary, OAuthCancelResult, OAuthFlowSnapshot, OAuthFlowStart, OAuthLogoutResult,
};

use super::{ApiError, KimiClient};

impl KimiClient {
    pub fn auth_summary(&self) -> Result<AuthSummary, ApiError> {
        self.get("/auth")
    }

    pub fn start_oauth_login(&self) -> Result<OAuthFlowStart, ApiError> {
        self.post("/oauth/login", &serde_json::json!({}))
    }

    pub fn oauth_login_status(&self) -> Result<Option<OAuthFlowSnapshot>, ApiError> {
        self.get_optional("/oauth/login")
    }

    pub fn cancel_oauth_login(&self) -> Result<OAuthCancelResult, ApiError> {
        self.delete("/oauth/login")
    }

    pub fn logout_oauth(&self) -> Result<OAuthLogoutResult, ApiError> {
        self.post("/oauth/logout", &serde_json::json!({}))
    }
}
