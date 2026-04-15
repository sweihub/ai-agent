pub struct IdGenerator {
    counter: u64,
    prefix: String,
}

impl IdGenerator {
    pub fn new(prefix: &str) -> Self {
        Self {
            counter: 0,
            prefix: prefix.to_string(),
        }
    }

    pub fn next(&mut self) -> String {
        self.counter += 1;
        format!("{}_{}", self.prefix, self.counter)
    }

    pub fn reset(&mut self) {
        self.counter = 0;
    }
}
