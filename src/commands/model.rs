// Source: /data/home/swei/claudecode/openclaudecode/src/utils/model/model.ts
use crate::constants::env::ai;
use super::{Command, CommandCallResult};

pub fn create_model_command() -> Command {
    Command::prompt("model", "Set the AI model for Claude Code").argument_hint("[model]")
}

pub fn get_main_loop_model() -> String {
    std::env::var(ai::MODEL).unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string())
}

pub fn render_model_name(model: &str) -> String {
    model.to_string()
}
