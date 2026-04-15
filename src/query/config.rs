// Source: /data/home/swei/claudecode/openclaudecode/src/services/autoDream/config.ts
use crate::constants::env::{ai, ai_code};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct QueryConfig {
    pub session_id: String,
    pub gates: QueryGates,
}

#[derive(Debug, Clone)]
pub struct QueryGates {
    pub streaming_tool_execution: bool,
    pub emit_tool_use_summaries: bool,
    pub is_ant: bool,
    pub fast_mode_enabled: bool,
}

impl QueryConfig {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            gates: QueryGates::default(),
        }
    }
}

impl Default for QueryGates {
    fn default() -> Self {
        Self {
            streaming_tool_execution: false,
            emit_tool_use_summaries: false,
            is_ant: std::env::var(ai::USER_TYPE).unwrap_or_default() == "ant",
            fast_mode_enabled: !std::env::var(ai_code::DISABLE_FAST_MODE)
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false),
        }
    }
}
