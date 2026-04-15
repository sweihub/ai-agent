//! Assistant mode functionality

use crate::constants::env::ai;

/// Check if assistant mode flag is set via environment variable
fn read_assistant_mode_flag() -> bool {
    std::env::var(ai::CODE_ASSISTANT_MODE)
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

pub fn is_assistant_mode() -> bool {
    read_assistant_mode_flag()
}

pub fn is_assistant_mode_enabled() -> bool {
    read_assistant_mode_flag()
}
