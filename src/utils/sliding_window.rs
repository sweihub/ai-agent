pub struct SlidingWindow<T> {
    items: Vec<T>,
    window_size: usize,
}

impl<T> SlidingWindow<T> {
    pub fn new(window_size: usize) -> Self {
        Self {
            items: Vec::with_capacity(window_size),
            window_size,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.items.len() >= self.window_size {
            self.items.remove(0);
        }
        self.items.push(item);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }
}
