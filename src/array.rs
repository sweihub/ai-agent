// Source: /data/home/swei/claudecode/openclaudecode/src/utils/array.ts
pub fn intersperse<A: Clone, F: Fn(usize) -> A>(as: &[A], separator: F) -> Vec<A> {
    let mut result = Vec::with_capacity(as.len() * 2);
    for (i, a) in as.iter().enumerate() {
        if i > 0 {
            result.push(separator(i));
        }
        result.push(a.clone());
    }
    result
}

pub fn count<T, F>(arr: &[T], pred: F) -> usize 
where
    F: Fn(&T) -> bool,
{
    arr.iter().filter(|x| pred(x)).count()
}

pub fn uniq<T: std::hash::Hash + Eq + Clone>(xs: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut set = std::collections::HashSet::new();
    let mut result = Vec::new();
    for x in xs {
        if set.insert(x.clone()) {
            result.push(x);
        }
    }
    result
}