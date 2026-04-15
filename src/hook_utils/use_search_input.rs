use std::collections::VecDeque;

pub struct SearchState {
    query: String,
    results: VecDeque<SearchResult>,
    current_index: usize,
    history: VecDeque<String>,
    max_history: usize,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub text: String,
    pub score: f64,
}

impl SearchState {
    pub fn new(max_history: usize) -> Self {
        Self {
            query: String::new(),
            results: VecDeque::new(),
            current_index: 0,
            history: VecDeque::new(),
            max_history,
        }
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
    }

    pub fn get_query(&self) -> &str {
        &self.query
    }

    pub fn add_to_history(&mut self, query: String) {
        if !query.is_empty() {
            self.history.push_front(query.clone());
            while self.history.len() > self.max_history {
                self.history.pop_back();
            }
        }
    }

    pub fn get_history(&self) -> Vec<String> {
        self.history.iter().cloned().collect()
    }

    pub fn set_results(&mut self, results: Vec<SearchResult>) {
        self.results = results.into();
        self.current_index = 0;
    }

    pub fn get_current_result(&self) -> Option<&SearchResult> {
        self.results.get(self.current_index)
    }

    pub fn next_result(&mut self) -> Option<&SearchResult> {
        if !self.results.is_empty() {
            self.current_index = (self.current_index + 1) % self.results.len();
            self.results.get(self.current_index)
        } else {
            None
        }
    }

    pub fn prev_result(&mut self) -> Option<&SearchResult> {
        if !self.results.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.results.len() - 1;
            } else {
                self.current_index -= 1;
            }
            self.results.get(self.current_index)
        } else {
            None
        }
    }

    pub fn has_results(&self) -> bool {
        !self.results.is_empty()
    }

    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    pub fn clear_results(&mut self) {
        self.results.clear();
        self.current_index = 0;
    }

    pub fn navigate_to(&mut self, index: usize) -> Option<&SearchResult> {
        if index < self.results.len() {
            self.current_index = index;
            self.results.get(index)
        } else {
            None
        }
    }
}

pub fn highlight_match(text: &str, query: &str) -> String {
    if query.is_empty() {
        return text.to_string();
    }

    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    let mut result = String::new();
    let mut last_end = 0;

    for (start, _) in text_lower.match_indices(&query_lower) {
        result.push_str(&text[last_end..start]);
        result.push_str("\x1b[7m");
        result.push_str(&text[start..start + query.len()]);
        result.push_str("\x1b[0m");
        last_end = start + query.len();
    }

    result.push_str(&text[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_state_query() {
        let mut state = SearchState::new(10);
        state.set_query("test".to_string());
        assert_eq!(state.get_query(), "test");
    }

    #[test]
    fn test_search_state_history() {
        let mut state = SearchState::new(2);
        state.add_to_history("query1".to_string());
        state.add_to_history("query2".to_string());
        state.add_to_history("query3".to_string());

        let history = state.get_history();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_search_results_navigation() {
        let mut state = SearchState::new(10);
        state.set_results(vec![
            SearchResult {
                id: "1".to_string(),
                text: "Result 1".to_string(),
                score: 1.0,
            },
            SearchResult {
                id: "2".to_string(),
                text: "Result 2".to_string(),
                score: 0.9,
            },
        ]);

        assert!(state.has_results());
        assert_eq!(state.result_count(), 2);

        state.next_result();
        let result = state.get_current_result();
        assert!(result.is_some());
    }
}
