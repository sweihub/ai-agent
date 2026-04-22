use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Default)]
pub struct CostState {
    pub total_cents: u64,
    pub api_call_count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl CostState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_api_cost(&mut self, input_tokens: u64, output_tokens: u64, cost_per_million: f64) {
        self.api_call_count += 1;
        self.input_tokens += input_tokens;
        self.output_tokens += output_tokens;
        let input_cost = (input_tokens as f64 / 1_000_000.0) * cost_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * cost_per_million;
        let total_cost_cents = ((input_cost + output_cost) * 100.0).round() as u64;
        self.total_cents += total_cost_cents;
    }
}

#[derive(Debug, Default, Clone)]
pub struct CostAccumulator {
    state: Arc<AtomicU64>,
}

impl CostAccumulator {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn add(&self, cents: u64) {
        self.state.fetch_add(cents, Ordering::Relaxed);
    }

    pub fn total(&self) -> u64 {
        self.state.load(Ordering::Relaxed)
    }

    pub fn total_dollars(&self) -> f64 {
        self.total() as f64 / 100.0
    }

    pub fn reset(&self) {
        self.state.store(0, Ordering::Relaxed);
    }
}

pub fn format_cost(cents: u64) -> String {
    let dollars = cents as f64 / 100.0;
    if dollars >= 1.0 {
        format!("${:.2}", dollars)
    } else {
        format!("{}¢", cents)
    }
}

pub fn format_total_cost(
    input_tokens: u64,
    output_tokens: u64,
    api_calls: u64,
    total_cents: u64,
) -> String {
    let total_tokens = input_tokens + output_tokens;
    let dollars = total_cents as f64 / 100.0;
    format!(
        "Session Costs:\n  API Calls: {}\n  Input Tokens: {}\n  Output Tokens: {}\n  Total Tokens: {}\n  Total Cost: ${:.4}",
        api_calls, input_tokens, output_tokens, total_tokens, dollars
    )
}

pub fn calculate_cost(
    input_tokens: u64,
    output_tokens: u64,
    input_cost_per_million: f64,
    output_cost_per_million: f64,
) -> u64 {
    let input_cost = (input_tokens as f64 / 1_000_000.0) * input_cost_per_million;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * output_cost_per_million;
    ((input_cost + output_cost) * 100.0).round() as u64
}

pub mod pricing {
    pub const OPUS_INPUT: f64 = 15.0;
    pub const OPUS_OUTPUT: f64 = 75.0;
    pub const SONNET_INPUT: f64 = 3.0;
    pub const SONNET_OUTPUT: f64 = 15.0;
    pub const HAIKU_INPUT: f64 = 0.8;
    pub const HAIKU_OUTPUT: f64 = 4.0;

    pub fn get_pricing(model_id: &str) -> Option<(f64, f64)> {
        let model = model_id.to_lowercase();
        if model.contains("opus") {
            Some((OPUS_INPUT, OPUS_OUTPUT))
        } else if model.contains("sonnet") {
            Some((SONNET_INPUT, SONNET_OUTPUT))
        } else if model.contains("haiku") {
            Some((HAIKU_INPUT, HAIKU_OUTPUT))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct CostSummary {
    pub total_cents: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub api_calls: u64,
}

impl CostSummary {
    pub fn format(&self) -> String {
        format_total_cost(
            self.input_tokens,
            self.output_tokens,
            self.api_calls,
            self.total_cents,
        )
    }
}

pub fn has_console_billing_access() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_state() {
        let mut state = CostState::new();
        state.add_api_cost(1000, 500, 3.0);
        assert_eq!(state.api_call_count, 1);
    }

    #[test]
    fn test_format_cost() {
        assert_eq!(format_cost(150), "$1.50");
    }
}
