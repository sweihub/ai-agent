use super::Command;

pub fn create_debug_tool_call_command() -> Command {
    Command::local("debug-tool-call", "Debug tool call").argument_hint("<tool-call-id>")
}
