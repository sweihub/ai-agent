use std::collections::HashMap;
use std::hash::Hash;

pub struct LruCache<K, V> {
    capacity: usize,
    cache: HashMap<K, V>,
    order: Vec<K>,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        LruCache {
            capacity,
            cache: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
            self.order.push(key.clone());
            self.cache.get(key)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if let Some(pos) = self.order.iter().position(|k| k == &key) {
            self.order.remove(pos);
            self.order.push(key.clone());
            self.cache.insert(key, value);
        } else {
            if self.cache.len() >= self.capacity {
                if let Some(oldest) = self.order.first() {
                    self.cache.remove(oldest);
                    self.order.remove(0);
                }
            }
            self.order.push(key.clone());
            self.cache.insert(key, value);
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
            self.cache.remove(key)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.order.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}
