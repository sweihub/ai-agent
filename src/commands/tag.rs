// Source: /data/home/swei/claudecode/openclaudecode/src/commands/tag/tag.tsx
use super::Command;

pub fn create_tag_command() -> Command {
    Command::local("tag", "Toggle a searchable tag on the current session")
        .argument_hint("<tag-name>")
}
