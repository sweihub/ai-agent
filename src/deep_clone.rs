#![allow(dead_code)]

pub fn deep_clone<T: Clone>(value: &T) -> T {
    value.clone()
}

pub fn merge_maps<K: std::hash::Hash + Eq + Clone, V: Clone>(
    a: &std::collections::HashMap<K, V>,
    b: &std::collections::HashMap<K, V>,
) -> std::collections::HashMap<K, V> {
    let mut result = a.clone();
    result.extend(b.clone());
    result
}
