use std::sync::OnceLock;

static MIGRATION_DONE: OnceLock<bool> = OnceLock::new();

pub fn migrate_opus_to_opus1m() -> bool {
    if MIGRATION_DONE.get().is_some() {
        return *MIGRATION_DONE.get().unwrap();
    }

    let _ = MIGRATION_DONE.set(true);
    true
}
