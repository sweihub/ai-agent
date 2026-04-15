//! A fixed-size circular buffer that automatically evicts the oldest items
//! when the buffer is full. Useful for maintaining a rolling window of data.

use std::collections::VecDeque;

/// A fixed-size circular buffer that automatically evicts the oldest items
/// when the buffer is full. Useful for maintaining a rolling window of data.
pub struct CircularBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T> CircularBuffer<T> {
    /// Create a new circular buffer with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Add an item to the buffer. If the buffer is full,
    /// the oldest item will be evicted.
    pub fn add(&mut self, item: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(item);
    }

    /// Add multiple items to the buffer at once.
    pub fn add_all(&mut self, items: impl IntoIterator<Item = T>) {
        for item in items {
            self.add(item);
        }
    }

    /// Get the most recent N items from the buffer.
    /// Returns fewer items if the buffer contains less than N items.
    pub fn get_recent(&self, count: usize) -> Vec<&T> {
        self.buffer.iter().rev().take(count).collect::<Vec<_>>()
    }

    /// Get all items currently in the buffer, in order from oldest to newest.
    pub fn to_vec(&self) -> Vec<&T> {
        self.buffer.iter().collect()
    }

    /// Clear all items from the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get the current number of items in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl<T: Clone> Clone for CircularBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            capacity: self.capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut buffer = CircularBuffer::new(3);
        assert_eq!(buffer.len(), 0);

        buffer.add(1);
        buffer.add(2);
        buffer.add(3);
        assert_eq!(buffer.len(), 3);

        // Adding another item should evict the oldest
        buffer.add(4);
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.to_vec(), vec![&2, &3, &4]);
    }

    #[test]
    fn test_get_recent() {
        let mut buffer = CircularBuffer::new(5);
        for i in 1..=10 {
            buffer.add(i);
        }

        let recent = buffer.get_recent(3);
        assert_eq!(recent, vec![&10, &9, &8]);
    }

    #[test]
    fn test_clear() {
        let mut buffer = CircularBuffer::new(3);
        buffer.add(1);
        buffer.add(2);
        buffer.clear();
        assert_eq!(buffer.len(), 0);
    }
}
