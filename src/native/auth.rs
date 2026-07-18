use crate::protocol::{AuthSummary, OAuthFlowStart};

#[derive(Debug, Default)]
pub(super) struct AuthState {
    pub(in crate::native) summary: Option<AuthSummary>,
    pub(in crate::native) flow: Option<OAuthFlowStart>,
    pub(in crate::native) loading: bool,
    pub(in crate::native) error: Option<String>,
    pub(in crate::native) poll_generation: u64,
    pub(in crate::native) poll_scheduled: bool,
}

impl AuthState {
    pub fn ready(&self) -> bool {
        self.summary.as_ref().is_some_and(|summary| summary.ready)
    }

    pub fn pending(&self) -> Option<(&str, &str, u64)> {
        self.flow.as_ref()?.pending_details()
    }

    pub fn begin_flow(&mut self, flow: OAuthFlowStart) {
        self.poll_generation = self.poll_generation.wrapping_add(1);
        self.poll_scheduled = false;
        self.error = None;
        self.flow = Some(flow);
    }

    pub fn clear_flow(&mut self) {
        self.poll_generation = self.poll_generation.wrapping_add(1);
        self.poll_scheduled = false;
        self.flow = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacing_and_clearing_login_flows_invalidates_old_pollers() {
        let mut state = AuthState::default();
        let flow = serde_json::from_value(serde_json::json!({
            "flow_id": "f1", "provider": "kimi", "status": "authenticated"
        }))
        .unwrap();

        state.begin_flow(flow);
        let started = state.poll_generation;
        state.clear_flow();

        assert!(state.poll_generation > started);
        assert!(state.flow.is_none());
    }
}
