use serde::{Deserialize, Serialize};

pub fn deep_clone<T: Clone>(value: &T) -> T {
    value.clone()
}

pub fn deep_clone_json<T: for<'de> Deserialize<'de> + Serialize>(value: &T) -> Result<T, String> {
    let json = serde_json::to_string(value).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneableMap<K, V> {
    inner: HashMap<K, V>,
}

impl<K: Hash + Eq + Clone, V: Clone> CloneableMap<K, V> {
    pub fn new() -> Self {
        CloneableMap {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.inner.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> Default for CloneableMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
