use std::collections::HashSet;

pub fn difference<T: std::hash::Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
    a.iter()
        .filter(|item| !b.contains(*item))
        .cloned()
        .collect()
}

pub fn intersects<T: std::hash::Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    a.iter().any(|item| b.contains(item))
}

pub fn is_subset<T: std::hash::Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> bool {
    a.iter().all(|item| b.contains(item))
}

pub fn union<T: std::hash::Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
    let mut result = a.clone();
    result.extend(b.iter().cloned());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difference() {
        let a: HashSet<i32> = [1, 2, 3, 4].into_iter().collect();
        let b: HashSet<i32> = [2, 4].into_iter().collect();
        let diff = difference(&a, &b);
        assert!(diff.contains(&1));
        assert!(diff.contains(&3));
        assert_eq!(diff.len(), 2);
    }

    #[test]
    fn test_intersects() {
        let a: HashSet<i32> = [1, 2, 3].into_iter().collect();
        let b: HashSet<i32> = [3, 4, 5].into_iter().collect();
        assert!(intersects(&a, &b));

        let c: HashSet<i32> = [1, 2].into_iter().collect();
        let d: HashSet<i32> = [3, 4].into_iter().collect();
        assert!(!intersects(&c, &d));
    }

    #[test]
    fn test_is_subset() {
        let a: HashSet<i32> = [1, 2].into_iter().collect();
        let b: HashSet<i32> = [1, 2, 3].into_iter().collect();
        assert!(is_subset(&a, &b));
        assert!(!is_subset(&b, &a));
    }

    #[test]
    fn test_union() {
        let a: HashSet<i32> = [1, 2].into_iter().collect();
        let b: HashSet<i32> = [2, 3, 4].into_iter().collect();
        let u = union(&a, &b);
        assert_eq!(u.len(), 4);
        assert!(u.contains(&1));
        assert!(u.contains(&2));
        assert!(u.contains(&3));
        assert!(u.contains(&4));
    }
}
