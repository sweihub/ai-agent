// Source: /data/home/swei/claudecode/openclaudecode/src/commands/theme/theme.tsx
use super::Command;

pub fn create_theme_command() -> Command {
    Command::local("theme", "Change the theme").argument_hint("[theme-name]")
}
