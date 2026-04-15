use super::Command;

pub fn create_session_command() -> Command {
    Command::local("session", "Manage sessions")
        .argument_hint("[list|new|switch|delete] [<session-id>]")
}

pub fn create_stats_command() -> Command {
    Command::local("stats", "Show session statistics").supports_non_interactive(true)
}

pub fn create_status_command() -> Command {
    Command::local("status", "Show current status").supports_non_interactive(true)
}
