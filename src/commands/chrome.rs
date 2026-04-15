// Source: /data/home/swei/claudecode/openclaudecode/src/commands/chrome/chrome.tsx
use super::Command;

pub fn create_chrome_command() -> Command {
    Command::local("chrome", "Open Chrome")
}
