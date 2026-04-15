use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DiffResult {
    pub lines_added: usize,
    pub lines_removed: usize,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub content: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: char,
    pub content: String,
}

pub fn compute_diff(old_text: &str, new_text: &str) -> DiffResult {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let lcs = longest_common_subsequence(&old_lines, &new_lines);

    let mut lines_added = 0;
    let mut lines_removed = 0;
    let mut hunks = Vec::new();
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_idx = 0;
    let mut new_idx = 0;

    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        if old_idx < lcs.len()
            && new_idx < lcs.len()
            && old_lines[old_idx] == new_lines[new_idx]
            && old_lines[old_idx] == lcs[old_idx]
        {
            if let Some(hunk) = current_hunk.take() {
                hunks.push(hunk);
            }
            old_idx += 1;
            new_idx += 1;
        } else if old_idx < old_lines.len()
            && (new_idx >= lcs.len() || old_lines[old_idx] != lcs[new_idx])
        {
            lines_removed += 1;
            if current_hunk.is_none() {
                current_hunk = Some(DiffHunk {
                    old_start: old_idx + 1,
                    old_lines: 0,
                    new_start: new_idx + 1,
                    new_lines: 0,
                    content: Vec::new(),
                });
            }
            if let Some(ref mut hunk) = current_hunk {
                hunk.old_lines += 1;
                hunk.content.push(DiffLine {
                    line_type: '-',
                    content: old_lines[old_idx].to_string(),
                });
            }
            old_idx += 1;
        } else if new_idx < new_lines.len() {
            lines_added += 1;
            if current_hunk.is_none() {
                current_hunk = Some(DiffHunk {
                    old_start: old_idx + 1,
                    old_lines: 0,
                    new_start: new_idx + 1,
                    new_lines: 0,
                    content: Vec::new(),
                });
            }
            if let Some(ref mut hunk) = current_hunk {
                hunk.new_lines += 1;
                hunk.content.push(DiffLine {
                    line_type: '+',
                    content: new_lines[new_idx].to_string(),
                });
            }
            new_idx += 1;
        } else {
            break;
        }
    }

    if let Some(hunk) = current_hunk {
        hunks.push(hunk);
    }

    DiffResult {
        lines_added,
        lines_removed,
        hunks,
    }
}

fn longest_common_subsequence(a: &[&str], b: &[&str]) -> Vec<&str> {
    let mut dp = vec![vec![0; b.len() + 1]; a.len() + 1];

    for i in 1..=a.len() {
        for j in 1..=b.len() {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut result = Vec::new();
    let mut i = a.len();
    let mut j = b.len();

    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push(a[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff() {
        let old = "line1\nline2\nline3";
        let new = "line1\nline2 modified\nline3\nline4";
        let result = compute_diff(old, new);
        assert!(result.lines_added > 0 || result.lines_removed > 0);
    }
}
