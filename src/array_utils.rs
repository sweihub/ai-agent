pub fn intersperse<A, F>(as: &[A], separator: F) -> Vec<A>
where
    F: Fn(usize) -> A,
{
    let mut result = Vec::with_capacity(as.len() * 2 - 1);
    for (i, a) in as.iter().enumerate() {
        if i > 0 {
            result.push(separator(i));
        }
        result.push(a.clone());
    }
    result
}

pub fn count<T, P>(arr: &[T], pred: P) -> usize
where
    P: Fn(&T) -> bool,
{
    arr.iter().filter(|x| pred(x)).count()
}

pub fn uniq<T: Clone + Eq>(xs: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut set = std::collections::HashSet::new();
    let mut result = Vec::new();
    for x in xs {
        if set.insert(x.clone()) {
            result.push(x);
        }
    }
    result
}