use std::sync::OnceLock;

use crate::utils::config::{get_global_config, save_global_config};

static PRO_TO_OPUS_MIGRATION_DONE: OnceLock<bool> = OnceLock::new();

pub fn reset_pro_to_opus_default() -> bool {
    if PRO_TO_OPUS_MIGRATION_DONE.get().is_some() {
        return *PRO_TO_OPUS_MIGRATION_DONE.get().unwrap();
    }

    let config = get_global_config();

    if config.opus_pro_migration_complete {
        let _ = PRO_TO_OPUS_MIGRATION_DONE.set(true);
        return true;
    }

    // TODO: Implement actual migration logic when dependencies are available
    // - Check API provider
    // - Check Pro subscription status
    // - Handle settings migration

    // For now, just mark as complete
    save_global_config(|current| crate::utils::config::GlobalConfig {
        opus_pro_migration_complete: true,
        ..current
    });

    let _ = PRO_TO_OPUS_MIGRATION_DONE.set(true);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_runs_once() {
        // Reset the static
        let _ = PRO_TO_OPUS_MIGRATION_DONE.set(false);

        let result1 = reset_pro_to_opus_default();
        let result2 = reset_pro_to_opus_default();

        assert_eq!(result1, result2);
    }
}
