// Source: /data/home/swei/claudecode/openclaudecode/src/utils/fullscreen.ts
use crate::constants::env::ai;

pub fn is_fullscreen_env_enabled() -> bool {
    std::env::var(ai::FULLSCREEN)
        .map(|v| v == "1")
        .unwrap_or(false)
}

pub fn is_terminal_fullscreen() -> bool {
    false
}
