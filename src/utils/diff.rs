// Source: /data/home/swei/claudecode/openclaudecode/src/utils/diff.ts
//! Structured patch generation for file edits

use similar::{ChangeTag, TextDiff};

/// A single hunk from a structured diff
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructuredPatchHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    /// Lines prefixed with ' ' (context), '+' (added), or '-' (removed)
    pub lines: Vec<String>,
}

/// Context lines around each hunk
pub const CONTEXT_LINES: usize = 3;

/// Generate a structured patch from old and new file contents.
/// Uses the `similar` crate for line-level diffing.
pub fn generate_patch(old_content: &str, new_content: &str) -> Vec<StructuredPatchHunk> {
    let old_escaped = escape_for_diff(old_content);
    let new_escaped = escape_for_diff(new_content);

    if old_escaped.lines().count() == 0 && new_escaped.lines().count() == 0 {
        return Vec::new();
    }

    let mut changes: Vec<(ChangeTag, &str)> = Vec::new();
    for change in TextDiff::from_lines(&old_escaped, &new_escaped).iter_all_changes() {
        changes.push((change.tag(), change.value()));
    }

    if changes.is_empty() {
        return Vec::new();
    }

    // Group consecutive changes into hunks with context
    let mut hunks: Vec<StructuredPatchHunk> = Vec::new();
    let mut current: Vec<(ChangeTag, Vec<String>)> = Vec::new();
    let mut trailing_context: Vec<(ChangeTag, Vec<String>)> = Vec::new();

    for (i, (tag, value)) in changes.iter().enumerate() {
        let lines: Vec<String> = value.lines().map(|l| unescape_from_diff(l)).collect();

        match tag {
            ChangeTag::Delete | ChangeTag::Insert => {
                // Flush trailing context as new group
                if !trailing_context.is_empty() {
                    current = trailing_context.clone();
                    trailing_context.clear();
                }
                let prefix = match tag {
                    ChangeTag::Delete => '-',
                    ChangeTag::Insert => '+',
                    _ => ' ',
                };
                for line in lines {
                    current.push((*tag, vec![format!("{}{}", prefix, line)]));
                }
            }
            ChangeTag::Equal => {
                if !current.is_empty() && trailing_context.len() < CONTEXT_LINES {
                    trailing_context.push((*tag, lines));
                } else if !current.is_empty() {
                    // Context is too far from changes — flush current hunk
                    hunks.push(build_hunk(&current));
                    current = trailing_context.clone();
                    trailing_context.clear();
                }
                // If current is empty, just skip standalone context
            }
        }
    }

    if !current.is_empty() {
        // Use the index tracking to get line numbers
        // Simplified: compute positions from the changes
        let mut pos_old = 0usize;
        let mut pos_new = 0usize;
        let mut first_hunk_old = None;
        let mut first_hunk_new = None;
        let mut total_old = 0usize;
        let mut total_new = 0usize;

        for (tag, prefixed_lines) in &current {
            let line_count = prefixed_lines.len();
            match tag {
                ChangeTag::Delete => {
                    if first_hunk_old.is_none() {
                        first_hunk_old = Some(pos_old);
                    }
                    total_old += line_count;
                    pos_old += line_count;
                }
                ChangeTag::Insert => {
                    if first_hunk_new.is_none() {
                        first_hunk_new = Some(pos_new);
                    }
                    total_new += line_count;
                    pos_new += line_count;
                }
                ChangeTag::Equal => {
                    if first_hunk_old.is_none() {
                        first_hunk_old = Some(pos_old);
                    }
                    if first_hunk_new.is_none() {
                        first_hunk_new = Some(pos_new);
                    }
                    total_old += line_count;
                    total_new += line_count;
                    pos_old += line_count;
                    pos_new += line_count;
                }
            }
        }

        let all_lines: Vec<String> = current
            .iter()
            .flat_map(|(_, lines)| lines.clone())
            .collect();
        hunks.push(StructuredPatchHunk {
            old_start: first_hunk_old.unwrap_or(0),
            old_lines: total_old,
            new_start: first_hunk_new.unwrap_or(0),
            new_lines: total_new,
            lines: all_lines,
        });
    }

    hunks
}

/// Count lines added and removed from a structured patch.
pub fn count_lines_changed(
    patch: &[StructuredPatchHunk],
    new_file_content: Option<&str>,
) -> (usize, usize) {
    if patch.is_empty() {
        if let Some(content) = new_file_content {
            let additions = content.lines().count();
            return (additions, 0);
        }
        return (0, 0);
    }

    let additions = patch
        .iter()
        .flat_map(|h| &h.lines)
        .filter(|l| l.starts_with('+'))
        .count();

    let removals = patch
        .iter()
        .flat_map(|h| &h.lines)
        .filter(|l| l.starts_with('-'))
        .count();

    (additions, removals)
}

// & and $ confuse the diff library, so we replace them with tokens
const AMPERSAND_TOKEN: &str = "<<:AMPERSAND_TOKEN:>>";
const DOLLAR_TOKEN: &str = "<<:DOLLAR_TOKEN:>>";

fn escape_for_diff(s: &str) -> String {
    s.replace('&', AMPERSAND_TOKEN).replace('$', DOLLAR_TOKEN)
}

fn unescape_from_diff(s: &str) -> String {
    s.replace(AMPERSAND_TOKEN, "&").replace(DOLLAR_TOKEN, "$")
}

/// Build a hunk from grouped changes, computing line positions.
fn build_hunk(current: &[(ChangeTag, Vec<String>)]) -> StructuredPatchHunk {
    let mut pos_old = 0usize;
    let mut pos_new = 0usize;
    let mut first_hunk_old = None;
    let mut first_hunk_new = None;
    let mut total_old = 0usize;
    let mut total_new = 0usize;

    for (tag, prefixed_lines) in current {
        let line_count = prefixed_lines.len();
        match tag {
            ChangeTag::Delete => {
                if first_hunk_old.is_none() {
                    first_hunk_old = Some(pos_old);
                }
                total_old += line_count;
                pos_old += line_count;
            }
            ChangeTag::Insert => {
                if first_hunk_new.is_none() {
                    first_hunk_new = Some(pos_new);
                }
                total_new += line_count;
                pos_new += line_count;
            }
            ChangeTag::Equal => {
                if first_hunk_old.is_none() {
                    first_hunk_old = Some(pos_old);
                }
                if first_hunk_new.is_none() {
                    first_hunk_new = Some(pos_new);
                }
                total_old += line_count;
                total_new += line_count;
                pos_old += line_count;
                pos_new += line_count;
            }
        }
    }

    let all_lines: Vec<String> = current
        .iter()
        .flat_map(|(_, lines)| lines.clone())
        .collect();
    StructuredPatchHunk {
        old_start: first_hunk_old.unwrap_or(0),
        old_lines: total_old,
        new_start: first_hunk_new.unwrap_or(0),
        new_lines: total_new,
        lines: all_lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_patch_simple_edit() {
        let old = "Hello\nWorld\nFoo";
        let new = "Hello\nRust\nFoo";
        let hunks = generate_patch(old, new);
        assert!(!hunks.is_empty());

        let first_hunk = &hunks[0];
        let has_addition = first_hunk.lines.iter().any(|l| l.starts_with('+'));
        let has_removal = first_hunk.lines.iter().any(|l| l.starts_with('-'));
        assert!(has_addition);
        assert!(has_removal);
    }

    #[test]
    fn test_generate_patch_no_changes() {
        let content = "Hello\nWorld";
        let hunks = generate_patch(content, content);
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_generate_patch_new_file() {
        let old = "";
        let new = "Hello\nWorld";
        let hunks = generate_patch(old, new);
        assert!(!hunks.is_empty());
    }

    #[test]
    fn test_generate_patch_deletion() {
        let old = "Hello\nWorld\nFoo";
        let new = "Hello\nFoo";
        let hunks = generate_patch(old, new);
        assert!(!hunks.is_empty());
    }

    #[test]
    fn test_count_lines_changed() {
        let old = "a\nb\nc";
        let new = "a\nX\nc";
        let hunks = generate_patch(old, new);
        let (additions, removals) = count_lines_changed(&hunks, None);
        assert_eq!(additions, 1);
        assert_eq!(removals, 1);
    }

    #[test]
    fn test_count_lines_changed_empty() {
        let (additions, removals) = count_lines_changed(&[], Some("hello\nworld"));
        assert_eq!(additions, 2);
        assert_eq!(removals, 0);
    }

    #[test]
    fn test_escape_ampsersand() {
        assert_eq!(escape_for_diff("a & b"), "a <<:AMPERSAND_TOKEN:>> b");
        assert_eq!(escape_for_diff("x$y"), "x<<:DOLLAR_TOKEN:>>y");
    }

    #[test]
    fn test_unescape() {
        assert_eq!(unescape_from_diff("a <<:AMPERSAND_TOKEN:>> b"), "a & b");
        assert_eq!(unescape_from_diff("x<<:DOLLAR_TOKEN:>>y"), "x$y");
    }
}
