use std::collections::HashMap;

pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}

pub struct MemoryCache<K, V> {
    store: HashMap<K, CacheEntry<V>>,
    max_size: usize,
}

impl<K: std::hash::Hash + Eq, V> MemoryCache<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            store: HashMap::new(),
            max_size,
        }
    }

    pub fn set(&mut self, key: K, value: V, ttl_secs: Option<u64>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let expires_at = ttl_secs.map(|secs| now + secs * 1000);

        if self.store.len() >= self.max_size {
            if let Some(oldest_key) = self.store.keys().next().cloned() {
                self.store.remove(&oldest_key);
            }
        }

        self.store.insert(
            key,
            CacheEntry {
                value,
                created_at: now,
                expires_at,
            },
        );
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.store.get(key).and_then(|entry| {
            if let Some(expires_at) = entry.expires_at {
                if now > expires_at {
                    return None;
                }
            }
            Some(&entry.value)
        })
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.store.remove(key).map(|e| e.value)
    }

    pub fn clear(&mut self) {
        self.store.clear();
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}
