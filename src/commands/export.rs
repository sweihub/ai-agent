// Source: /data/home/swei/claudecode/openclaudecode/src/commands/export/export.tsx
use super::Command;

pub fn create_export_command() -> Command {
    Command::local("export", "Export session").argument_hint("[json|markdown] [<path>]")
}
