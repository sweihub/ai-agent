use crate::constants::env::system;

pub fn is_running_with_bun() -> bool {
    std::env::var(system::BUN_VERSION).is_ok()
}

pub fn is_in_bundled_mode() -> bool {
    std::env::var(system::BUN_EMBEDDED).is_ok()
}
