use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalSnapshot {
    pub goal_id: String,
    pub objective: String,
    #[serde(default)]
    pub completion_criterion: Option<String>,
    pub status: String,
    #[serde(default)]
    pub turns_used: u64,
    #[serde(default)]
    pub tokens_used: u64,
    #[serde(default)]
    pub wall_clock_ms: u64,
    #[serde(default)]
    pub terminal_reason: Option<String>,
    #[serde(default)]
    pub budget: GoalBudget,
}

impl GoalSnapshot {
    pub fn is_complete(&self) -> bool {
        self.status == "complete"
    }

    pub fn can_pause(&self) -> bool {
        self.status == "active"
    }

    pub fn can_resume(&self) -> bool {
        matches!(self.status.as_str(), "paused" | "blocked")
    }

    pub fn token_percent(&self) -> Option<u8> {
        let budget = self.budget.token_budget.filter(|budget| *budget > 0)?;
        let percent = self.tokens_used.saturating_mul(100) / budget;
        Some(percent.min(100) as u8)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalBudget {
    pub token_budget: Option<u64>,
    pub turn_budget: Option<u64>,
    pub wall_clock_budget_ms: Option<u64>,
    pub remaining_tokens: Option<u64>,
    pub remaining_turns: Option<u64>,
    pub remaining_wall_clock_ms: Option<u64>,
    #[serde(default)]
    pub token_budget_reached: bool,
    #[serde(default)]
    pub turn_budget_reached: bool,
    #[serde(default)]
    pub wall_clock_budget_reached: bool,
    #[serde(default)]
    pub over_budget: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GoalControl {
    Pause,
    Resume,
    Cancel,
}

impl GoalControl {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Cancel => "cancel",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_helpers_preserve_forward_compatible_statuses_and_bound_progress() {
        let mut goal: GoalSnapshot = serde_json::from_value(serde_json::json!({
            "goalId": "goal_01",
            "objective": "Ship the native GUI",
            "status": "active",
            "tokensUsed": 150,
            "budget": {
                "tokenBudget": 100,
                "turnBudget": null,
                "wallClockBudgetMs": null,
                "remainingTokens": 0,
                "remainingTurns": null,
                "remainingWallClockMs": null
            }
        }))
        .unwrap();

        assert_eq!(goal.token_percent(), Some(100));
        assert!(goal.can_pause());
        goal.status = "future-state".into();
        assert!(!goal.is_complete());
        assert!(!goal.can_resume());
    }
}
