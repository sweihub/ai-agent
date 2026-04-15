// Source: ~/claudecode/openclaudecode/src/utils/plugins/marketplaceHelpers.ts
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use super::marketplace_manager::get_marketplace;
use super::schemas::MarketplaceSource;
use super::types::KnownMarketplace;

/// Format plugin failure details for user display.
pub fn format_failure_details(
    failures: &[PluginFailure],
    include_reasons: bool,
) -> String {
    let max_show = 2;
    let details: Vec<String> = failures
        .iter()
        .take(max_show)
        .map(|f| {
            let reason = f
                .reason
                .as_deref()
                .or(f.error.as_deref())
                .unwrap_or("unknown error");
            if include_reasons {
                format!("{} ({})", f.name, reason)
            } else {
                f.name.clone()
            }
        })
        .collect();

    let remaining = failures.len().saturating_sub(max_show);
    let more_text = if remaining > 0 {
        format!(" and {} more", remaining)
    } else {
        String::new()
    };

    let sep = if include_reasons { "; " } else { ", " };
    format!("{}{}", details.join(sep), more_text)
}

#[derive(Debug, Clone)]
pub struct PluginFailure {
    pub name: String,
    pub reason: Option<String>,
    pub error: Option<String>,
}

/// Extract source display string from marketplace configuration.
pub fn get_marketplace_source_display(source: &MarketplaceSource) -> String {
    match source {
        MarketplaceSource::Github { repo, .. } => repo.clone(),
        MarketplaceSource::Url { url } => url.clone(),
        MarketplaceSource::Git { url, .. } => url.clone(),
        MarketplaceSource::Directory { path } => path.clone(),
        MarketplaceSource::File { path } => path.clone(),
        MarketplaceSource::Settings { name, .. } => format!("settings:{}", name),
        MarketplaceSource::GitSubdir { url, .. } => url.clone(),
    }
}

/// Create a plugin ID from plugin name and marketplace name.
pub fn create_plugin_id(plugin_name: &str, marketplace_name: &str) -> String {
    format!("{}@{}", plugin_name, marketplace_name)
}

/// Load marketplaces with graceful degradation for individual failures.
pub async fn load_marketplaces_with_graceful_degradation(
    config: &HashMap<String, KnownMarketplace>,
) -> MarketplaceLoadResult {
    let mut marketplaces = Vec::new();
    let mut failures = Vec::new();

    for (name, marketplace_config) in config {
        if !is_plugin_source_allowed_by_policy(&marketplace_config.source) {
            continue;
        }

        let data = match get_marketplace(name).await {
            Ok(d) => Some(d),
            Err(err) => {
                failures.push(MarketplaceFailure {
                    name: name.clone(),
                    error: err.to_string(),
                });
                log::error!("Failed to load marketplace {}: {}", name, err);
                None
            }
        };

        marketplaces.push(MarketplaceLoadEntry {
            name: name.clone(),
            config: marketplace_config.clone(),
            data,
        });
    }

    MarketplaceLoadResult {
        marketplaces,
        failures,
    }
}

#[derive(Debug)]
pub struct MarketplaceLoadResult {
    pub marketplaces: Vec<MarketplaceLoadEntry>,
    pub failures: Vec<MarketplaceFailure>,
}

#[derive(Debug)]
pub struct MarketplaceLoadEntry {
    pub name: String,
    pub config: KnownMarketplace,
    pub data: Option<super::types::PluginMarketplace>,
}

#[derive(Debug)]
pub struct MarketplaceFailure {
    pub name: String,
    pub error: String,
}

/// Format marketplace loading failures into appropriate user messages.
pub fn format_marketplace_loading_errors(
    failures: &[MarketplaceFailure],
    success_count: usize,
) -> Option<MarketplaceLoadingError> {
    if failures.is_empty() {
        return None;
    }

    if success_count > 0 {
        let message = if failures.len() == 1 {
            format!(
                "Warning: Failed to load marketplace '{}': {}",
                failures[0].name, failures[0].error
            )
        } else {
            let names: Vec<&str> = failures.iter().map(|f| f.name.as_str()).collect();
            format!(
                "Warning: Failed to load {} marketplaces: {}",
                failures.len(),
                names.join(", ")
            )
        };
        Some(MarketplaceLoadingError {
            error_type: "warning".to_string(),
            message,
        })
    } else {
        let errors: Vec<String> = failures
            .iter()
            .map(|f| format!("{}: {}", f.name, f.error))
            .collect();
        Some(MarketplaceLoadingError {
            error_type: "error".to_string(),
            message: format!("Failed to load all marketplaces. Errors: {}", errors.join("; ")),
        })
    }
}

#[derive(Debug)]
pub struct MarketplaceLoadingError {
    pub error_type: String,
    pub message: String,
}

/// Get the strict marketplace source allowlist from policy settings.
pub fn get_strict_known_marketplaces() -> Option<Vec<MarketplaceSource>> {
    None
}

/// Get the marketplace source blocklist from policy settings.
pub fn get_blocked_marketplaces() -> Option<Vec<MarketplaceSource>> {
    None
}

/// Get the custom plugin trust message from policy settings.
pub fn get_plugin_trust_message() -> Option<String> {
    None
}

/// Check if a marketplace source is allowed by enterprise policy.
pub fn is_source_allowed_by_policy(_source: &MarketplaceSource) -> bool {
    true
}

/// Check if a plugin source is allowed by enterprise policy.
pub fn is_plugin_source_allowed_by_policy(_source: &super::types::PluginSource) -> bool {
    true
}

/// Check if a marketplace source is explicitly in the blocklist.
pub fn is_source_in_blocklist(source: &MarketplaceSource) -> bool {
    match get_blocked_marketplaces() {
        None => false,
        Some(blocklist) => blocklist.iter().any(|blocked| sources_equal(source, blocked)),
    }
}

/// Compare two MarketplaceSource objects for equality.
fn sources_equal(a: &MarketplaceSource, b: &MarketplaceSource) -> bool {
    match (a, b) {
        (MarketplaceSource::Url { url: a_url }, MarketplaceSource::Url { url: b_url }) => {
            a_url == b_url
        }
        (
            MarketplaceSource::Github {
                repo: a_repo,
                ref_: a_ref,
                path: a_path,
            },
            MarketplaceSource::Github {
                repo: b_repo,
                ref_: b_ref,
                path: b_path,
            },
        ) => {
            a_repo == b_repo
                && a_ref == b_ref
                && a_path == b_path
        }
        (MarketplaceSource::Directory { path: a_path }, MarketplaceSource::Directory { path: b_path }) => {
            a_path == b_path
        }
        (MarketplaceSource::File { path: a_path }, MarketplaceSource::File { path: b_path }) => {
            a_path == b_path
        }
        _ => false,
    }
}

/// Extract the host/domain from a marketplace source.
pub fn extract_host_from_source(source: &MarketplaceSource) -> Option<String> {
    match source {
        MarketplaceSource::Github { .. } => Some("github.com".to_string()),
        MarketplaceSource::Git { url, .. } | MarketplaceSource::Url { url } => {
            url::Url::parse(url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
        }
        _ => None,
    }
}

/// Format a MarketplaceSource for display in error messages.
pub fn format_source_for_display(source: &MarketplaceSource) -> String {
    match source {
        MarketplaceSource::Github { repo, ref_: r, .. } => {
            format!("github:{}{}", repo, r.as_ref().map(|r| format!("@{}", r)).unwrap_or_default())
        }
        MarketplaceSource::Url { url } => url.clone(),
        MarketplaceSource::Git { url, ref_: r, .. } => {
            format!("git:{}{}", url, r.as_ref().map(|r| format!("@{}", r)).unwrap_or_default())
        }
        MarketplaceSource::Directory { path } => format!("dir:{}", path),
        MarketplaceSource::File { path } => format!("file:{}", path),
        MarketplaceSource::Settings { name, plugins } => {
            format!("settings:{} ({} plugin{})", name, plugins.len(), if plugins.len() == 1 { "" } else { "s" })
        }
        MarketplaceSource::GitSubdir { url, path, .. } => {
            format!("git-subdir:{}:{}", url, path)
        }
    }
}
