#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CostEntry {
    pub timestamp: i64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model: String,
}

pub fn track_cost(entry: CostEntry) {}

pub fn get_total_cost() -> u64 {
    0
}
