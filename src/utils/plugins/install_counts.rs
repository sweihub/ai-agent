// Source: ~/claudecode/openclaudecode/src/utils/plugins/installCounts.ts
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::utils::http::get_user_agent;

use super::fetch_telemetry::{
    PluginFetchOutcome, PluginFetchSource, classify_fetch_error, log_plugin_fetch,
};
use super::plugin_directories::get_plugins_directory;

const INSTALL_COUNTS_CACHE_VERSION: u32 = 1;
const INSTALL_COUNTS_CACHE_FILENAME: &str = "install-counts-cache.json";
const INSTALL_COUNTS_URL: &str = "https://raw.githubusercontent.com/anthropics/claude-plugins-official/refs/heads/stats/stats/plugin-installs.json";
const CACHE_TTL_MS: u64 = 24 * 60 * 60 * 1000; // 24 hours

static LAST_FETCH_TIME: AtomicU64 = AtomicU64::new(0);

/// Structure of the install counts cache file.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct InstallCountsCache {
    version: u32,
    #[serde(rename = "fetchedAt")]
    fetched_at: String,
    counts: Vec<CountEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CountEntry {
    plugin: String,
    unique_installs: u64,
}

/// Expected structure of the GitHub stats response.
#[derive(Deserialize)]
struct GitHubStatsResponse {
    plugins: Vec<CountEntry>,
}

fn get_install_counts_cache_path() -> PathBuf {
    PathBuf::from(get_plugins_directory()).join(INSTALL_COUNTS_CACHE_FILENAME)
}

/// Load the install counts cache from disk.
async fn load_install_counts_cache()
-> Result<Option<InstallCountsCache>, Box<dyn std::error::Error + Send + Sync>> {
    let cache_path = get_install_counts_cache_path();

    let content = match fs::read_to_string(&cache_path).await {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    let cache: InstallCountsCache = serde_json::from_str(&content)?;

    if cache.version != INSTALL_COUNTS_CACHE_VERSION {
        log::debug!(
            "Install counts cache version mismatch (got {}, expected {})",
            cache.version,
            INSTALL_COUNTS_CACHE_VERSION
        );
        return Ok(None);
    }

    // Check if cache is stale
    let fetched_at = SystemTime::UNIX_EPOCH
        + std::time::Duration::from_millis(
            chrono::DateTime::parse_from_rfc3339(&cache.fetched_at)
                .ok()
                .map(|dt| dt.timestamp_millis() as u64)
                .unwrap_or(0),
        );

    let now = SystemTime::now();
    if now
        .duration_since(fetched_at)
        .map_or(true, |d| d.as_millis() as u64 > CACHE_TTL_MS)
    {
        log::debug!("Install counts cache is stale (>24h old)");
        return Ok(None);
    }

    Ok(Some(cache))
}

/// Save the install counts cache to disk atomically.
async fn save_install_counts_cache(
    cache: &InstallCountsCache,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cache_path = get_install_counts_cache_path();
    let temp_path = PathBuf::from(format!(
        "{}.{}.tmp",
        cache_path.display(),
        rand::random::<u64>()
    ));

    let plugins_dir = get_plugins_directory();
    tokio::fs::create_dir_all(&plugins_dir).await?;

    let content = serde_json::to_string_pretty(cache)?;
    tokio::fs::write(&temp_path, content).await?;

    // Atomic rename
    tokio::fs::rename(&temp_path, &cache_path).await?;
    log::debug!("Install counts cache saved successfully");
    Ok(())
}

/// Fetch install counts from GitHub stats repository.
async fn fetch_install_counts_from_github()
-> Result<Vec<CountEntry>, Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("Fetching install counts from {}", INSTALL_COUNTS_URL);

    let started = std::time::Instant::now();

    let client = reqwest::Client::builder()
        .user_agent(get_user_agent())
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(INSTALL_COUNTS_URL).send().await?;
    let stats: GitHubStatsResponse = response.json().await?;

    if stats.plugins.is_empty() {
        return Err(format!("Invalid response format from install counts API").into());
    }

    log_plugin_fetch(
        PluginFetchSource::InstallCounts,
        Some(INSTALL_COUNTS_URL),
        PluginFetchOutcome::Success,
        started.elapsed().as_millis() as u64,
        None,
    );

    Ok(stats.plugins)
}

/// Get plugin install counts as a HashMap.
pub async fn get_install_counts()
-> Result<Option<HashMap<String, u64>>, Box<dyn std::error::Error + Send + Sync>> {
    // Try to load from cache first
    if let Some(cache) = load_install_counts_cache().await? {
        log::debug!("Using cached install counts");
        log_plugin_fetch(
            PluginFetchSource::InstallCounts,
            Some(INSTALL_COUNTS_URL),
            PluginFetchOutcome::CacheHit,
            0,
            None,
        );
        let mut map = HashMap::new();
        for entry in cache.counts {
            map.insert(entry.plugin, entry.unique_installs);
        }
        return Ok(Some(map));
    }

    // Cache miss or stale - fetch from GitHub
    match fetch_install_counts_from_github().await {
        Ok(counts) => {
            let now = chrono::Utc::now().to_rfc3339();
            let new_cache = InstallCountsCache {
                version: INSTALL_COUNTS_CACHE_VERSION,
                fetched_at: now,
                counts: counts.clone(),
            };
            save_install_counts_cache(&new_cache).await?;

            let mut map = HashMap::new();
            for entry in counts {
                map.insert(entry.plugin, entry.unique_installs);
            }
            Ok(Some(map))
        }
        Err(e) => {
            log::error!("Failed to fetch install counts: {}", e);
            Ok(None)
        }
    }
}

/// Format an install count for display.
pub fn format_install_count(count: u64) -> String {
    if count < 1000 {
        return count.to_string();
    }

    if count < 1_000_000 {
        let k = count as f64 / 1000.0;
        let formatted = format!("{:.1}", k);
        if formatted.ends_with(".0") {
            format!("{}K", &formatted[..formatted.len() - 2])
        } else {
            format!("{}K", formatted)
        }
    } else {
        let m = count as f64 / 1_000_000.0;
        let formatted = format!("{:.1}", m);
        if formatted.ends_with(".0") {
            format!("{}M", &formatted[..formatted.len() - 2])
        } else {
            format!("{}M", formatted)
        }
    }
}
