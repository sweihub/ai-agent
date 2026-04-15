use std::collections::HashMap;

pub struct Trie {
    children: HashMap<char, Trie>,
    is_end: bool,
    count: usize,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            children: HashMap::new(),
            is_end: false,
            count: 0,
        }
    }

    pub fn insert(&mut self, word: &str) {
        let mut node = self;
        for ch in word.chars() {
            node = node.children.entry(ch).or_insert_with(Trie::new);
        }
        node.is_end = true;
        node.count += 1;
    }

    pub fn search(&self, word: &str) -> bool {
        self.find_node(word).map(|n| n.is_end).unwrap_or(false)
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        self.find_node(prefix).is_some()
    }

    fn find_node(&self, prefix: &str) -> Option<&Trie> {
        let mut node = self;
        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(n) => node = n,
                None => return None,
            }
        }
        Some(node)
    }

    pub fn autocomplete(&self, prefix: &str) -> Vec<String> {
        let mut results = Vec::new();
        if let Some(node) = self.find_node(prefix) {
            self.collect_words(node, prefix, &mut results);
        }
        results
    }

    fn collect_words(&self, node: &Trie, prefix: &str, results: &mut Vec<String>) {
        if node.is_end {
            results.push(prefix.to_string());
        }
        for (ch, child) in &node.children {
            let mut new_prefix = prefix.to_string();
            new_prefix.push(*ch);
            self.collect_words(child, &new_prefix, results);
        }
    }

    pub fn remove(&mut self, word: &str) -> bool {
        self.remove_recursive(word, 0)
    }

    fn remove_recursive(&mut self, word: &str, depth: usize) -> bool {
        if depth == word.len() {
            if !self.is_end {
                return false;
            }
            self.is_end = false;
            return self.children.is_empty();
        }

        let ch = word.chars().nth(depth).unwrap();
        if let Some(child) = self.children.get_mut(&ch) {
            let should_delete = child.remove_recursive(word, depth + 1);
            if should_delete {
                self.children.remove(&ch);
                return !self.is_end && self.children.is_empty();
            }
        }
        false
    }
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}
