use crate::protocol::FsPreview;

pub(super) fn source_markdown(file: &FsPreview) -> String {
    let language = file
        .language_id
        .as_deref()
        .filter(|value| {
            value.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            })
        })
        .unwrap_or_default();
    fenced_markdown(language, &file.content, "~~~~")
}

pub(super) fn diff_markdown(diff: &str) -> String {
    fenced_markdown("diff", diff, "~~~~")
}

fn fenced_markdown(language: &str, content: &str, base: &str) -> String {
    let mut fence = base.to_owned();
    while content.contains(&fence) {
        fence.push('~');
    }
    format!("{fence}{language}\n{content}\n{fence}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_fences_grow_past_fences_in_source() {
        let rendered = fenced_markdown("rust", "~~~~\ncode", "~~~~");
        assert!(rendered.starts_with("~~~~~rust\n"));
        assert!(rendered.ends_with("\n~~~~~"));
    }

    #[test]
    fn dangerous_language_ids_are_not_injected_into_markdown() {
        let file: FsPreview = serde_json::from_value(serde_json::json!({
            "path": "a", "content": "body", "encoding": "utf-8", "size": 4,
            "truncated": false, "mime": "text/plain", "language_id": "rust\n# title",
            "is_binary": false
        }))
        .unwrap();
        assert!(source_markdown(&file).starts_with("~~~~\n"));
    }
}
