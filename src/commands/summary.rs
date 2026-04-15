use super::Command;

pub fn create_summary_command() -> Command {
    Command::prompt("summary", "Summarize conversation")
}

pub fn create_reset_limits_command() -> Command {
    Command::local("reset-limits", "Reset usage limits")
}

pub fn create_mock_limits_command() -> Command {
    Command::local("mock-limits", "Mock usage limits for testing").argument_hint("[<limits>]")
}

pub fn create_oauth_refresh_command() -> Command {
    Command::local("oauth-refresh", "Refresh OAuth token")
}

pub fn create_perf_issue_command() -> Command {
    Command::local("perf-issue", "Report performance issue").argument_hint("<description>")
}
