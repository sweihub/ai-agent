//! Query profiler utilities.

use std::time::{Duration, Instant};

/// Query profiler for tracking query performance
pub struct QueryProfiler {
    start_time: Instant,
    queries: Vec<QueryProfileEntry>,
}

#[derive(Debug, Clone)]
pub struct QueryProfileEntry {
    query: String,
    duration: Duration,
    success: bool,
}

impl QueryProfiler {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            queries: Vec::new(),
        }
    }

    pub fn record(&mut self, query: String, duration: Duration, success: bool) {
        self.queries.push(QueryProfileEntry {
            query,
            duration,
            success,
        });
    }

    pub fn get_total_queries(&self) -> usize {
        self.queries.len()
    }

    pub fn get_successful_queries(&self) -> usize {
        self.queries.iter().filter(|q| q.success).count()
    }

    pub fn get_total_duration(&self) -> Duration {
        self.queries.iter().map(|q| q.duration).sum()
    }

    pub fn get_average_duration(&self) -> Duration {
        let count = self.queries.len();
        if count == 0 {
            Duration::ZERO
        } else {
            self.get_total_duration() / count as u32
        }
    }

    pub fn get_slowest_query(&self) -> Option<&QueryProfileEntry> {
        self.queries.iter().max_by_key(|q| q.duration)
    }
}

impl Default for QueryProfiler {
    fn default() -> Self {
        Self::new()
    }
}
