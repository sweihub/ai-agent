#![allow(dead_code)]

use std::collections::HashMap;

pub struct Cache<K: std::hash::Hash + Eq, V> {
    data: HashMap<K, V>,
    max_size: usize,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> Cache<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.data.get(key).cloned()
    }

    pub fn set(&mut self, key: K, value: V) {
        if self.data.len() >= self.max_size {
            if let Some(first) = self.data.keys().next().cloned() {
                self.data.remove(&first);
            }
        }
        self.data.insert(key, value);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache() {
        let mut cache = Cache::new(2);
        cache.set("a", 1);
        cache.set("b", 2);
        assert_eq!(cache.get(&"a"), Some(1));
        cache.set("c", 3);
        assert!(cache.get(&"a").is_none() || cache.get(&"b").is_some());
    }
}
