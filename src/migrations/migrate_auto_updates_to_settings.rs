use std::sync::OnceLock;

use crate::utils::config::{get_global_config, save_global_config};

static AUTO_UPDATES_MIGRATION_DONE: OnceLock<bool> = OnceLock::new();

pub fn migrate_auto_updates_to_settings() {
    if AUTO_UPDATES_MIGRATION_DONE.get().is_some() {
        return;
    }

    let global_config = get_global_config();

    if global_config.auto_updates != Some(false)
        || global_config.auto_updates_protected_for_native == Some(true)
    {
        let _ = AUTO_UPDATES_MIGRATION_DONE.set(true);
        return;
    }

    let _ = AUTO_UPDATES_MIGRATION_DONE.set(true);
}
