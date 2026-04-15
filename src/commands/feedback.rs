// Source: /data/home/swei/claudecode/openclaudecode/src/commands/feedback/feedback.tsx
use super::Command;

pub fn create_feedback_command() -> Command {
    Command::local("feedback", "Send feedback").argument_hint("<message>")
}
