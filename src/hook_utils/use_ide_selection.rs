use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Selection {
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
}

impl Selection {
    pub fn new(start_line: usize, start_column: usize, end_line: usize, end_column: usize) -> Self {
        Self {
            start_line,
            end_line,
            start_column,
            end_column,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start_line == self.end_line && self.start_column == self.end_column
    }

    pub fn contains_position(&self, line: usize, column: usize) -> bool {
        if line < self.start_line || line > self.end_line {
            return false;
        }

        if line == self.start_line && column < self.start_column {
            return false;
        }

        if line == self.end_line && column > self.end_column {
            return false;
        }

        true
    }

    pub fn text(&self, source: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();
        if self.start_line >= lines.len() {
            return String::new();
        }

        if self.start_line == self.end_line {
            let line = lines[self.start_line];
            let start = self.start_column.min(line.len());
            let end = self.end_column.min(line.len());
            return line[start..end].to_string();
        }

        let mut result = String::new();

        let first_line = lines[self.start_line];
        result.push_str(&first_line[self.start_column.min(first_line.len())..]);
        result.push('\n');

        for line_num in (self.start_line + 1)..self.end_line {
            if line_num < lines.len() {
                result.push_str(lines[line_num]);
                result.push('\n');
            }
        }

        if self.end_line < lines.len() {
            let last_line = lines[self.end_line];
            result.push_str(&last_line[..self.end_column.min(last_line.len())]);
        }

        result
    }
}

pub struct SelectionManager {
    selections: HashSet<Selection>,
    current: Option<Selection>,
}

impl SelectionManager {
    pub fn new() -> Self {
        Self {
            selections: HashSet::new(),
            current: None,
        }
    }

    pub fn set_current(&mut self, selection: Option<Selection>) {
        self.current = selection;
    }

    pub fn get_current(&self) -> Option<&Selection> {
        self.current.as_ref()
    }

    pub fn add_selection(&mut self, selection: Selection) {
        self.selections.insert(selection);
    }

    pub fn clear_selections(&mut self) {
        self.selections.clear();
        self.current = None;
    }

    pub fn has_selection(&self) -> bool {
        self.current
            .as_ref()
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }
}

impl Default for SelectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_empty() {
        let sel = Selection::new(0, 0, 0, 0);
        assert!(sel.is_empty());
    }

    #[test]
    fn test_selection_contains() {
        let sel = Selection::new(1, 5, 3, 10);
        assert!(sel.contains_position(2, 6));
        assert!(!sel.contains_position(0, 0));
    }

    #[test]
    fn test_selection_manager() {
        let mut manager = SelectionManager::new();
        assert!(!manager.has_selection());

        manager.set_current(Some(Selection::new(0, 0, 1, 5)));
        assert!(manager.has_selection());
    }
}
