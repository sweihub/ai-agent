// Source: /data/home/swei/claudecode/openclaudecode/src/commands/diff/diff.tsx
#![allow(dead_code)]

use similar::{ChangeTag, TextDiff};

pub struct Diff<'a> {
    diff: TextDiff<'a, 'a, 'a, str>,
}

impl<'a> Diff<'a> {
    pub fn from_lines(a: &'a str, b: &'a str) -> Self {
        Self {
            diff: TextDiff::from_lines(a, b),
        }
    }

    pub fn iter_changes(&self) -> impl Iterator<Item = (&str, &str)> {
        self.diff.iter_all_changes().map(|c| {
            let tag = match c.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            (tag, c.value())
        })
    }
}

pub fn diff_lines<'a>(a: &'a str, b: &'a str) -> Diff<'a> {
    Diff::from_lines(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_lines() {
        let diff = diff_lines("hello\nworld", "hello\nrust");
        let changes: Vec<_> = diff.iter_changes().collect();
        assert!(!changes.is_empty());
    }
}
