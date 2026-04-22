// Source: ~/claudecode/openclaudecode/src/utils/plugins/schemas.ts
#![allow(dead_code)]

use once_cell::sync::Lazy;
use std::collections::HashSet;

// Re-export types that are defined in types.rs to avoid duplicate definitions
pub use super::types::{
    KnownMarketplace, KnownMarketplacesFile, PluginMarketplaceEntry, PluginSource,
};

/// Official marketplace names that are reserved for Anthropic/Claude official use.
pub fn allowed_official_marketplace_names() -> &'static HashSet<&'static str> {
    static NAMES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
        HashSet::from([
            "claude-code-marketplace",
            "claude-code-plugins",
            "claude-plugins-official",
            "anthropic-marketplace",
            "anthropic-plugins",
            "agent-skills",
            "life-sciences",
            "knowledge-work-plugins",
        ])
    });
    &NAMES
}

/// Official marketplaces that should NOT auto-update by default.
fn no_auto_update_official_marketplaces() -> &'static HashSet<&'static str> {
    static NAMES: Lazy<HashSet<&'static str>> =
        Lazy::new(|| HashSet::from(["knowledge-work-plugins"]));
    &NAMES
}

/// Check if auto-update is enabled for a marketplace.
pub fn is_marketplace_auto_update(marketplace_name: &str, _entry: &serde_json::Value) -> bool {
    let normalized = marketplace_name.to_lowercase();
    allowed_official_marketplace_names().contains(normalized.as_str())
        && !no_auto_update_official_marketplaces().contains(normalized.as_str())
}

/// Check if a source is a local plugin source.
pub fn is_local_plugin_source(source: &PluginSource) -> bool {
    matches!(source, PluginSource::Relative(_))
}

/// Marketplace source types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(tag = "source")]
pub enum MarketplaceSource {
    #[serde(rename = "github")]
    Github {
        repo: String,
        #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
        ref_: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    #[serde(rename = "url")]
    Url { url: String },
    #[serde(rename = "git")]
    Git {
        url: String,
        #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
        ref_: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    #[serde(rename = "directory")]
    Directory { path: String },
    #[serde(rename = "file")]
    File { path: String },
    #[serde(rename = "settings")]
    Settings { name: String, plugins: Vec<String> },
    #[serde(rename = "git-subdir")]
    GitSubdir {
        url: String,
        path: String,
        #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
        ref_: Option<String>,
    },
}

/// Check if a marketplace source is a local marketplace source.
pub fn is_local_marketplace_source(source: &MarketplaceSource) -> bool {
    matches!(
        source,
        MarketplaceSource::Directory { .. } | MarketplaceSource::File { .. }
    )
}

/// Plugin manifest (from plugin.json).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<PluginAuthor>,
    pub dependencies: Option<Vec<String>>,
    #[serde(rename = "userConfig")]
    pub user_config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PluginAuthor {
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
}

/// Plugin installation scope.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PluginScope {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "managed")]
    Managed,
    #[serde(rename = "project")]
    Project,
    #[serde(rename = "local")]
    Local,
}

/// Plugin installation entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginInstallationEntry {
    pub scope: PluginScope,
    #[serde(rename = "installPath")]
    pub install_path: String,
    pub version: Option<String>,
    #[serde(rename = "installedAt")]
    pub installed_at: String,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
    #[serde(rename = "gitCommitSha", skip_serializing_if = "Option::is_none")]
    pub git_commit_sha: Option<String>,
    #[serde(rename = "projectPath", skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
}
