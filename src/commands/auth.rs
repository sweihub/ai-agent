// Source: /data/home/swei/claudecode/openclaudecode/src/types/generated/events_mono/common/v1/auth.ts
use super::Command;

pub fn create_login_command() -> Command {
    Command::local("login", "Authenticate with the API")
}

pub fn create_logout_command() -> Command {
    Command::local("logout", "Sign out of the current session")
}

pub fn create_mcp_command() -> Command {
    Command::local("mcp", "Manage MCP servers").argument_hint("[add|remove|list] [<server-name>]")
}
