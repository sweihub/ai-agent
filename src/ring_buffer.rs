#![allow(dead_code)]

pub struct RingBuffer<T> {
    data: Vec<T>,
    capacity: usize,
    head: usize,
    size: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
            head: 0,
            size: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.size < self.capacity {
            self.data.push(value);
            self.size += 1;
        } else {
            self.data[self.head] = value;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }
}
