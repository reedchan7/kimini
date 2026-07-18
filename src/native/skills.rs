use crate::protocol::SkillDescriptor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SkillActivation {
    pub name: String,
    pub args: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SkillSuggestion {
    pub command: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Default)]
pub(super) struct SkillCatalogState {
    pub session_id: Option<String>,
    pub items: Vec<SkillDescriptor>,
    pub loading: bool,
    pub error: Option<String>,
    pub activating: Option<String>,
    pub activated: Option<String>,
}

impl SkillCatalogState {
    pub fn suggestions(&self, input: &str, limit: usize) -> Vec<SkillSuggestion> {
        if !input.starts_with('/') || input.chars().any(char::is_whitespace) {
            return Vec::new();
        }
        let query = input.trim_start_matches('/').to_ascii_lowercase();
        let mut matches = self
            .items
            .iter()
            .filter_map(|skill| {
                let command = if skill.source == "builtin" {
                    format!("/{}", skill.name)
                } else {
                    format!("/skill:{}", skill.name)
                };
                let candidate = command.trim_start_matches('/').to_ascii_lowercase();
                let score = if candidate == query {
                    0
                } else if candidate.starts_with(&query) {
                    1
                } else if candidate.contains(&query)
                    || skill.name.to_ascii_lowercase().contains(&query)
                {
                    2
                } else {
                    return None;
                };
                Some((
                    score,
                    SkillSuggestion {
                        command,
                        name: skill.name.clone(),
                        description: skill.description.clone(),
                    },
                ))
            })
            .collect::<Vec<_>>();
        matches.sort_by(|(left_score, left), (right_score, right)| {
            left_score
                .cmp(right_score)
                .then_with(|| left.name.cmp(&right.name))
        });
        matches
            .into_iter()
            .take(limit)
            .map(|(_, item)| item)
            .collect()
    }

    pub fn parse_activation(&self, input: &str) -> Option<SkillActivation> {
        let command = input.strip_prefix('/')?;
        let (name, args) = command
            .split_once(char::is_whitespace)
            .unwrap_or((command, ""));
        let name = name.strip_prefix("skill:").unwrap_or(name);
        self.items
            .iter()
            .any(|skill| skill.name == name)
            .then(|| SkillActivation {
                name: name.to_owned(),
                args: (!args.trim().is_empty()).then(|| args.trim().to_owned()),
            })
    }

    pub fn begin_load(&mut self, session_id: String) {
        if self.session_id.as_deref() != Some(&session_id) {
            self.items.clear();
            self.activated = None;
        }
        self.session_id = Some(session_id);
        self.loading = true;
        self.error = None;
    }

    pub fn install(&mut self, session_id: &str, mut items: Vec<SkillDescriptor>) -> bool {
        if self.session_id.as_deref() != Some(session_id) {
            return false;
        }
        items.sort_by(|left, right| left.name.cmp(&right.name));
        self.items = items;
        self.loading = false;
        true
    }

    pub fn fail(&mut self, session_id: &str, error: String) -> bool {
        if self.session_id.as_deref() != Some(session_id) {
            return false;
        }
        self.loading = false;
        self.activating = None;
        self.error = Some(error);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(name: &str) -> SkillDescriptor {
        serde_json::from_value(serde_json::json!({"name": name})).unwrap()
    }

    #[test]
    fn catalogs_are_sorted_and_stale_sessions_cannot_replace_the_active_one() {
        let mut state = SkillCatalogState::default();
        state.begin_load("new".into());
        assert!(!state.install("old", vec![skill("old")]));
        assert!(state.install("new", vec![skill("z"), skill("a")]));
        assert_eq!(
            state
                .items
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "z"]
        );
    }

    #[test]
    fn slash_parser_accepts_prefixed_and_bare_known_skills_only() {
        let mut state = SkillCatalogState::default();
        state.begin_load("session".into());
        state.install("session", vec![skill("review")]);
        assert_eq!(
            state.parse_activation("/skill:review --fix"),
            Some(SkillActivation {
                name: "review".into(),
                args: Some("--fix".into())
            })
        );
        assert_eq!(
            state.parse_activation("/review"),
            Some(SkillActivation {
                name: "review".into(),
                args: None
            })
        );
        assert_eq!(state.parse_activation(" /review"), None);
        assert_eq!(state.parse_activation("/unknown"), None);
    }

    #[test]
    fn suggestions_use_source_aware_commands_and_stop_when_args_begin() {
        let mut state = SkillCatalogState::default();
        state.begin_load("session".into());
        let mut builtin = skill("review");
        builtin.source = "builtin".into();
        state.install("session", vec![skill("release"), builtin]);
        assert_eq!(
            state
                .suggestions("/re", 5)
                .iter()
                .map(|item| item.command.as_str())
                .collect::<Vec<_>>(),
            vec!["/review", "/skill:release"]
        );
        assert!(state.suggestions("/review ", 5).is_empty());
    }
}
