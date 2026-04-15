pub struct Gauge {
    value: f64,
}

impl Gauge {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }

    pub fn set(&mut self, value: f64) {
        self.value = value;
    }

    pub fn get(&self) -> f64 {
        self.value
    }

    pub fn increment(&mut self, delta: f64) {
        self.value += delta;
    }

    pub fn decrement(&mut self, delta: f64) {
        self.value -= delta;
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}
