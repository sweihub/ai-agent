pub struct Percentile {
    values: Vec<f64>,
}

impl Percentile {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn add(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn get(&self, percentile: f64) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }

        let mut sorted = self.values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let idx = ((percentile / 100.0) * (sorted.len() - 1) as f64) as usize;
        sorted[idx]
    }
}

impl Default for Percentile {
    fn default() -> Self {
        Self::new()
    }
}
