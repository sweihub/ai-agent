// Source: /data/home/swei/claudecode/openclaudecode/src/commands/resume/resume.tsx
use super::Command;

pub fn create_resume_command() -> Command {
    Command::local("resume", "Resume a previous session").argument_hint("<session-id>")
}

pub fn create_fork_command() -> Command {
    Command::local("fork", "Fork the current conversation").argument_hint("[<name>]")
}
