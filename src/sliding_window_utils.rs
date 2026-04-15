use std::collections::VecDeque;

pub struct SlidingWindow<T> {
    window: VecDeque<T>,
    capacity: usize,
}

impl<T> SlidingWindow<T> {
    pub fn new(capacity: usize) -> Self {
        SlidingWindow {
            window: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.window.len() >= self.capacity {
            self.window.pop_front();
        }
        self.window.push_back(item);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.window.get(index)
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }

    pub fn clear(&mut self) {
        self.window.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.window.iter()
    }

    pub fn to_vec(&self) -> Vec<&T> {
        self.window.iter().collect()
    }

    pub fn front(&self) -> Option<&T> {
        self.window.front()
    }

    pub fn back(&self) -> Option<&T> {
        self.window.back()
    }
}

impl<T: Clone> SlidingWindow<T> {
    pub fn latest(&self) -> Option<T> {
        self.window.back().cloned()
    }

    pub fn oldest(&self) -> Option<T> {
        self.window.front().cloned()
    }
}
