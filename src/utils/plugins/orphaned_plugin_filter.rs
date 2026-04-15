// Source: ~/claudecode/openclaudecode/src/utils/plugins/orphanedPluginFilter.rs
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use once_cell::sync::Lazy;

use super::plugin_directories::get_plugins_directory;

const ORPHANED_AT_FILENAME: &str = ".orphaned_at";

/// Session-scoped cache. Frozen once computed.
static CACHED_EXCLUSIONS: Lazy<Mutex<Option<Vec<String>>>> =
    Lazy::new(|| Mutex::new(None));

/// Get ripgrep glob exclusion patterns for orphaned plugin versions.
pub async fn get_glob_exclusions_for_plugin_cache(
    search_path: Option<&str>,
) -> Vec<String> {
    let cache_path = PathBuf::from(get_plugins_directory()).join("cache");

    // If search_path is provided, check if it overlaps the cache
    if let Some(sp) = search_path {
        if !paths_overlap(sp, &cache_path) {
            return Vec::new();
        }
    }

    // Return cached exclusions if available
    {
        let exclusions = CACHED_EXCLUSIONS.lock().unwrap();
        if let Some(ref cached) = *exclusions {
            return cached.clone();
        }
    }

    // Find all .orphaned_at files within the plugin cache directory
    let exclusions = match find_orphaned_markers(&cache_path).await {
        Ok(markers) => markers
            .iter()
            .map(|marker_path| {
                let version_dir = marker_path.parent().unwrap_or(Path::new(""));
                let rel = version_dir
                    .strip_prefix(&cache_path)
                    .unwrap_or(version_dir);
                let posix_relative = rel
                    .to_string_lossy()
                    .replace('\\', "/");
                format!("!**/{}/**", posix_relative)
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    // Cache the result
    {
        let mut cached = CACHED_EXCLUSIONS.lock().unwrap();
        *cached = Some(exclusions.clone());
    }

    exclusions
}

async fn find_orphaned_markers(cache_path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let mut markers = Vec::new();

    // Walk the cache directory looking for .orphaned_at files
    // Depth limited to 4: cache/<marketplace>/<plugin>/<version>/.orphaned_at
    let mut stack: Vec<(PathBuf, u32)> = vec![(cache_path.to_path_buf(), 0)];
    while let Some((dir, depth)) = stack.pop() {
        if depth >= 4 {
            continue;
        }
        if let Ok(mut entries) = tokio::fs::read_dir(&dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if let Ok(ft) = entry.file_type().await {
                    if ft.is_dir() {
                        let orphaned_path = path.join(ORPHANED_AT_FILENAME);
                        if orphaned_path.exists() {
                            markers.push(orphaned_path);
                        }
                        stack.push((path, depth + 1));
                    }
                }
            }
        }
    }

    Ok(markers)
}

/// Clear the plugin cache exclusions.
pub fn clear_plugin_cache_exclusions() {
    let mut exclusions = CACHED_EXCLUSIONS.lock().unwrap();
    *exclusions = None;
}

/// Check if one path is a prefix of the other.
fn paths_overlap(a: &str, b: &Path) -> bool {
    let na = normalize_for_compare(a);
    let nb = normalize_for_compare(&b.to_string_lossy());

    na == nb
        || na == std::path::MAIN_SEPARATOR.to_string()
        || nb == std::path::MAIN_SEPARATOR.to_string()
        || na.starts_with(&(nb.clone() + &std::path::MAIN_SEPARATOR.to_string()))
        || nb.starts_with(&(na.clone() + &std::path::MAIN_SEPARATOR.to_string()))
}

fn normalize_for_compare(p: &str) -> String {
    // On Windows, normalize to lowercase for case-insensitive comparison
    if cfg!(windows) {
        p.to_lowercase()
    } else {
        p.to_string()
    }
}
