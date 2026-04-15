// Source: /data/home/swei/claudecode/openclaudecode/src/entrypoints/mcp.ts
use super::Command;

pub fn create_mcp_command() -> Command {
    Command::local("mcp", "Manage MCP servers")
        .argument_hint("[list|add|remove|start|stop] [<server-name>]")
}

pub fn create_mcp_add_command() -> Command {
    Command::local("mcp add", "Add an MCP server").argument_hint("<name> <commandOrUrl> [args...]")
}

pub fn create_mcp_remove_command() -> Command {
    Command::local("mcp remove", "Remove an MCP server").argument_hint("<server-name>")
}

pub fn create_mcp_list_command() -> Command {
    Command::local("mcp list", "List MCP servers")
}
