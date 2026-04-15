// Source: /data/home/swei/claudecode/openclaudecode/src/utils/set.ts
//! Set utility functions optimized for performance.
//! This code is hot, so it's optimized for speed.

/// Returns the difference of two sets (elements in `a` but not in `b`).
pub fn difference<T: std::hash::Hash + Eq + Clone>(
    a: &std::collections::HashSet<T>,
    b: &std::collections::HashSet<T>,
) -> std::collections::HashSet<T> {
    a.iter().filter(|item| !b.contains(item)).cloned().collect()
}

/// Returns true if sets intersect (have any common elements).
pub fn intersects<T: std::hash::Hash + Eq>(
    a: &std::collections::HashSet<T>,
    b: &std::collections::HashSet<T>,
) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    a.iter().any(|item| b.contains(item))
}

/// Returns true if every element in `a` is also in `b`.
pub fn every<T: std::hash::Hash + Eq>(
    a: &std::collections::HashSet<T>,
    b: &std::collections::HashSet<T>,
) -> bool {
    a.iter().all(|item| b.contains(item))
}

/// Returns the union of two sets.
pub fn union<T: std::hash::Hash + Eq + Clone>(
    a: &std::collections::HashSet<T>,
    b: &std::collections::HashSet<T>,
) -> std::collections::HashSet<T> {
    let mut result = a.clone();
    result.extend(b.iter().cloned());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difference() {
        let mut a = std::collections::HashSet::new();
        a.insert(1);
        a.insert(2);
        a.insert(3);

        let mut b = std::collections::HashSet::new();
        b.insert(2);
        b.insert(4);

        let result = difference(&a, &b);
        assert!(result.contains(&1));
        assert!(result.contains(&3));
        assert!(!result.contains(&2));
    }

    #[test]
    fn test_intersects() {
        let mut a = std::collections::HashSet::new();
        a.insert(1);
        a.insert(2);

        let mut b = std::collections::HashSet::new();
        b.insert(2);
        b.insert(3);

        assert!(intersects(&a, &b));

        let mut c = std::collections::HashSet::new();
        c.insert(4);

        assert!(!intersects(&a, &c));
    }

    #[test]
    fn test_every() {
        let mut a = std::collections::HashSet::new();
        a.insert(1);
        a.insert(2);

        let mut b = std::collections::HashSet::new();
        b.insert(1);
        b.insert(2);
        b.insert(3);

        assert!(every(&a, &b));

        let mut c = std::collections::HashSet::new();
        c.insert(1);
        c.insert(4);

        assert!(!every(&a, &c));
    }

    #[test]
    fn test_union() {
        let mut a = std::collections::HashSet::new();
        a.insert(1);
        a.insert(2);

        let mut b = std::collections::HashSet::new();
        b.insert(2);
        b.insert(3);

        let result = union(&a, &b);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
    }
}
