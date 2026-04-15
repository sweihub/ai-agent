// Source: /data/home/swei/claudecode/openclaudecode/src/components/design-system/color.ts
use super::Command;

pub fn create_color_command() -> Command {
    Command::local("color", "Set the prompt bar color for this session")
        .argument_hint("<color|default>")
}
