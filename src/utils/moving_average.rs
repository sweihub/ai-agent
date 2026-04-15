pub struct MovingAverage {
    values: Vec<f64>,
    window_size: usize,
    sum: f64,
}

impl MovingAverage {
    pub fn new(window_size: usize) -> Self {
        Self {
            values: Vec::with_capacity(window_size),
            window_size,
            sum: 0.0,
        }
    }

    pub fn add(&mut self, value: f64) {
        if self.values.len() == self.window_size {
            self.sum -= self.values[0];
            self.values.remove(0);
        }
        self.values.push(value);
        self.sum += value;
    }

    pub fn average(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.sum / self.values.len() as f64
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}
