pub struct Counter {
    value: i64,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn decrement(&mut self) {
        self.value -= 1;
    }

    pub fn get(&self) -> i64 {
        self.value
    }

    pub fn reset(&mut self) -> i64 {
        let old = self.value;
        self.value = 0;
        old
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}
