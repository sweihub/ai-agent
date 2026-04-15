#![allow(dead_code)]

use std::collections::HashMap;

pub struct Trie {
    children: HashMap<char, Trie>,
    is_end: bool,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            children: HashMap::new(),
            is_end: false,
        }
    }

    pub fn insert(&mut self, word: &str) {
        let mut node = self;
        for c in word.chars() {
            node = node.children.entry(c).or_insert_with(Trie::new);
        }
        node.is_end = true;
    }

    pub fn search(&self, word: &str) -> bool {
        let mut node = self;
        for c in word.chars() {
            match node.children.get(&c) {
                Some(n) => node = n,
                None => return false,
            }
        }
        node.is_end
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        let mut node = self;
        for c in prefix.chars() {
            match node.children.get(&c) {
                Some(n) => node = n,
                None => return false,
            }
        }
        true
    }
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie() {
        let mut trie = Trie::new();
        trie.insert("hello");
        assert!(trie.search("hello"));
        assert!(!trie.search("hell"));
        assert!(trie.starts_with("hel"));
    }
}
