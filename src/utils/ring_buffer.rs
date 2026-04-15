pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    read_index: usize,
    write_index: usize,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![None; capacity],
            read_index: 0,
            write_index: 0,
            capacity,
        }
    }

    pub fn write(&mut self, item: T) -> bool {
        let next = (self.write_index + 1) % self.capacity;
        if next == self.read_index {
            return false;
        }
        self.buffer[self.write_index] = Some(item);
        self.write_index = next;
        true
    }

    pub fn read(&mut self) -> Option<T> {
        if self.read_index == self.write_index {
            return None;
        }
        let item = self.buffer[self.read_index].take();
        self.read_index = (self.read_index + 1) % self.capacity;
        item
    }

    pub fn is_empty(&self) -> bool {
        self.read_index == self.write_index
    }

    pub fn is_full(&self) -> bool {
        (self.write_index + 1) % self.capacity == self.read_index
    }

    pub fn len(&self) -> usize {
        if self.write_index >= self.read_index {
            self.write_index - self.read_index
        } else {
            self.capacity - self.read_index + self.write_index
        }
    }
}
