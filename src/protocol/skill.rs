use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SkillDescriptor {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub source: String,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    #[serde(default)]
    pub disable_model_invocation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SkillList {
    #[serde(default)]
    pub skills: Vec<SkillDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActivateSkillRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ActivateSkillResult {
    pub activated: bool,
    pub skill_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_descriptor_accepts_optional_and_future_source_fields() {
        let skill: SkillDescriptor = serde_json::from_value(serde_json::json!({
            "name": "review", "description": "Review code", "path": "/skills/review",
            "source": "plugin", "future": true
        }))
        .unwrap();
        assert_eq!(skill.name, "review");
        assert_eq!(skill.source, "plugin");
        assert!(!skill.disable_model_invocation);
    }
}
