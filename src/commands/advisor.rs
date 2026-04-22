// Source: /data/home/swei/claudecode/openclaudecode/src/commands/advisor.ts
use super::Command;
use crate::constants::env::ai;

pub fn create_advisor_command() -> Command {
    Command::local("advisor", "Configure the advisor model").argument_hint("[<model>|off]")
}

pub fn can_user_configure_advisor() -> bool {
    std::env::var(ai::USER_TYPE)
        .map(|v| v == "ant")
        .unwrap_or(false)
}

pub fn is_valid_advisor_model(model: &str) -> bool {
    matches!(model, "opus" | "sonnet" | "haiku")
}

pub fn model_supports_advisor(model: &str) -> bool {
    matches!(model, "opus" | "sonnet")
}
