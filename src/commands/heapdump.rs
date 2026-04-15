// Source: /data/home/swei/claudecode/openclaudecode/src/commands/heapdump/heapdump.ts
use super::Command;

pub fn create_heapdump_command() -> Command {
    Command::local("heapdump", "Generate heap dump")
}
