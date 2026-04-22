// Source: ~/claudecode/openclaudecode/src/utils/plugins/pluginInstallationHelpers.ts
#![allow(dead_code)]

use super::schemas::{PluginMarketplaceEntry, PluginScope};

/// Get current ISO timestamp.
pub fn get_current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Validate that a resolved path stays within a base directory.
pub fn _validate_path_within_base(
    base_path: &str,
    relative_path: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let base = std::fs::canonicalize(base_path)?;
    let resolved = std::fs::canonicalize(std::path::Path::new(base_path).join(relative_path))?;

    let normalized_base = base.to_string_lossy();
    let normalized_base_with_sep = format!("{}{}", normalized_base, std::path::MAIN_SEPARATOR);

    if !resolved.starts_with(&normalized_base_with_sep) && resolved != base {
        return Err(format!(
            "Path traversal detected: \"{}\" would escape the base directory",
            relative_path
        )
        .into());
    }

    Ok(resolved.to_string_lossy().to_string())
}

/// Cache a plugin and add it to installed_plugins.json.
pub async fn cache_and_register_plugin(
    _plugin_id: &str,
    _entry: &PluginMarketplaceEntry,
    _scope: PluginScope,
    _project_path: Option<&str>,
    _local_source_path: Option<&str>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Stub: simplified implementation
    Err("cache_and_register_plugin not fully implemented".into())
}

/// Register a plugin installation without caching.
pub fn _register_plugin_installation(
    _plugin_id: &str,
    _install_path: &str,
    _version: Option<&str>,
    _scope: PluginScope,
    _project_path: Option<&str>,
) {
    // Stub
}

/// Format a failed ResolutionResult into a user-facing message.
pub fn _format_resolution_error(_r: &super::dependency_resolver::ResolutionResult) -> String {
    "Unknown resolution error".to_string()
}
