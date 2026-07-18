use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStatus {
    pub busy: bool,
    #[serde(default)]
    pub model: Option<String>,
    pub thinking_level: String,
    pub permission: String,
    pub plan_mode: bool,
    pub swarm_mode: bool,
    pub context_tokens: u64,
    pub max_context_tokens: u64,
    pub context_usage: f64,
}

impl SessionStatus {
    pub fn context_percent(&self) -> u8 {
        (self.context_usage.clamp(0.0, 1.0) * 100.0).round() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_percentage_is_bounded_and_rounded() {
        let mut status = SessionStatus {
            busy: false,
            model: None,
            thinking_level: "off".into(),
            permission: "manual".into(),
            plan_mode: false,
            swarm_mode: false,
            context_tokens: 0,
            max_context_tokens: 0,
            context_usage: 0.426,
        };
        assert_eq!(status.context_percent(), 43);
        status.context_usage = 2.0;
        assert_eq!(status.context_percent(), 100);
    }
}
