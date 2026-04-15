// Source: /data/home/swei/claudecode/openclaudecode/src/ink/screen.ts
//! Screen buffer management for terminal display
//!
//! Provides screen buffer operations similar to the original TypeScript implementation.

use std::collections::VecDeque;

/// Maximum number of lines to keep in the screen buffer
pub const MAX_SCREEN_BUFFER_LINES: usize = 10000;

/// A single line in the screen buffer
#[derive(Debug, Clone, String)]
pub struct ScreenLine(pub String);

impl ScreenLine {
    pub fn new(content: impl Into<String>) -> Self {
        Self(content.into())
    }

    pub fn content(&self) -> &str {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Screen buffer for storing terminal output
#[derive(Debug)]
pub struct ScreenBuffer {
    lines: VecDeque<ScreenLine>,
    cursor_row: usize,
    cursor_col: usize,
    scroll_top: usize,
    scroll_bottom: usize,
}

impl Default for ScreenBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenBuffer {
    pub fn new() -> Self {
        Self {
            lines: VecDeque::with_capacity(MAX_SCREEN_BUFFER_LINES),
            cursor_row: 0,
            cursor_col: 0,
            scroll_top: 0,
            scroll_bottom: MAX_SCREEN_BUFFER_LINES,
        }
    }

    /// Add a new line to the screen buffer
    pub fn add_line(&mut self, line: ScreenLine) {
        if self.lines.len() >= MAX_SCREEN_BUFFER_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    /// Add a string as a new line
    pub fn add_string_line(&mut self, s: impl Into<String>) {
        self.add_line(ScreenLine::new(s));
    }

    /// Get a line at the given index
    pub fn get_line(&self, index: usize) -> Option<&ScreenLine> {
        self.lines.get(index)
    }

    /// Get the number of lines in the buffer
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Get all lines as an iterator
    pub fn lines(&self) -> impl Iterator<Item = &ScreenLine> {
        self.lines.iter()
    }

    /// Clear all lines from the buffer
    pub fn clear(&mut self) {
        self.lines.clear();
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Get current cursor position
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    /// Set cursor position
    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.lines.saturating_len());
        self.cursor_col = col;
    }

    /// Get the visible lines based on scroll position
    pub fn visible_lines(&self, viewport_height: usize) -> impl Iterator<Item = &ScreenLine> {
        let start = self.scroll_top;
        let end = (self.scroll_top + viewport_height).min(self.lines.len());
        self.lines.iter().skip(start).take(end - start)
    }

    /// Scroll the buffer
    pub fn scroll(&mut self, lines: isize) {
        let new_top = if lines < 0 {
            self.scroll_top.saturating_sub((-lines) as usize)
        } else {
            self.scroll_top.saturating_add(lines as usize)
        };
        self.scroll_top = new_top.min(self.lines.saturating_len().saturating_sub(1));
    }

    /// Set scroll region
    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        self.scroll_top = top;
        self.scroll_bottom = bottom;
    }

    /// Get scroll region
    pub fn scroll_region(&self) -> (usize, usize) {
        (self.scroll_top, self.scroll_bottom)
    }
}

/// ANSI escape codes for screen operations
pub mod ansi {
    /// Move cursor to specific position
    pub fn cursor_position(row: usize, col: usize) -> String {
        format!("\x1b[{};{}H", row + 1, col + 1)
    }

    /// Clear screen
    pub fn clear_screen(mode: ClearMode) -> String {
        match mode {
            ClearMode::FromCursorToEnd => "\x1b[0J".to_string(),
            ClearMode::FromStartToCursor => "\x1b[1J".to_string(),
            ClearMode::Complete => "\x1b[2J".to_string(),
            ClearMode::CompleteWithScrollback => "\x1b[3J".to_string(),
        }
    }

    /// Clear line
    pub fn clear_line(mode: ClearMode) -> String {
        match mode {
            ClearMode::FromCursorToEnd => "\x1b[0K".to_string(),
            ClearMode::FromStartToCursor => "\x1b[1K".to_string(),
            ClearMode::Complete => "\x1b[2K".to_string(),
            ClearMode::FromCursorToEnd => "\x1b[0J".to_string(),
        }
    }

    /// Scroll the screen up by n lines
    pub fn scroll_up(lines: usize) -> String {
        format!("\x1b[{}S", lines)
    }

    /// Scroll the screen down by n lines
    pub fn scroll_down(lines: usize) -> String {
        format!("\x1b[{}T", lines)
    }

    /// Save cursor position
    pub fn save_cursor() -> String {
        "\x1b[s".to_string()
    }

    /// Restore cursor position
    pub fn restore_cursor() -> String {
        "\x1b[u".to_string()
    }

    /// Hide cursor
    pub fn hide_cursor() -> String {
        "\x1b[?25l".to_string()
    }

    /// Show cursor
    pub fn show_cursor() -> String {
        "\x1b[?25h".to_string()
    }

    #[derive(Debug, Clone, Copy)]
    pub enum ClearMode {
        FromCursorToEnd,
        FromStartToCursor,
        Complete,
        CompleteWithScrollback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_buffer() {
        let mut buf = ScreenBuffer::new();
        assert!(buf.is_empty());

        buf.add_string_line("Hello");
        buf.add_string_line("World");

        assert_eq!(buf.len(), 2);
        assert_eq!(buf.get_line(0).map(|l| l.content()), Some("Hello"));
    }

    #[test]
    fn test_ansi_escape() {
        assert_eq!(ansi::cursor_position(0, 0), "\x1b[1;1H");
        assert_eq!(ansi::clear_screen(ansi::ClearMode::Complete), "\x1b[2J");
    }
}
