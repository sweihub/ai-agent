use std::sync::OnceLock;

static MIGRATION_DONE: OnceLock<bool> = OnceLock::new();

pub fn reset_auto_mode_opt_in() -> bool {
    if MIGRATION_DONE.get().is_some() {
        return *MIGRATION_DONE.get().unwrap();
    }

    let _ = MIGRATION_DONE.set(true);
    true
}
