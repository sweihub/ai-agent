use std::collections::HashMap;

pub struct MetricsCollector {
    counters: HashMap<String, i64>,
    gauges: HashMap<String, f64>,
    histograms: HashMap<String, Vec<f64>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }

    pub fn inc_counter(&mut self, name: &str, delta: i64) {
        *self.counters.entry(name.to_string()).or_insert(0) += delta;
    }

    pub fn set_gauge(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn record_histogram(&mut self, name: &str, value: f64) {
        self.histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }

    pub fn get_counter(&self, name: &str) -> i64 {
        *self.counters.get(name).unwrap_or(&0)
    }

    pub fn get_gauge(&self, name: &str) -> f64 {
        *self.gauges.get(name).unwrap_or(&0.0)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
