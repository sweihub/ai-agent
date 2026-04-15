// Source: /data/home/swei/claudecode/openclaudecode/src/commands/stats/stats.tsx
use super::Command;

pub fn create_stats_command() -> Command {
    Command::local(
        "stats",
        "Show your Claude Code usage statistics and activity",
    )
}
