pub fn object_group_by<T: Clone, K: std::hash::Hash + Eq>(
    items: &[T],
    key_selector: impl Fn(&T, usize) -> K,
) -> std::collections::HashMap<K, Vec<T>> {
    let mut result: std::collections::HashMap<K, Vec<T>> = std::collections::HashMap::new();
    for (index, item) in items.iter().enumerate() {
        let key = key_selector(item, index);
        result
            .entry(key)
            .or_insert_with(Vec::new)
            .push(item.clone());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_group_by() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let grouped = object_group_by(&items, |&x, _| x % 2);

        assert_eq!(grouped.get(&0), Some(&vec![2, 4, 6]));
        assert_eq!(grouped.get(&1), Some(&vec![1, 3, 5]));
    }

    #[test]
    fn test_object_group_by_strings() {
        let items = vec!["apple", "banana", "apricot", "blueberry"];
        let grouped = object_group_by(items.as_slice(), |s, _| s.chars().next().unwrap());

        assert_eq!(grouped.get(&'a'), Some(&vec!["apple", "apricot"]));
        assert_eq!(grouped.get(&'b'), Some(&vec!["banana", "blueberry"]));
    }
}
