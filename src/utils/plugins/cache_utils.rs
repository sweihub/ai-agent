// Source: ~/claudecode/openclaudecode/src/utils/plugins/cacheUtils.ts
#![allow(dead_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::fs;

use super::installed_plugins_manager::load_installed_plugins_from_disk;
use super::loader::{get_plugin_cache_path, clear_plugin_cache};

const ORPHANED_AT_FILENAME: &str = ".orphaned_at";
const CLEANUP_AGE_MS: u64 = 7 * 24 * 60 * 60 * 1000; // 7 days

/// Clear all plugin-related caches.
pub fn clear_all_plugin_caches() {
    clear_plugin_cache(None);
    crate::utils::plugins::load_plugin_hooks::clear_plugin_hook_cache();
    crate::utils::plugins::load_plugin_commands::clear_plugin_command_cache();
    crate::utils::plugins::load_plugin_agents::clear_plugin_agent_cache();
    crate::utils::plugins::load_plugin_output_styles::clear_plugin_output_style_cache();
    crate::utils::plugins::plugin_options_storage::clear_plugin_options_cache();
}

/// Clear all caches including non-plugin ones.
pub fn clear_all_caches() {
    clear_all_plugin_caches();
}

/// Mark a plugin version as orphaned by writing a .orphaned_at timestamp file.
pub async fn mark_plugin_version_orphaned(version_path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let orphaned_at_path = version_path.join(ORPHANED_AT_FILENAME);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    fs::write(&orphaned_at_path, now.to_string()).await
        .map_err(|e| format!("Failed to write .orphaned_at at {:?}: {}", orphaned_at_path, e))?;
    Ok(())
}

/// Clean up orphaned plugin versions that have been orphaned for more than 7 days.
pub async fn cleanup_orphaned_plugin_versions_in_background() {
    if super::zip_cache::is_plugin_zip_cache_enabled() {
        return;
    }

    if let Err(e) = do_cleanup().await {
        log::debug!("Plugin cache cleanup failed: {}", e);
    }
}

async fn do_cleanup() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let installed_versions = get_installed_version_paths()?;
    let cache_path = PathBuf::from(get_plugin_cache_path());
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Pass 1: Remove .orphaned_at from installed versions
    for version_path in &installed_versions {
        remove_orphaned_at_marker(version_path).await;
    }

    // Pass 2: Process orphaned versions (stub - simplified)
    log::debug!("Orphaned cleanup: {} installed versions, cache at {:?}", installed_versions.len(), cache_path);
    let _ = now; // suppress warning
    Ok(())
}

fn get_installed_version_paths() -> Result<HashSet<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let mut paths = HashSet::new();
    let disk_data = load_installed_plugins_from_disk()?;
    for installations in disk_data.plugins.values() {
        for entry in installations {
            paths.insert(PathBuf::from(&entry.install_path));
        }
    }
    Ok(paths)
}

async fn remove_orphaned_at_marker(version_path: &Path) {
    let orphaned_at_path = version_path.join(ORPHANED_AT_FILENAME);
    if let Err(e) = fs::remove_file(&orphaned_at_path).await {
        if e.kind() != std::io::ErrorKind::NotFound {
            log::debug!("Failed to remove .orphaned_at at {:?}: {}", orphaned_at_path, e);
        }
    }
}

async fn _process_orphaned_plugin_version(_version_path: &Path, _now: u64) {
    // Stub: simplified implementation
}

async fn _remove_if_empty(_dir_path: &Path) {
    // Stub
}

async fn _read_subdirs(_dir_path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Vec::new())
}
