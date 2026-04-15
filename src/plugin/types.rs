// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
//! Plugin types - ported from ~/claudecode/openclaudecode/src/types/plugin.ts
//!
//! This module provides the core plugin types for the Rust SDK.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin author information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginAuthor {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Command metadata when using object-mapping format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
}

/// Command availability - determines which auth/provider environments can use the command
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandAvailability {
    /// claude.ai OAuth subscriber (Pro/Max/Team/Enterprise via claude.ai)
    ClaudeAi,
    /// Console API key user (direct api.anthropic.com)
    Console,
}

/// How to display command result
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandResultDisplay {
    /// Skip displaying result
    Skip,
    /// Display as system message
    System,
    /// Display as user message (default)
    User,
}

/// Command result types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandResult {
    /// Text result
    Text { value: String },
    /// Skip messages
    Skip,
    /// Compact result
    Compact {
        #[serde(skip_serializing_if = "Option::is_none")]
        display_text: Option<String>,
    },
}

/// Command source - where the command was loaded from
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandSource {
    /// Builtin command
    Builtin,
    /// Loaded from skills directory
    Skills,
    /// Loaded from plugin
    Plugin,
    /// Managed/remote command
    Managed,
    /// Bundled command
    Bundled,
    /// MCP command
    Mcp,
}

/// Plugin manifest (plugin.json structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    /// Unique identifier for the plugin (kebab-case recommended)
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<PluginAuthor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
    // Additional component paths
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
    pub channels: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsp_servers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_config: Option<HashMap<String, serde_json::Value>>,
}

/// Plugin repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRepository {
    pub url: String,
    pub branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
}

/// Plugin configuration (repositories)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfig {
    pub repositories: HashMap<String, PluginRepository>,
}

/// Component types that a plugin can provide
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginComponent {
    Commands,
    Agents,
    Skills,
    Hooks,
    OutputStyles,
}

impl std::fmt::Display for PluginComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginComponent::Commands => write!(f, "commands"),
            PluginComponent::Agents => write!(f, "agents"),
            PluginComponent::Skills => write!(f, "skills"),
            PluginComponent::Hooks => write!(f, "hooks"),
            PluginComponent::OutputStyles => write!(f, "output-styles"),
        }
    }
}

/// Loaded plugin with all its metadata and paths
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedPlugin {
    pub name: String,
    pub manifest: PluginManifest,
    pub path: String,
    pub source: String,
    /// Repository identifier, usually same as source
    pub repository: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// true for built-in plugins that ship with the CLI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_builtin: Option<bool>,
    /// Git commit SHA for version pinning (from marketplace entry source)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands_paths: Option<Vec<String>>,
    /// Metadata for named commands from object-mapping format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands_metadata: Option<HashMap<String, CommandMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_styles_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_styles_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks_config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsp_servers: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<HashMap<String, serde_json::Value>>,
}

/// Discriminated union of plugin error types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum PluginError {
    /// Path not found error
    PathNotFound {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        path: String,
        component: PluginComponent,
    },
    /// Git authentication failed
    GitAuthFailed {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        git_url: String,
        auth_type: String,
    },
    /// Git operation timeout
    GitTimeout {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        git_url: String,
        operation: String,
    },
    /// Network error
    NetworkError {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },
    /// Manifest parse error
    ManifestParseError {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        manifest_path: String,
        parse_error: String,
    },
    /// Manifest validation error
    ManifestValidationError {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        manifest_path: String,
        validation_errors: Vec<String>,
    },
    /// Plugin not found in marketplace
    PluginNotFound {
        source: String,
        plugin_id: String,
        marketplace: String,
    },
    /// Marketplace not found
    MarketplaceNotFound {
        source: String,
        marketplace: String,
        available_marketplaces: Vec<String>,
    },
    /// Marketplace load failed
    MarketplaceLoadFailed {
        source: String,
        marketplace: String,
        reason: String,
    },
    /// MCP config invalid
    McpConfigInvalid {
        source: String,
        plugin: String,
        server_name: String,
        validation_error: String,
    },
    /// MCP server suppressed duplicate
    McpServerSuppressedDuplicate {
        source: String,
        plugin: String,
        server_name: String,
        duplicate_of: String,
    },
    /// LSP config invalid
    LspConfigInvalid {
        source: String,
        plugin: String,
        server_name: String,
        validation_error: String,
    },
    /// Hook load failed
    HookLoadFailed {
        source: String,
        plugin: String,
        hook_path: String,
        reason: String,
    },
    /// Component load failed
    ComponentLoadFailed {
        source: String,
        plugin: String,
        component: PluginComponent,
        path: String,
        reason: String,
    },
    /// MCPB download failed
    McpbDownloadFailed {
        source: String,
        plugin: String,
        url: String,
        reason: String,
    },
    /// MCPB extract failed
    McpbExtractFailed {
        source: String,
        plugin: String,
        mcpb_path: String,
        reason: String,
    },
    /// MCPB invalid manifest
    McpbInvalidManifest {
        source: String,
        plugin: String,
        mcpb_path: String,
        validation_error: String,
    },
    /// LSP server start failed
    LspServerStartFailed {
        source: String,
        plugin: String,
        server_name: String,
        reason: String,
    },
    /// LSP server crashed
    LspServerCrashed {
        source: String,
        plugin: String,
        server_name: String,
        exit_code: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signal: Option<String>,
    },
    /// LSP request timeout
    LspRequestTimeout {
        source: String,
        plugin: String,
        server_name: String,
        method: String,
        timeout_ms: u64,
    },
    /// LSP request failed
    LspRequestFailed {
        source: String,
        plugin: String,
        server_name: String,
        method: String,
        error: String,
    },
    /// Marketplace blocked by policy
    MarketplaceBlockedByPolicy {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        marketplace: String,
        /// true if blocked by blockedMarketplaces, false if not in strictKnownMarketplaces
        #[serde(skip_serializing_if = "Option::is_none")]
        blocked_by_blocklist: Option<bool>,
        /// Formatted source strings (e.g., "github:owner/repo")
        allowed_sources: Vec<String>,
    },
    /// Dependency unsatisfied
    DependencyUnsatisfied {
        source: String,
        plugin: String,
        dependency: String,
        reason: String,
    },
    /// Plugin cache miss
    PluginCacheMiss {
        source: String,
        plugin: String,
        install_path: String,
    },
    /// Generic error
    GenericError {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        error: String,
    },
}

/// Result of loading plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginLoadResult {
    pub enabled: Vec<LoadedPlugin>,
    pub disabled: Vec<LoadedPlugin>,
    pub errors: Vec<PluginError>,
}

/// Get a display message from any PluginError
pub fn get_plugin_error_message(error: &PluginError) -> String {
    match error {
        PluginError::GenericError { error, .. } => error.clone(),
        PluginError::PathNotFound {
            path, component, ..
        } => {
            format!("Path not found: {} ({})", path, component)
        }
        PluginError::GitAuthFailed {
            git_url, auth_type, ..
        } => {
            format!("Git authentication failed ({}): {}", auth_type, git_url)
        }
        PluginError::GitTimeout {
            git_url, operation, ..
        } => {
            format!("Git {} timeout: {}", operation, git_url)
        }
        PluginError::NetworkError { url, details, .. } => {
            if let Some(d) = details {
                format!("Network error: {} - {}", url, d)
            } else {
                format!("Network error: {}", url)
            }
        }
        PluginError::ManifestParseError { parse_error, .. } => {
            format!("Manifest parse error: {}", parse_error)
        }
        PluginError::ManifestValidationError {
            validation_errors, ..
        } => {
            format!(
                "Manifest validation failed: {}",
                validation_errors.join(", ")
            )
        }
        PluginError::PluginNotFound {
            plugin_id,
            marketplace,
            ..
        } => {
            format!(
                "Plugin {} not found in marketplace {}",
                plugin_id, marketplace
            )
        }
        PluginError::MarketplaceNotFound { marketplace, .. } => {
            format!("Marketplace {} not found", marketplace)
        }
        PluginError::MarketplaceLoadFailed {
            marketplace,
            reason,
            ..
        } => {
            format!("Marketplace {} failed to load: {}", marketplace, reason)
        }
        PluginError::McpConfigInvalid {
            server_name,
            validation_error,
            ..
        } => {
            format!("MCP server {} invalid: {}", server_name, validation_error)
        }
        PluginError::McpServerSuppressedDuplicate {
            server_name,
            duplicate_of,
            ..
        } => {
            let dup = if duplicate_of.starts_with("plugin:") {
                format!(
                    "server provided by plugin \"{}\"",
                    duplicate_of.strip_prefix("plugin:").unwrap_or("?")
                )
            } else {
                format!("already-configured \"{}\"", duplicate_of)
            };
            format!(
                "MCP server \"{}\" skipped — same command/URL as {}",
                server_name, dup
            )
        }
        PluginError::HookLoadFailed { reason, .. } => {
            format!("Hook load failed: {}", reason)
        }
        PluginError::ComponentLoadFailed {
            component,
            path,
            reason,
            ..
        } => {
            format!("{} load failed from {}: {}", component, path, reason)
        }
        PluginError::McpbDownloadFailed { url, reason, .. } => {
            format!("Failed to download MCPB from {}: {}", url, reason)
        }
        PluginError::McpbExtractFailed {
            mcpb_path, reason, ..
        } => {
            format!("Failed to extract MCPB {}: {}", mcpb_path, reason)
        }
        PluginError::McpbInvalidManifest {
            mcpb_path,
            validation_error,
            ..
        } => {
            format!(
                "MCPB manifest invalid at {}: {}",
                mcpb_path, validation_error
            )
        }
        PluginError::LspConfigInvalid {
            plugin,
            server_name,
            validation_error,
            ..
        } => {
            format!(
                "Plugin \"{}\" has invalid LSP server config for \"{}\": {}",
                plugin, server_name, validation_error
            )
        }
        PluginError::LspServerStartFailed {
            plugin,
            server_name,
            reason,
            ..
        } => {
            format!(
                "Plugin \"{}\" failed to start LSP server \"{}\": {}",
                plugin, server_name, reason
            )
        }
        PluginError::LspServerCrashed {
            plugin,
            server_name,
            exit_code,
            signal,
            ..
        } => {
            if let Some(s) = signal {
                format!(
                    "Plugin \"{}\" LSP server \"{}\" crashed with signal {}",
                    plugin, server_name, s
                )
            } else {
                format!(
                    "Plugin \"{}\" LSP server \"{}\" crashed with exit code {}",
                    plugin,
                    server_name,
                    exit_code
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                )
            }
        }
        PluginError::LspRequestTimeout {
            plugin,
            server_name,
            method,
            timeout_ms,
            ..
        } => {
            format!(
                "Plugin \"{}\" LSP server \"{}\" timed out on {} request after {}ms",
                plugin, server_name, method, timeout_ms
            )
        }
        PluginError::LspRequestFailed {
            plugin,
            server_name,
            method,
            error,
            ..
        } => {
            format!(
                "Plugin \"{}\" LSP server \"{}\" {} request failed: {}",
                plugin, server_name, method, error
            )
        }
        PluginError::MarketplaceBlockedByPolicy {
            marketplace,
            blocked_by_blocklist,
            ..
        } => {
            if blocked_by_blocklist.unwrap_or(false) {
                format!(
                    "Marketplace '{}' is blocked by enterprise policy",
                    marketplace
                )
            } else {
                format!(
                    "Marketplace '{}' is not in the allowed marketplace list",
                    marketplace
                )
            }
        }
        PluginError::DependencyUnsatisfied {
            dependency, reason, ..
        } => {
            let hint = if reason == "not-enabled" {
                "disabled — enable it or remove the dependency"
            } else {
                "not found in any configured marketplace"
            };
            format!("Dependency \"{}\" is {}", dependency, hint)
        }
        PluginError::PluginCacheMiss {
            plugin,
            install_path,
            ..
        } => {
            format!(
                "Plugin \"{}\" not cached at {} — run /plugins to refresh",
                plugin, install_path
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_author_serialization() {
        let author = PluginAuthor {
            name: "Test Author".to_string(),
            email: Some("test@example.com".to_string()),
            url: Some("https://example.com".to_string()),
        };
        let json = serde_json::to_string(&author).unwrap();
        assert!(json.contains("test@example.com"));
    }

    #[test]
    fn test_plugin_manifest_serialization() {
        let manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("A test plugin".to_string()),
            author: None,
            homepage: None,
            repository: None,
            license: None,
            keywords: None,
            dependencies: None,
            commands: None,
            agents: None,
            skills: None,
            hooks: None,
            output_styles: None,
            channels: None,
            mcp_servers: None,
            lsp_servers: None,
            settings: None,
            user_config: None,
        };
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("test-plugin"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_plugin_component_display() {
        assert_eq!(PluginComponent::Commands.to_string(), "commands");
        assert_eq!(PluginComponent::Agents.to_string(), "agents");
        assert_eq!(PluginComponent::Skills.to_string(), "skills");
        assert_eq!(PluginComponent::Hooks.to_string(), "hooks");
        assert_eq!(PluginComponent::OutputStyles.to_string(), "output-styles");
    }

    #[test]
    fn test_plugin_error_generic() {
        let error = PluginError::GenericError {
            source: "test".to_string(),
            plugin: Some("my-plugin".to_string()),
            error: "Something went wrong".to_string(),
        };
        let message = get_plugin_error_message(&error);
        assert_eq!(message, "Something went wrong");
    }

    #[test]
    fn test_plugin_error_path_not_found() {
        let error = PluginError::PathNotFound {
            source: "test".to_string(),
            plugin: Some("my-plugin".to_string()),
            path: "./commands/test.md".to_string(),
            component: PluginComponent::Commands,
        };
        let message = get_plugin_error_message(&error);
        assert!(message.contains("Path not found"));
        assert!(message.contains("commands"));
    }

    #[test]
    fn test_plugin_error_network() {
        let error = PluginError::NetworkError {
            source: "test".to_string(),
            plugin: None,
            url: "https://example.com".to_string(),
            details: Some("Connection refused".to_string()),
        };
        let message = get_plugin_error_message(&error);
        assert!(message.contains("Network error"));
        assert!(message.contains("Connection refused"));
    }

    #[test]
    fn test_plugin_load_result() {
        let result = PluginLoadResult {
            enabled: vec![],
            disabled: vec![],
            errors: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("enabled"));
        assert!(json.contains("disabled"));
        assert!(json.contains("errors"));
    }
}
