// Source: /data/home/swei/claudecode/openclaudecode/src/commands/usage/usage.tsx
use super::Command;

pub fn create_usage_command() -> Command {
    Command::local("usage", "Show token usage")
}

pub fn create_extra_usage_command() -> Command {
    Command::local("extra-usage", "Manage extra usage").argument_hint("[buy|manage]")
}

pub fn create_cost_command() -> Command {
    Command::prompt("cost", "Estimate cost of the conversation")
}

pub fn create_privacy_settings_command() -> Command {
    Command::local("privacy-settings", "Configure privacy settings")
}

pub fn create_rate_limit_options_command() -> Command {
    Command::local("rate-limit-options", "Configure rate limiting")
}
