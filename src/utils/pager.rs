//! Terminal pager functionality
//!
//! Provides a pager for navigating through large amounts of text.

use std::collections::VecDeque;

/// Maximum lines to keep in pager history
pub const PAGER_MAX_LINES: usize = 10000;

/// A pager for navigating through large text output
#[derive(Debug)]
pub struct Pager {
    lines: VecDeque<String>,
    current_line: usize,
    lines_per_page: usize,
}

impl Default for Pager {
    fn default() -> Self {
        Self::new()
    }
}

impl Pager {
    pub fn new() -> Self {
        Self {
            lines: VecDeque::new(),
            current_line: 0,
            lines_per_page: 24,
        }
    }

    /// Set the number of lines per page
    pub fn with_lines_per_page(mut self, lines: usize) -> Self {
        self.lines_per_page = lines;
        self
    }

    /// Add lines to the pager
    pub fn add_lines(&mut self, text: &str) {
        for line in text.lines() {
            if self.lines.len() >= PAGER_MAX_LINES {
                self.lines.pop_front();
            }
            self.lines.push_back(line.to_string());
        }
    }

    /// Add a single line
    pub fn add_line(&mut self, line: impl Into<String>) {
        let line = line.into();
        if self.lines.len() >= PAGER_MAX_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    /// Get the total number of lines
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// Get the current line number (0-indexed)
    pub fn current_line(&self) -> usize {
        self.current_line
    }

    /// Get the number of pages
    pub fn total_pages(&self) -> usize {
        (self.lines.len() + self.lines_per_page - 1) / self.lines_per_page
    }

    /// Get the current page number (0-indexed)
    pub fn current_page(&self) -> usize {
        self.current_line / self.lines_per_page
    }

    /// Get lines for the current page
    pub fn current_page_lines(&self) -> Vec<&str> {
        let start = self.current_line;
        let end = (start + self.lines_per_page).min(self.lines.len());
        self.lines
            .iter()
            .skip(start)
            .take(end - start)
            .map(|s| s.as_str())
            .collect()
    }

    /// Move to the next page
    pub fn next_page(&mut self) -> bool {
        let next_line = (self.current_line + self.lines_per_page).min(self.lines.len() - 1);
        if next_line != self.current_line {
            self.current_line = next_line;
            true
        } else {
            false
        }
    }

    /// Move to the previous page
    pub fn prev_page(&mut self) -> bool {
        if self.current_line >= self.lines_per_page {
            self.current_line -= self.lines_per_page;
            true
        } else {
            false
        }
    }

    /// Move to the next line
    pub fn next_line(&mut self) -> bool {
        if self.current_line < self.lines.len() - 1 {
            self.current_line += 1;
            true
        } else {
            false
        }
    }

    /// Move to the previous line
    pub fn prev_line(&mut self) -> bool {
        if self.current_line > 0 {
            self.current_line -= 1;
            true
        } else {
            false
        }
    }

    /// Move to the start
    pub fn go_to_start(&mut self) {
        self.current_line = 0;
    }

    /// Move to the end
    pub fn go_to_end(&mut self) {
        self.current_line = self.lines.saturating_len().saturating_sub(1);
    }

    /// Go to a specific line
    pub fn go_to_line(&mut self, line: usize) {
        self.current_line = line.min(self.lines.saturating_len().saturating_sub(1));
    }

    /// Check if at the start
    pub fn at_start(&self) -> bool {
        self.current_line == 0
    }

    /// Check if at the end
    pub fn at_end(&self) -> bool {
        self.current_line >= self.lines.len().saturating_sub(1)
    }

    /// Check if there's a next page
    pub fn has_next_page(&self) -> bool {
        self.current_line + self.lines_per_page < self.lines.len()
    }

    /// Check if there's a previous page
    pub fn has_prev_page(&self) -> bool {
        self.current_line >= self.lines_per_page
    }

    /// Clear the pager
    pub fn clear(&mut self) {
        self.lines.clear();
        self.current_line = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pager_basic() {
        let mut pager = Pager::new().with_lines_per_page(5);

        for i in 0..20 {
            pager.add_line(format!("Line {}", i));
        }

        assert_eq!(pager.total_lines(), 20);
        assert_eq!(pager.total_pages(), 4);

        pager.go_to_end();
        assert!(pager.at_end());

        pager.go_to_start();
        assert!(pager.at_start());
    }
}
