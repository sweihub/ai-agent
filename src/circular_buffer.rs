pub struct CircularBuffer<T> {
    buffer: Vec<T>,
    head: usize,
    size: usize,
    capacity: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            head: 0,
            size: 0,
            capacity,
        }
    }

    pub fn add(&mut self, item: T) {
        if self.size < self.capacity {
            self.buffer.push(item);
            self.size += 1;
        } else {
            self.buffer[self.head] = item;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    pub fn add_all(&mut self, items: &[T]) {
        for item in items {
            self.add(item.clone());
        }
    }

    pub fn get_recent(&self, count: usize) -> Vec<&T> {
        if self.size == 0 {
            return vec![];
        }

        let start = if self.size < self.capacity {
            0
        } else {
            self.head
        };
        let available = count.min(self.size);
        let mut result = Vec::with_capacity(available);

        for i in 0..available {
            let index = (start + self.size - available + i) % self.capacity;
            result.push(&self.buffer[index]);
        }

        result
    }

    pub fn to_array(&self) -> Vec<&T> {
        if self.size == 0 {
            return vec![];
        }

        let start = if self.size < self.capacity {
            0
        } else {
            self.head
        };
        let mut result = Vec::with_capacity(self.size);

        for i in 0..self.size {
            let index = (start + i) % self.capacity;
            result.push(&self.buffer[index]);
        }

        result
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.head = 0;
        self.size = 0;
    }

    pub fn length(&self) -> usize {
        self.size
    }
}
