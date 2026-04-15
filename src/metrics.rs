//! Metrics types.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histograms: HashMap<String, Vec<f64>>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }
}

impl Metrics {
    pub fn increment(&mut self, name: &str, value: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += value;
    }

    pub fn gauge(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn histogram(&mut self, name: &str, value: f64) {
        self.histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }
}
