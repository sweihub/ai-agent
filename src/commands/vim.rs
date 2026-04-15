// Source: /data/home/swei/claudecode/openclaudecode/src/commands/vim/vim.ts
use super::Command;

pub fn create_vim_command() -> Command {
    Command::local("vim", "Toggle Vim mode")
}
