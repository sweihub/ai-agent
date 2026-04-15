// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/claude.ts
use super::Command;

pub fn create_good_claude_command() -> Command {
    Command::prompt("good-claude", "Find good Claude behavior patterns")
}

pub fn create_bughunter_command() -> Command {
    Command::local("bughunter", "Run bug hunting").argument_hint("[<target>]")
}

pub fn create_break_cache_command() -> Command {
    Command::local("break-cache", "Break context cache")
}

pub fn create_backfill_sessions_command() -> Command {
    Command::local("backfill-sessions", "Backfill session data")
}

pub fn create_autofix_pr_command() -> Command {
    Command::local("autofix-pr", "Auto-fix PR").argument_hint("<pr-number>")
}
