//! Diff viewer component for terminal

use std::fmt::{Display, Formatter};

/// A single line in a diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_number_old: Option<usize>,
    pub line_number_new: Option<usize>,
    pub content: String,
    pub diff_type: DiffType,
}

/// Type of diff line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffType {
    Context,
    Addition,
    Deletion,
    Header,
    HunkHeader,
}

/// A hunk (section) in a diff
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

/// A complete diff
#[derive(Debug, Clone)]
pub struct Diff {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub hunks: Vec<DiffHunk>,
    pub is_binary: bool,
}

impl Diff {
    pub fn new() -> Self {
        Self {
            old_path: None,
            new_path: None,
            hunks: Vec::new(),
            is_binary: false,
        }
    }

    pub fn with_old_path(mut self, path: impl Into<String>) -> Self {
        self.old_path = Some(path.into());
        self
    }

    pub fn with_new_path(mut self, path: impl Into<String>) -> Self {
        self.new_path = Some(path.into());
        self
    }

    pub fn add_hunk(&mut self, hunk: DiffHunk) {
        self.hunks.push(hunk);
    }

    /// Get the total number of additions
    pub fn additions(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.diff_type == DiffType::Addition)
            .count()
    }

    /// Get the total number of deletions
    pub fn deletions(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.diff_type == DiffType::Deletion)
            .count()
    }
}

impl Default for Diff {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Diff {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Header
        if let Some(ref old) = self.old_path {
            writeln!(f, "--- a/{}", old)?;
        }
        if let Some(ref new) = self.new_path {
            writeln!(f, "+++ b/{}", new)?;
        }

        for hunk in &self.hunks {
            // Hunk header
            writeln!(
                f,
                "@@ -{},{} +{},{} @@",
                hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
            )?;

            for line in &hunk.lines {
                match line.diff_type {
                    DiffType::Addition => {
                        writeln!(f, "+{}", line.content)?;
                    }
                    DiffType::Deletion => {
                        writeln!(f, "-{}", line.content)?;
                    }
                    _ => {
                        writeln!(f, " {}", line.content)?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// ANSI color codes for diff
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const GREEN: &str = "\x1b[32m";
    pub const RED: &str = "\x1b[31m";
    pub const CYAN: &str = "\x1b[36m";
    pub const DIM: &str = "\x1b[2m";
}

/// Render a diff line with ANSI colors
pub fn render_diff_line(line: &DiffLine) -> String {
    let prefix = match line.diff_type {
        DiffType::Addition => "+",
        DiffType::Deletion => "-",
        DiffType::HunkHeader => "@",
        _ => " ",
    };

    let color = match line.diff_type {
        DiffType::Addition => colors::GREEN,
        DiffType::Deletion => colors::RED,
        DiffType::HunkHeader => colors::CYAN,
        DiffType::Header => colors::DIM,
        _ => colors::RESET,
    };

    format!("{}{}{}", color, prefix, colors::RESET)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff() {
        let mut diff = Diff::new();
        diff.add_hunk(DiffHunk {
            old_start: 1,
            old_count: 3,
            new_start: 1,
            new_count: 4,
            lines: vec![
                DiffLine {
                    line_number_old: Some(1),
                    line_number_new: Some(1),
                    content: "line 1".to_string(),
                    diff_type: DiffType::Context,
                },
                DiffLine {
                    line_number_old: Some(2),
                    line_number_new: Some(2),
                    content: "deleted".to_string(),
                    diff_type: DiffType::Deletion,
                },
                DiffLine {
                    line_number_old: None,
                    line_number_new: Some(3),
                    content: "added".to_string(),
                    diff_type: DiffType::Addition,
                },
            ],
        });

        assert_eq!(diff.additions(), 1);
        assert_eq!(diff.deletions(), 1);
    }
}
