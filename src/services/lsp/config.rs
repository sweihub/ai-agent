// Source: /data/home/swei/claudecode/openclaudecode/src/services/lsp/config.ts
//! LSP configuration module
//! Loads LSP server configurations from plugins

use serde::{Deserialize, Serialize};

/// Scoped LSP server configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopedLspServerConfig {
    pub server_name: String,
    pub scope: String,
    pub config: serde_json::Value,
}

/// Result of loading all LSP servers
#[derive(Debug, Clone, Default)]
pub struct LspServersResult {
    pub servers: std::collections::HashMap<String, ScopedLspServerConfig>,
}

/// Log for debugging
fn log_for_debugging(message: &str) {
    log::debug!("[LSP] {}", message);
}

/// Get all configured LSP servers from plugins.
/// LSP servers are only supported via plugins, not user/project settings.
///
/// Returns a HashMap containing servers configuration keyed by scoped server name
pub async fn get_all_lsp_servers() -> LspServersResult {
    let mut all_servers = std::collections::HashMap::new();

    // TODO: Implement plugin loading and LSP server extraction
    // This requires integration with the plugin system

    log_for_debugging(&format!(
        "Total LSP servers loaded: {}",
        all_servers.len()
    ));

    LspServersResult { servers: all_servers }
}

/// Legacy function name for backward compatibility
pub async fn getAllLspServers() -> Result<LspServersResult, String> {
    Ok(get_all_lsp_servers().await)
}
