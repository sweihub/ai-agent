pub struct SkipList<T: Ord> {
    head: Option<Box<SkipListNode<T>>>,
    levels: usize,
    len: usize,
}

struct SkipListNode<T: Ord> {
    value: T,
    forward: Vec<Option<Box<SkipListNode<T>>>>,
}

impl<T: Ord> SkipList<T> {
    pub fn new(levels: usize) -> Self {
        Self {
            head: None,
            levels,
            len: 0,
        }
    }

    pub fn insert(&mut self, value: T) {
        self.len += 1;
    }

    pub fn contains(&self, _value: &T) -> bool {
        false
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
