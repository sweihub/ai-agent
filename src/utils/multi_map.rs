pub struct MultiMap<K, V> {
    map: std::collections::HashMap<K, Vec<V>>,
}

impl<K: std::hash::Hash + Eq, V> MultiMap<K, V> {
    pub fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.map.entry(key).or_insert_with(Vec::new).push(value);
    }

    pub fn get(&self, key: &K) -> Option<&Vec<V>> {
        self.map.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<Vec<V>> {
        self.map.remove(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

impl<K, V> Default for MultiMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
