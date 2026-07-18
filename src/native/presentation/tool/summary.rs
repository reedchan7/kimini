pub(super) const MAX_SUMMARY_CHARS: usize = 96;
const MAX_TOOL_CHARS: usize = 6_000;

pub(super) fn display_value(value: &serde_json::Value) -> String {
    let rendered = value.as_str().map(str::to_owned).unwrap_or_else(|| {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    });
    truncate_chars(rendered, MAX_TOOL_CHARS)
}

pub(super) fn tool_detail(name: &str, input: &serde_json::Value, has_diff: bool) -> Option<String> {
    if has_diff {
        return None;
    }
    match normalize_tool_name(name).as_str() {
        "read" | "grep" | "search" | "glob" | "ls" => None,
        "bash" => string_field(input, &["command", "cmd", "script"]).map(str::to_owned),
        _ => {
            let detail = display_value(input);
            (!matches!(detail.trim(), "" | "{}" | "[]" | "null")).then_some(detail)
        }
    }
}

pub(super) fn tool_summary(name: &str, input: &serde_json::Value) -> String {
    let fallback = || display_value(input);
    let summary = match normalize_tool_name(name).as_str() {
        "read" => file_path(input).map(|path| {
            let start = number_field(input, &["offset", "line_start", "start_line"]);
            let len = number_field(input, &["limit", "length", "n_lines"]);
            let end = number_field(input, &["line_end", "end_line"])
                .or_else(|| start.zip(len).map(|(start, len)| start + len));
            match (start, end) {
                (Some(start), Some(end)) => format!("{path}:{start}-{end}"),
                (Some(start), None) => format!("{path}:{start}"),
                _ => path.to_owned(),
            }
        }),
        "edit" | "multi_edit" | "write" => file_path(input).map(str::to_owned),
        "bash" => string_field(input, &["command", "cmd", "script"]).map(str::to_owned),
        "grep" | "search" => {
            let pattern = string_field(input, &["pattern", "query", "regex"]);
            let path = string_field(input, &["path", "glob", "include"]);
            match (pattern, path) {
                (Some(pattern), Some(path)) => Some(format!("{pattern} in {path}")),
                (Some(pattern), None) => Some(pattern.to_owned()),
                _ => None,
            }
        }
        "glob" => string_field(input, &["pattern", "glob", "query", "path"]).map(str::to_owned),
        "ls" => string_field(input, &["path", "dir", "directory", "cwd"]).map(str::to_owned),
        _ => None,
    }
    .unwrap_or_else(fallback);
    clip_chars(summary)
}

pub(super) fn clip_chars(value: String) -> String {
    let Some((index, _)) = value.char_indices().nth(MAX_SUMMARY_CHARS) else {
        return value;
    };
    format!("{}…", &value[..index])
}

pub(super) fn normalize_tool_name(name: &str) -> String {
    match name.trim().to_lowercase().replace([' ', '-'], "_").as_str() {
        "shell" | "run" | "exec" => "bash".into(),
        "ripgrep" | "rg" => "grep".into(),
        "multiedit" | "multiedits" => "multi_edit".into(),
        "find" => "glob".into(),
        "list" | "listdir" | "list_dir" => "ls".into(),
        other => other.to_owned(),
    }
}

fn file_path(input: &serde_json::Value) -> Option<&str> {
    string_field(input, &["path", "file_path", "filePath", "filename"])
}

fn string_field<'a>(input: &'a serde_json::Value, names: &[&str]) -> Option<&'a str> {
    names
        .iter()
        .find_map(|name| input.get(name).and_then(serde_json::Value::as_str))
        .filter(|value| !value.is_empty())
}

fn number_field(input: &serde_json::Value, names: &[&str]) -> Option<u64> {
    names
        .iter()
        .find_map(|name| input.get(name).and_then(serde_json::Value::as_u64))
}

fn truncate_chars(value: String, limit: usize) -> String {
    let Some((index, _)) = value.char_indices().nth(limit) else {
        return value;
    };
    format!("{}\n… output truncated", &value[..index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_summaries_surface_the_action_instead_of_raw_json() {
        assert_eq!(
            tool_summary(
                "Read",
                &serde_json::json!({ "path": "/tmp/main.rs", "line_start": 4, "n_lines": 8 })
            ),
            "/tmp/main.rs:4-12"
        );
        assert_eq!(
            tool_summary(
                "Grep",
                &serde_json::json!({ "pattern": "Session", "path": "src" })
            ),
            "Session in src"
        );
    }

    #[test]
    fn summaries_clip_unicode_on_character_boundaries() {
        let summary = tool_summary("Bash", &serde_json::json!({ "command": "你".repeat(120) }));
        assert_eq!(summary.chars().count(), MAX_SUMMARY_CHARS + 1);
        assert!(summary.ends_with('…'));
    }

    #[test]
    fn tool_output_is_bounded_without_breaking_unicode() {
        let output = display_value(&serde_json::Value::String("你".repeat(MAX_TOOL_CHARS + 1)));
        assert_eq!(
            output
                .chars()
                .filter(|character| *character == '你')
                .count(),
            MAX_TOOL_CHARS
        );
        assert!(output.ends_with("output truncated"));
    }
}
