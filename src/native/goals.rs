use std::collections::HashSet;

#[derive(Debug, Default)]
pub(super) struct GoalUiState {
    armed_sessions: HashSet<String>,
    expanded_goals: HashSet<String>,
}

impl GoalUiState {
    pub fn is_armed(&self, session_id: &str) -> bool {
        self.armed_sessions.contains(session_id)
    }

    pub fn toggle_armed(&mut self, session_id: &str) -> bool {
        if self.armed_sessions.remove(session_id) {
            false
        } else {
            self.armed_sessions.insert(session_id.into());
            true
        }
    }

    pub fn disarm(&mut self, session_id: &str) {
        self.armed_sessions.remove(session_id);
    }

    pub fn is_expanded(&self, goal_id: &str) -> bool {
        self.expanded_goals.contains(goal_id)
    }

    pub fn toggle_expanded(&mut self, goal_id: &str) {
        if !self.expanded_goals.remove(goal_id) {
            self.expanded_goals.insert(goal_id.into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armed_and_expanded_state_are_independent() {
        let mut state = GoalUiState::default();
        assert!(state.toggle_armed("session"));
        state.toggle_expanded("goal");
        assert!(state.is_armed("session"));
        assert!(state.is_expanded("goal"));
        assert!(!state.toggle_armed("session"));
        assert!(state.is_expanded("goal"));
    }
}
