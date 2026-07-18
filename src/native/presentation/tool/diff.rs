use super::summary::normalize_tool_name;

const MAX_DIFF_CELLS: usize = 1_000_000;
const MAX_DIFF_ROWS: usize = 5_000;
const MAX_RENDERED_DIFF_ROWS: usize = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::native) enum DiffKind {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::native) struct DiffLine {
    pub kind: DiffKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::native) struct ToolDiff {
    pub lines: Vec<DiffLine>,
    pub added: usize,
    pub removed: usize,
}

impl ToolDiff {
    pub fn markdown(&self) -> String {
        let mut body = String::from("```diff\n");
        for line in self.lines.iter().take(MAX_RENDERED_DIFF_ROWS) {
            body.push(match line.kind {
                DiffKind::Context => ' ',
                DiffKind::Added => '+',
                DiffKind::Removed => '-',
            });
            body.push_str(&line.text);
            body.push('\n');
        }
        if self.lines.len() > MAX_RENDERED_DIFF_ROWS {
            body.push_str(" … diff truncated\n");
        }
        body.push_str("```");
        body
    }
}

pub(super) fn edit_diff(name: &str, input: &serde_json::Value) -> Option<ToolDiff> {
    if normalize_tool_name(name) != "edit"
        || input
            .get("replace_all")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
    {
        return None;
    }
    let before = input.get("old_string")?.as_str()?;
    let after = input.get("new_string")?.as_str()?;
    build_diff(before, after)
}

fn build_diff(before: &str, after: &str) -> Option<ToolDiff> {
    let old = split_lines(before);
    let new = split_lines(after);
    let rows = old.len().checked_add(1)?.checked_mul(new.len() + 1)?;
    if old.len() > MAX_DIFF_ROWS || new.len() > MAX_DIFF_ROWS || rows > MAX_DIFF_CELLS {
        return None;
    }
    let width = new.len() + 1;
    let mut lengths = vec![0_u32; rows];
    for old_index in 1..=old.len() {
        for new_index in 1..=new.len() {
            let slot = old_index * width + new_index;
            lengths[slot] = if old[old_index - 1] == new[new_index - 1] {
                lengths[(old_index - 1) * width + new_index - 1] + 1
            } else {
                lengths[(old_index - 1) * width + new_index]
                    .max(lengths[old_index * width + new_index - 1])
            };
        }
    }

    let mut lines = Vec::with_capacity(old.len() + new.len());
    let (mut old_index, mut new_index) = (old.len(), new.len());
    while old_index > 0 || new_index > 0 {
        if old_index > 0 && new_index > 0 && old[old_index - 1] == new[new_index - 1] {
            lines.push(DiffLine {
                kind: DiffKind::Context,
                text: old[old_index - 1].to_owned(),
            });
            old_index -= 1;
            new_index -= 1;
        } else if new_index > 0
            && (old_index == 0
                || lengths[old_index * width + new_index - 1]
                    >= lengths[(old_index - 1) * width + new_index])
        {
            lines.push(DiffLine {
                kind: DiffKind::Added,
                text: new[new_index - 1].to_owned(),
            });
            new_index -= 1;
        } else {
            lines.push(DiffLine {
                kind: DiffKind::Removed,
                text: old[old_index - 1].to_owned(),
            });
            old_index -= 1;
        }
    }
    lines.reverse();
    let added = lines
        .iter()
        .filter(|line| line.kind == DiffKind::Added)
        .count();
    let removed = lines
        .iter()
        .filter(|line| line.kind == DiffKind::Removed)
        .count();
    Some(ToolDiff {
        lines,
        added,
        removed,
    })
}

fn split_lines(value: &str) -> Vec<&str> {
    let mut lines = value.split('\n').collect::<Vec<_>>();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_diff_uses_line_level_alignment_and_counts_changes() {
        let diff = build_diff("one\ntwo\nthree\n", "one\nsecond\nthree\nfour\n").unwrap();
        assert_eq!(diff.added, 2);
        assert_eq!(diff.removed, 1);
        assert!(diff.markdown().contains("-two"));
        assert!(diff.markdown().contains("+second"));
    }

    #[test]
    fn oversized_diffs_fall_back_without_allocating_the_matrix() {
        let before = "old\n".repeat(MAX_DIFF_ROWS + 1);
        assert!(build_diff(&before, "new").is_none());
    }
}
