pub struct Pool<T> {
    items: Vec<T>,
    max_size: usize,
}

impl<T> Pool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn acquire(&mut self, factory: impl FnOnce() -> T) -> T {
        self.items.pop().unwrap_or_else(factory)
    }

    pub fn release(&mut self, item: T) {
        if self.items.len() < self.max_size {
            self.items.push(item);
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn available(&self) -> usize {
        self.items.len()
    }
}
