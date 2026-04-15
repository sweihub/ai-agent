// Source: /data/home/swei/claudecode/openclaudecode/src/utils/undercover.ts
use std::collections::HashSet;

pub struct Undercover {
    keywords: HashSet<String>,
    enabled: bool,
}

impl Undercover {
    pub fn new() -> Self {
        let mut keywords = HashSet::new();
        keywords.insert("password".to_string());
        keywords.insert("secret".to_string());
        keywords.insert("api_key".to_string());
        keywords.insert("token".to_string());
        keywords.insert("private_key".to_string());

        Self {
            keywords,
            enabled: true,
        }
    }

    pub fn with_custom_keywords(keywords: Vec<String>) -> Self {
        Self {
            keywords: keywords.into_iter().collect(),
            enabled: true,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn scan(&self, content: &str) -> Vec<String> {
        if !self.enabled {
            return vec![];
        }

        let content_lower = content.to_lowercase();
        self.keywords
            .iter()
            .filter(|kw| content_lower.contains(&kw.to_lowercase()))
            .cloned()
            .collect()
    }

    pub fn contains_secret(&self, content: &str) -> bool {
        !self.scan(content).is_empty()
    }

    pub fn add_keyword(&mut self, keyword: String) {
        self.keywords.insert(keyword);
    }

    pub fn remove_keyword(&mut self, keyword: &str) {
        self.keywords.remove(keyword);
    }
}

impl Default for Undercover {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan() {
        let undercover = Undercover::new();
        let found = undercover.scan("My password is secret123");
        assert!(!found.is_empty());
    }

    #[test]
    fn test_no_match() {
        let undercover = Undercover::new();
        let found = undercover.scan("Hello world");
        assert!(found.is_empty());
    }
}
