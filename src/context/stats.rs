// Source: /data/home/swei/claudecode/openclaudecode/src/commands/stats/stats.tsx
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    pub total_messages: u64,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub api_calls: u64,
    pub tool_uses: u64,
    pub session_duration_ms: u64,
}

impl Stats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_messages(&mut self) {
        self.total_messages += 1;
    }

    pub fn add_tokens(&mut self, tokens: u64) {
        self.total_tokens += tokens;
    }

    pub fn add_cost(&mut self, cost: f64) {
        self.total_cost_usd += cost;
    }

    pub fn increment_api_calls(&mut self) {
        self.api_calls += 1;
    }

    pub fn increment_tool_uses(&mut self) {
        self.tool_uses += 1;
    }

    pub fn update_duration(&mut self, duration_ms: u64) {
        self.session_duration_ms = duration_ms;
    }
}
