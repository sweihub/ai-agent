use super::Command;

pub fn create_mock_limits_command() -> Command {
    Command::local("mock-limits", "Mock usage limits for testing").argument_hint("[<limits>]")
}
