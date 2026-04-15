use std::collections::HashMap;

pub struct CacheWithTTL<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    max_size: usize,
}

struct CacheEntry<V> {
    value: V,
    expires_at: u64,
}

impl<K: std::hash::Hash + Eq, V> CacheWithTTL<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
        }
    }

    pub fn set(&mut self, key: K, value: V, ttl_secs: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if self.entries.len() >= self.max_size {
            if let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.expires_at)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&oldest);
            }
        }

        self.entries.insert(
            key,
            CacheEntry {
                value,
                expires_at: now + ttl_secs * 1000,
            },
        );
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.entries.get(key).and_then(|e| {
            if now < e.expires_at {
                Some(&e.value)
            } else {
                None
            }
        })
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|e| e.value)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
