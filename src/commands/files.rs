// Source: /data/home/swei/claudecode/openclaudecode/src/commands/files/files.ts
use super::Command;

pub fn create_files_command() -> Command {
    Command::local("files", "List files in session")
}

pub fn create_context_command() -> Command {
    Command::local("context", "Show context usage")
}

pub fn create_copy_command() -> Command {
    Command::local("copy", "Copy to clipboard").argument_hint("<text>")
}

pub fn create_export_command() -> Command {
    Command::local("export", "Export session").argument_hint("[json|markdown] [<path>]")
}

pub fn create_share_command() -> Command {
    Command::local("share", "Share session")
}
