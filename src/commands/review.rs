// Source: /data/home/swei/claudecode/openclaudecode/src/commands/review.ts
use super::{is_ultrareview_enabled, Command};

pub fn create_review_command() -> Command {
    Command::prompt("review", "Review a pull request")
}

pub fn create_ultrareview_command() -> Command {
    Command::local(
        "ultrareview",
        "~10–20 min · Finds and verifies bugs in your branch. Runs in Claude Code on the web.",
    )
}
