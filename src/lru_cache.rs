#![allow(dead_code)]

pub struct Cache<K, V> {
    data: std::collections::HashMap<K, V>,
    max_size: usize,
}

impl<K: std::hash::Hash + Eq, V> Cache<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: std::collections::HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: K, value: V) {
        if self.data.len() >= self.max_size {
            if let Some(first) = self.data.keys().next().cloned() {
                self.data.remove(&first);
            }
        }
        self.data.insert(key, value);
    }
}
