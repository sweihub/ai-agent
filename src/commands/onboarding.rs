use super::Command;

pub fn create_onboarding_command() -> Command {
    Command::local("onboarding", "Start onboarding")
}

pub fn create_env_command() -> Command {
    Command::local("env", "Show environment")
}

pub fn create_debug_tool_call_command() -> Command {
    Command::local("debug-tool-call", "Debug tool call").argument_hint("<tool-call-id>")
}

pub fn create_ctx_viz_command() -> Command {
    Command::local("ctx-viz", "Visualize context")
}

pub fn create_issue_command() -> Command {
    Command::local("issue", "Manage issues").argument_hint("[create|list|close] [<issue-id>]")
}
