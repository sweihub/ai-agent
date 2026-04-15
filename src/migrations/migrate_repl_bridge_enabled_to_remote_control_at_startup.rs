use std::sync::OnceLock;

static MIGRATION_DONE: OnceLock<bool> = OnceLock::new();

pub fn migrate_repl_bridge_enabled() -> bool {
    if MIGRATION_DONE.get().is_some() {
        return *MIGRATION_DONE.get().unwrap();
    }

    let _ = MIGRATION_DONE.set(true);
    true
}
