// Source: ~/claudecode/openclaudecode/src/types/plugin.ts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Re-export of plugin author from schemas.
pub type PluginAuthor = serde_json::Value;

/// Re-export of plugin manifest from schemas.
pub type PluginManifest = serde_json::Value;

/// Re-export of command metadata from schemas.
pub type CommandMetadata = serde_json::Value;

/// Definition for a built-in plugin that ships with the CLI.
pub struct BuiltinPluginDefinition {
    /// Plugin name (used in `{name}@builtin` identifier)
    pub name: String,
    /// Description shown in the /plugin UI
    pub description: String,
    /// Optional version string
    pub version: Option<String>,
    /// Skills provided by this plugin
    pub skills: Option<Vec<serde_json::Value>>,
    /// Hooks provided by this plugin
    pub hooks: Option<serde_json::Value>,
    /// MCP servers provided by this plugin
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
    /// Whether this plugin is available
    pub is_available: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    /// Default enabled state before the user sets a preference
    pub default_enabled: Option<bool>,
}

/// A plugin repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRepository {
    pub url: String,
    pub branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "commitSha")]
    pub commit_sha: Option<String>,
}

/// Plugin configuration with repositories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub repositories: HashMap<String, PluginRepository>,
}

/// A loaded plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedPlugin {
    pub name: String,
    pub manifest: PluginManifest,
    pub path: String,
    pub source: String,
    /// Repository identifier
    pub repository: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// True for built-in plugins that ship with the CLI
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isBuiltin")]
    pub is_builtin: Option<bool>,
    /// Git commit SHA for version pinning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "commandsPath")]
    pub commands_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "commandsPaths")]
    pub commands_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "commandsMetadata")]
    pub commands_metadata: Option<HashMap<String, CommandMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "agentsPath")]
    pub agents_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "agentsPaths")]
    pub agents_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "skillsPath")]
    pub skills_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "skillsPaths")]
    pub skills_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "outputStylesPath")]
    pub output_styles_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "outputStylesPaths")]
    pub output_styles_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hooksConfig")]
    pub hooks_config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mcpServers")]
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lspServers")]
    pub lsp_servers: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<HashMap<String, serde_json::Value>>,
}

/// Plugin component types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginComponent {
    Commands,
    Agents,
    Skills,
    Hooks,
    OutputStyles,
}

/// Discriminated union of plugin error types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PluginError {
    #[serde(rename = "path-not-found")]
    PathNotFound {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        path: String,
        component: PluginComponent,
    },
    #[serde(rename = "git-auth-failed")]
    GitAuthFailed {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        #[serde(rename = "gitUrl")]
        git_url: String,
        #[serde(rename = "authType")]
        auth_type: GitAuthType,
    },
    #[serde(rename = "git-timeout")]
    GitTimeout {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        #[serde(rename = "gitUrl")]
        git_url: String,
        operation: GitOperation,
    },
    #[serde(rename = "network-error")]
    NetworkError {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },
    #[serde(rename = "manifest-parse-error")]
    ManifestParseError {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        #[serde(rename = "manifestPath")]
        manifest_path: String,
        #[serde(rename = "parseError")]
        parse_error: String,
    },
    #[serde(rename = "manifest-validation-error")]
    ManifestValidationError {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        #[serde(rename = "manifestPath")]
        manifest_path: String,
        #[serde(rename = "validationErrors")]
        validation_errors: Vec<String>,
    },
    #[serde(rename = "plugin-not-found")]
    PluginNotFound {
        source: String,
        #[serde(rename = "pluginId")]
        plugin_id: String,
        marketplace: String,
    },
    #[serde(rename = "marketplace-not-found")]
    MarketplaceNotFound {
        source: String,
        marketplace: String,
        #[serde(rename = "availableMarketplaces")]
        available_marketplaces: Vec<String>,
    },
    #[serde(rename = "marketplace-load-failed")]
    MarketplaceLoadFailed {
        source: String,
        marketplace: String,
        reason: String,
    },
    #[serde(rename = "mcp-config-invalid")]
    McpConfigInvalid {
        source: String,
        plugin: String,
        #[serde(rename = "serverName")]
        server_name: String,
        #[serde(rename = "validationError")]
        validation_error: String,
    },
    #[serde(rename = "mcp-server-suppressed-duplicate")]
    McpServerSuppressedDuplicate {
        source: String,
        plugin: String,
        #[serde(rename = "serverName")]
        server_name: String,
        #[serde(rename = "duplicateOf")]
        duplicate_of: String,
    },
    #[serde(rename = "lsp-config-invalid")]
    LspConfigInvalid {
        source: String,
        plugin: String,
        #[serde(rename = "serverName")]
        server_name: String,
        #[serde(rename = "validationError")]
        validation_error: String,
    },
    #[serde(rename = "hook-load-failed")]
    HookLoadFailed {
        source: String,
        plugin: String,
        #[serde(rename = "hookPath")]
        hook_path: String,
        reason: String,
    },
    #[serde(rename = "component-load-failed")]
    ComponentLoadFailed {
        source: String,
        plugin: String,
        component: PluginComponent,
        path: String,
        reason: String,
    },
    #[serde(rename = "mcpb-download-failed")]
    McpbDownloadFailed {
        source: String,
        plugin: String,
        url: String,
        reason: String,
    },
    #[serde(rename = "mcpb-extract-failed")]
    McpbExtractFailed {
        source: String,
        plugin: String,
        #[serde(rename = "mcpbPath")]
        mcpb_path: String,
        reason: String,
    },
    #[serde(rename = "mcpb-invalid-manifest")]
    McpbInvalidManifest {
        source: String,
        plugin: String,
        #[serde(rename = "mcpbPath")]
        mcpb_path: String,
        #[serde(rename = "validationError")]
        validation_error: String,
    },
    #[serde(rename = "lsp-server-start-failed")]
    LspServerStartFailed {
        source: String,
        plugin: String,
        #[serde(rename = "serverName")]
        server_name: String,
        reason: String,
    },
    #[serde(rename = "lsp-server-crashed")]
    LspServerCrashed {
        source: String,
        plugin: String,
        #[serde(rename = "serverName")]
        server_name: String,
        #[serde(rename = "exitCode")]
        exit_code: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signal: Option<String>,
    },
    #[serde(rename = "lsp-request-timeout")]
    LspRequestTimeout {
        source: String,
        plugin: String,
        #[serde(rename = "serverName")]
        server_name: String,
        method: String,
        #[serde(rename = "timeoutMs")]
        timeout_ms: u64,
    },
    #[serde(rename = "lsp-request-failed")]
    LspRequestFailed {
        source: String,
        plugin: String,
        #[serde(rename = "serverName")]
        server_name: String,
        method: String,
        error: String,
    },
    #[serde(rename = "marketplace-blocked-by-policy")]
    MarketplaceBlockedByPolicy {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        marketplace: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "blockedByBlocklist")]
        blocked_by_blocklist: Option<bool>,
        #[serde(rename = "allowedSources")]
        allowed_sources: Vec<String>,
    },
    #[serde(rename = "dependency-unsatisfied")]
    DependencyUnsatisfied {
        source: String,
        plugin: String,
        dependency: String,
        reason: DependencyReason,
    },
    #[serde(rename = "plugin-cache-miss")]
    PluginCacheMiss {
        source: String,
        plugin: String,
        #[serde(rename = "installPath")]
        install_path: String,
    },
    #[serde(rename = "generic-error")]
    GenericError {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
        error: String,
    },
}

/// Git authentication type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitAuthType {
    Ssh,
    Https,
}

/// Git operation type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitOperation {
    Clone,
    Pull,
}

/// Dependency reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyReason {
    NotEnabled,
    NotFound,
}

/// Plugin load result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLoadResult {
    pub enabled: Vec<LoadedPlugin>,
    pub disabled: Vec<LoadedPlugin>,
    pub errors: Vec<PluginError>,
}

/// Helper function to get a display message from any PluginError.
pub fn get_plugin_error_message(error: &PluginError) -> String {
    match error {
        PluginError::GenericError { error: msg, .. } => msg.clone(),
        PluginError::PathNotFound {
            path, component, ..
        } => {
            format!("Path not found: {} ({:?})", path, component)
        }
        PluginError::GitAuthFailed {
            auth_type, git_url, ..
        } => {
            format!("Git authentication failed ({:?}): {}", auth_type, git_url)
        }
        PluginError::GitTimeout {
            operation, git_url, ..
        } => {
            format!("Git {:?} timeout: {}", operation, git_url)
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
                    duplicate_of.split(':').nth(1).unwrap_or("?")
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
            format!("{:?} load failed from {}: {}", component, path, reason)
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
            if let Some(sig) = signal {
                format!(
                    "Plugin \"{}\" LSP server \"{}\" crashed with signal {}",
                    plugin, server_name, sig
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
            if blocked_by_blocklist == &Some(true) {
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
            let hint = match reason {
                DependencyReason::NotEnabled => "disabled — enable it or remove the dependency",
                DependencyReason::NotFound => "not found in any configured marketplace",
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
