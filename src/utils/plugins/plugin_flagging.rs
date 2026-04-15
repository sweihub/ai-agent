// Source: ~/claudecode/openclaudecode/src/utils/plugins/pluginFlagging.ts
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use super::plugin_directories::get_plugins_directory;

const FLAGGED_PLUGINS_FILENAME: &str = "flagged-plugins.json";
const SEEN_EXPIRY_MS: u64 = 48 * 60 * 60 * 1000; // 48 hours

/// Flagged plugin tracking data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlaggedPlugin {
    #[serde(rename = "flaggedAt")]
    pub flagged_at: String,
    #[serde(rename = "seenAt", skip_serializing_if = "Option::is_none")]
    pub seen_at: Option<String>,
}

static CACHE: Lazy<Mutex<Option<HashMap<String, FlaggedPlugin>>>> =
    Lazy::new(|| Mutex::new(None));

fn get_flagged_plugins_path() -> PathBuf {
    PathBuf::from(get_plugins_directory()).join(FLAGGED_PLUGINS_FILENAME)
}

/// Load flagged plugins from disk into the module cache.
pub async fn load_flagged_plugins() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let all = read_from_disk().await.unwrap_or_default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let mut changed = false;
    let mut filtered = HashMap::new();

    for (id, entry) in all {
        if let Some(ref seen_at) = entry.seen_at {
            let seen_time = chrono::DateTime::parse_from_rfc3339(seen_at)
                .map(|dt| dt.timestamp_millis() as u64)
                .unwrap_or(0);

            if now.saturating_sub(seen_time) >= SEEN_EXPIRY_MS {
                changed = true;
                continue;
            }
        }
        filtered.insert(id, entry);
    }

    if changed {
        write_to_disk(&filtered).await?;
    }

    {
        let mut cache = CACHE.lock().unwrap();
        *cache = Some(filtered);
    }

    Ok(())
}

async fn read_from_disk() -> Result<HashMap<String, FlaggedPlugin>, Box<dyn std::error::Error + Send + Sync>> {
    let path = get_flagged_plugins_path();
    let content = tokio::fs::read_to_string(&path).await?;
    let data: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(plugins) = data.get("plugins").and_then(|p| p.as_object()) {
        let mut result = HashMap::new();
        for (id, entry) in plugins {
            if let Some(flagged_at) = entry.get("flaggedAt").and_then(|v| v.as_str()) {
                let seen_at = entry.get("seenAt").and_then(|v| v.as_str()).map(|s| s.to_string());
                result.insert(
                    id.clone(),
                    FlaggedPlugin {
                        flagged_at: flagged_at.to_string(),
                        seen_at,
                    },
                );
            }
        }
        Ok(result)
    } else {
        Ok(HashMap::new())
    }
}

async fn write_to_disk(plugins: &HashMap<String, FlaggedPlugin>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = get_flagged_plugins_path();
    let temp_path = PathBuf::from(format!(
        "{}.{}.tmp",
        path.display(),
        rand::random::<u64>()
    ));

    tokio::fs::create_dir_all(get_plugins_directory()).await?;

    let data = serde_json::json!({ "plugins": plugins });
    let content = serde_json::to_string_pretty(&data)?;
    tokio::fs::write(&temp_path, content).await?;
    tokio::fs::rename(&temp_path, &path).await?;

    {
        let mut cache = CACHE.lock().unwrap();
        *cache = Some(plugins.clone());
    }

    Ok(())
}

/// Get all flagged plugins from the in-memory cache.
pub fn get_flagged_plugins() -> HashMap<String, FlaggedPlugin> {
    let cache = CACHE.lock().unwrap();
    cache.clone().unwrap_or_default()
}

/// Add a plugin to the flagged list.
pub async fn add_flagged_plugin(plugin_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut cache = CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(read_from_disk().await.unwrap_or_default());
    }

    let now = chrono::Utc::now().to_rfc3339();
    if let Some(ref mut plugins) = *cache {
        plugins.insert(
            plugin_id.to_string(),
            FlaggedPlugin {
                flagged_at: now,
                seen_at: None,
            },
        );

        write_to_disk(plugins).await?;
        log::debug!("Flagged plugin: {}", plugin_id);
    }

    Ok(())
}

/// Mark flagged plugins as seen.
pub async fn mark_flagged_plugins_seen(plugin_ids: &[String]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut cache = CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(read_from_disk().await.unwrap_or_default());
    }

    let now = chrono::Utc::now().to_rfc3339();
    let mut changed = false;

    if let Some(ref mut plugins) = *cache {
        for id in plugin_ids {
            if let Some(entry) = plugins.get_mut(id) {
                if entry.seen_at.is_none() {
                    entry.seen_at = Some(now.clone());
                    changed = true;
                }
            }
        }
    }

    if changed {
        if let Some(ref plugins) = *cache {
            write_to_disk(plugins).await?;
        }
    }

    Ok(())
}

/// Remove a plugin from the flagged list.
pub async fn remove_flagged_plugin(plugin_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut cache = CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(read_from_disk().await.unwrap_or_default());
    }

    if let Some(ref mut plugins) = *cache {
        if plugins.remove(plugin_id).is_some() {
            write_to_disk(plugins).await?;
        }
    }

    Ok(())
}
