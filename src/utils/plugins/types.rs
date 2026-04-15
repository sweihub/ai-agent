// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
//! Plugin types - ported from ~/claudecode/openclaudecode/src/utils/plugins/schemas.ts
//!
//! This module provides types for plugin marketplaces and sources.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin source - can be a local path (relative) or a remote source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginSource {
    /// Local path relative to marketplace root (starts with "./")
    Relative(String),
    /// NPM package source
    Npm {
        source: String,
        package: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        registry: Option<String>,
    },
    /// Pip package source
    Pip {
        source: String,
        package: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        index_url: Option<String>,
    },
    /// GitHub repository source
    Github {
        source: String,
        repo: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        ref_: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sparse_paths: Option<Vec<String>>,
    },
    /// Git subdirectory source
    GitSubdir {
        source: String,
        repo: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        ref_: Option<String>,
        subdir: String,
    },
    /// Git URL source
    Git {
        source: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        ref_: Option<String>,
    },
    /// Direct URL source
    Url {
        source: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },
    /// Settings source (inline manifest from settings.json)
    Settings { source: String },
}

impl PluginSource {
    /// Check if this is a local plugin source (relative path starting with "./")
    pub fn is_local(&self) -> bool {
        matches!(self, PluginSource::Relative(s) if s.starts_with("./"))
    }
}

/// Plugin marketplace entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginMarketplaceEntry {
    /// Unique identifier matching the plugin name
    pub name: String,
    /// Where to fetch the plugin from
    pub source: PluginSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    // Inherited from PluginManifest partial
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<super::super::super::plugin::types::PluginAuthor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_styles: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsp_servers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<HashMap<String, serde_json::Value>>,
}

impl PluginMarketplaceEntry {
    /// Get the relative path from the source for local plugins.
    /// Returns the path component from a Relative source, or empty string for others.
    pub fn source_as_path(&self) -> &str {
        match &self.source {
            PluginSource::Relative(path) => path,
            _ => "",
        }
    }
}

/// Plugin marketplace owner
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginMarketplaceOwner {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Plugin marketplace metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginMarketplaceMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Plugin marketplace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginMarketplace {
    /// Marketplace name
    pub name: String,
    /// Marketplace owner
    pub owner: PluginMarketplaceOwner,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Collection of available plugins in this marketplace
    pub plugins: Vec<PluginMarketplaceEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_remove_deleted_plugins: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PluginMarketplaceMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_cross_marketplace_dependencies_on: Option<Vec<String>>,
}

/// Known marketplace configuration (for known_marketplaces.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownMarketplace {
    /// Marketplace source configuration
    pub source: PluginSource,
    /// Installation location path
    pub install_location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_update: Option<bool>,
}

/// Known marketplaces configuration file
pub type KnownMarketplacesFile = HashMap<String, KnownMarketplace>;

/// Plugin ID in "plugin@marketplace" format
pub type PluginId = String;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_source_relative() {
        let source = PluginSource::Relative("./my-plugin".to_string());
        assert!(source.is_local());
    }

    #[test]
    fn test_plugin_source_npm() {
        let source = PluginSource::Npm {
            source: "npm".to_string(),
            package: "my-npm-plugin".to_string(),
            version: Some("1.0.0".to_string()),
            registry: None,
        };
        assert!(!source.is_local());
    }

    #[test]
    fn test_marketplace_serialization() {
        let marketplace = PluginMarketplace {
            name: "my-marketplace".to_string(),
            owner: PluginMarketplaceOwner {
                name: "Test Owner".to_string(),
                email: Some("test@example.com".to_string()),
                url: None,
            },
            description: Some("A test marketplace".to_string()),
            plugins: vec![PluginMarketplaceEntry {
                name: "test-plugin".to_string(),
                source: PluginSource::Relative("./test-plugin".to_string()),
                description: Some("A test plugin".to_string()),
                version: Some("1.0.0".to_string()),
                strict: Some(true),
                category: None,
                tags: None,
                author: None,
                homepage: None,
                repository: None,
                keywords: None,
                commands: None,
                agents: None,
                skills: None,
                hooks: None,
                output_styles: None,
                mcp_servers: None,
                lsp_servers: None,
                settings: None,
            }],
            force_remove_deleted_plugins: None,
            metadata: None,
            allow_cross_marketplace_dependencies_on: None,
        };

        let json = serde_json::to_string(&marketplace).unwrap();
        assert!(json.contains("my-marketplace"));
        assert!(json.contains("test-plugin"));
    }

    #[test]
    fn test_known_marketplace_serialization() {
        let known = KnownMarketplacesFile::from_iter([(
            "my-marketplace".to_string(),
            KnownMarketplace {
                source: PluginSource::Url {
                    source: "url".to_string(),
                    url: "https://example.com/marketplace.json".to_string(),
                    headers: None,
                },
                install_location: "/path/to/marketplace".to_string(),
                auto_update: Some(true),
            },
        )]);

        let json = serde_json::to_string(&known).unwrap();
        assert!(json.contains("my-marketplace"));
    }
}
