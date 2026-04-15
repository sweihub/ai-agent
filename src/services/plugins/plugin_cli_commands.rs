// Source: ~/claudecode/openclaudecode/src/services/plugins/pluginCliCommands.ts
#![allow(dead_code)]

//! CLI command wrappers for plugin operations
//!
//! This module provides thin wrappers around the core plugin operations
//! that handle CLI-specific concerns like console output and process exit.
//!
//! For the core operations (without CLI side effects), see plugin_operations.rs

use std::collections::HashMap;
use std::process;

use super::plugin_operations::{
    disable_all_plugins_op, disable_plugin_op, enable_plugin_op, install_plugin_op,
    uninstall_plugin_op, update_plugin_op, InstallableScope, PluginOperationResult,
    PluginUpdateResult,
};
use crate::utils::plugins::loader::parse_plugin_identifier;

pub use super::plugin_operations::{VALID_INSTALLABLE_SCOPES, VALID_UPDATE_SCOPES};

type PluginCliCommand =
    &'static str; // "install" | "uninstall" | "enable" | "disable" | "disable-all" | "update"

/// Telemetry fields for plugin operations
#[derive(Debug, Clone, Default)]
struct PluginTelemetryFields {
    plugin_name: Option<String>,
    marketplace_name: Option<String>,
    is_managed: bool,
}

/// Analytics event metadata
#[derive(Debug, Clone, Default)]
struct AnalyticsEvent {
    event_name: String,
    properties: HashMap<String, serde_json::Value>,
}

/// Classifies plugin command errors into categories for telemetry
fn classify_plugin_command_error(error: &dyn std::error::Error) -> String {
    let msg = error.to_string().to_lowercase();
    if msg.contains("not found") {
        "not_found".to_string()
    } else if msg.contains("permission") || msg.contains("blocked") || msg.contains("policy") {
        "permission_denied".to_string()
    } else if msg.contains("network") || msg.contains("timeout") || msg.contains("connection") {
        "network_error".to_string()
    } else if msg.contains("parse") || msg.contains("invalid") || msg.contains("format") {
        "parse_error".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Build plugin telemetry fields for analytics
fn build_plugin_telemetry_fields(
    name: Option<&str>,
    marketplace: Option<&str>,
    managed_plugin_names: &[String],
) -> PluginTelemetryFields {
    PluginTelemetryFields {
        plugin_name: name.map(String::from),
        marketplace_name: marketplace.map(String::from),
        is_managed: name
            .map(|n| managed_plugin_names.iter().any(|m| m == n))
            .unwrap_or(false),
    }
}

/// Get managed plugin names (stub - would read from managed plugins config)
fn get_managed_plugin_names() -> Vec<String> {
    Vec::new()
}

/// Log an analytics event
fn log_event(event: AnalyticsEvent) {
    log::debug!(
        "Analytics event: {} {:?}",
        event.event_name,
        event.properties
    );
}

/// Unicode figures for console output
mod figures {
    pub const TICK: &str = "\u{2713}"; // ✓
    pub const CROSS: &str = "\u{2717}"; // ✗
}

/// Generic error handler for plugin CLI commands. Emits
/// tengu_plugin_command_failed before exit so dashboards can compute a
/// success rate against the corresponding success events.
fn handle_plugin_command_error(
    error: &dyn std::error::Error,
    command: PluginCliCommand,
    plugin: Option<&str>,
) -> ! {
    log::error!("Plugin command error: {}", error);

    let operation = match plugin {
        Some(p) => format!("{} plugin \"{}\"", command, p),
        None if command == "disable-all" => "disable all plugins".to_string(),
        None => format!("{} plugins", command),
    };

    eprintln!("{} Failed to {}: {}", figures::CROSS, operation, error);

    let telemetry_fields = plugin.map(|p| {
        let (name, marketplace) = parse_plugin_identifier(p);
        let telemetry = build_plugin_telemetry_fields(
            name.as_deref(),
            marketplace.as_deref(),
            &get_managed_plugin_names(),
        );
        (name, marketplace, telemetry)
    });

    let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
    properties.insert(
        "command".to_string(),
        serde_json::Value::String(command.to_string()),
    );
    properties.insert(
        "error_category".to_string(),
        serde_json::Value::String(classify_plugin_command_error(error)),
    );

    if let Some((name, marketplace, telemetry)) = telemetry_fields {
        if let Some(n) = name {
            properties.insert("_PROTO_plugin_name".to_string(), serde_json::Value::String(n));
        }
        if let Some(m) = marketplace {
            properties.insert(
                "_PROTO_marketplace_name".to_string(),
                serde_json::Value::String(m),
            );
        }
        properties.insert(
            "is_managed".to_string(),
            serde_json::Value::Bool(telemetry.is_managed),
        );
    }

    log_event(AnalyticsEvent {
        event_name: "tengu_plugin_command_failed".to_string(),
        properties,
    });

    process::exit(1);
}

/// CLI command: Install a plugin non-interactively
///
/// # Arguments
/// * `plugin` - Plugin identifier (name or plugin@marketplace)
/// * `scope` - Installation scope: user, project, or local (defaults to 'user')
pub async fn install_plugin(
    plugin: &str,
    scope: InstallableScope,
) -> Result<PluginOperationResult, Box<dyn std::error::Error>> {
    println!("Installing plugin \"{}\"...", plugin);

    let result = install_plugin_op(plugin, scope).await;

    if !result.success {
        return Err(result.message.clone().into());
    }

    println!("{} {}", figures::TICK, result.message);

    // Log analytics
    let (name, marketplace) = parse_plugin_identifier(
        result
            .plugin_id
            .as_deref()
            .unwrap_or(plugin),
    );
    let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
    if let Some(n) = &name {
        properties.insert("_PROTO_plugin_name".to_string(), serde_json::Value::String(n.clone()));
    }
    if let Some(m) = &marketplace {
        properties.insert(
            "_PROTO_marketplace_name".to_string(),
            serde_json::Value::String(m.clone()),
        );
    }
    properties.insert(
        "scope".to_string(),
        serde_json::Value::String(result.scope.clone().unwrap_or_else(|| scope.to_string())),
    );
    properties.insert(
        "install_source".to_string(),
        serde_json::Value::String("cli-explicit".to_string()),
    );

    let telemetry = build_plugin_telemetry_fields(
        name.as_deref(),
        marketplace.as_deref(),
        &get_managed_plugin_names(),
    );
    properties.insert(
        "is_managed".to_string(),
        serde_json::Value::Bool(telemetry.is_managed),
    );

    log_event(AnalyticsEvent {
        event_name: "tengu_plugin_installed_cli".to_string(),
        properties,
    });

    Ok(result)
}

/// CLI command: Uninstall a plugin non-interactively
///
/// # Arguments
/// * `plugin` - Plugin name or plugin@marketplace identifier
/// * `scope` - Uninstall from scope: user, project, or local (defaults to 'user')
/// * `keep_data` - Whether to keep the plugin's data directory
pub async fn uninstall_plugin(
    plugin: &str,
    scope: InstallableScope,
    keep_data: bool,
) -> Result<PluginOperationResult, Box<dyn std::error::Error>> {
    let result = uninstall_plugin_op(plugin, scope, !keep_data).await;

    if !result.success {
        return Err(result.message.clone().into());
    }

    println!("{} {}", figures::TICK, result.message);

    // Log analytics
    let (name, marketplace) = parse_plugin_identifier(
        result
            .plugin_id
            .as_deref()
            .unwrap_or(plugin),
    );
    let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
    if let Some(n) = &name {
        properties.insert("_PROTO_plugin_name".to_string(), serde_json::Value::String(n.clone()));
    }
    if let Some(m) = &marketplace {
        properties.insert(
            "_PROTO_marketplace_name".to_string(),
            serde_json::Value::String(m.clone()),
        );
    }
    properties.insert(
        "scope".to_string(),
        serde_json::Value::String(result.scope.clone().unwrap_or_else(|| scope.to_string())),
    );

    let telemetry = build_plugin_telemetry_fields(
        name.as_deref(),
        marketplace.as_deref(),
        &get_managed_plugin_names(),
    );
    properties.insert(
        "is_managed".to_string(),
        serde_json::Value::Bool(telemetry.is_managed),
    );

    log_event(AnalyticsEvent {
        event_name: "tengu_plugin_uninstalled_cli".to_string(),
        properties,
    });

    Ok(result)
}

/// CLI command: Enable a plugin non-interactively
///
/// # Arguments
/// * `plugin` - Plugin name or plugin@marketplace identifier
/// * `scope` - Optional scope. If not provided, finds the most specific scope for the current project.
pub async fn enable_plugin(
    plugin: &str,
    scope: Option<InstallableScope>,
) -> Result<PluginOperationResult, Box<dyn std::error::Error>> {
    let result = enable_plugin_op(plugin, scope).await;

    if !result.success {
        return Err(result.message.clone().into());
    }

    println!("{} {}", figures::TICK, result.message);

    // Log analytics
    let (name, marketplace) = parse_plugin_identifier(
        result
            .plugin_id
            .as_deref()
            .unwrap_or(plugin),
    );
    let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
    if let Some(n) = &name {
        properties.insert("_PROTO_plugin_name".to_string(), serde_json::Value::String(n.clone()));
    }
    if let Some(m) = &marketplace {
        properties.insert(
            "_PROTO_marketplace_name".to_string(),
            serde_json::Value::String(m.clone()),
        );
    }
    if let Some(ref s) = result.scope {
        properties.insert("scope".to_string(), serde_json::Value::String(s.clone()));
    }

    let telemetry = build_plugin_telemetry_fields(
        name.as_deref(),
        marketplace.as_deref(),
        &get_managed_plugin_names(),
    );
    properties.insert(
        "is_managed".to_string(),
        serde_json::Value::Bool(telemetry.is_managed),
    );

    log_event(AnalyticsEvent {
        event_name: "tengu_plugin_enabled_cli".to_string(),
        properties,
    });

    Ok(result)
}

/// CLI command: Disable a plugin non-interactively
///
/// # Arguments
/// * `plugin` - Plugin name or plugin@marketplace identifier
/// * `scope` - Optional scope. If not provided, finds the most specific scope for the current project.
pub async fn disable_plugin(
    plugin: &str,
    scope: Option<InstallableScope>,
) -> Result<PluginOperationResult, Box<dyn std::error::Error>> {
    let result = disable_plugin_op(plugin, scope).await;

    if !result.success {
        return Err(result.message.clone().into());
    }

    println!("{} {}", figures::TICK, result.message);

    // Log analytics
    let (name, marketplace) = parse_plugin_identifier(
        result
            .plugin_id
            .as_deref()
            .unwrap_or(plugin),
    );
    let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
    if let Some(n) = &name {
        properties.insert("_PROTO_plugin_name".to_string(), serde_json::Value::String(n.clone()));
    }
    if let Some(m) = &marketplace {
        properties.insert(
            "_PROTO_marketplace_name".to_string(),
            serde_json::Value::String(m.clone()),
        );
    }
    if let Some(ref s) = result.scope {
        properties.insert("scope".to_string(), serde_json::Value::String(s.clone()));
    }

    let telemetry = build_plugin_telemetry_fields(
        name.as_deref(),
        marketplace.as_deref(),
        &get_managed_plugin_names(),
    );
    properties.insert(
        "is_managed".to_string(),
        serde_json::Value::Bool(telemetry.is_managed),
    );

    log_event(AnalyticsEvent {
        event_name: "tengu_plugin_disabled_cli".to_string(),
        properties,
    });

    Ok(result)
}

/// CLI command: Disable all enabled plugins non-interactively
pub async fn disable_all_plugins() -> Result<PluginOperationResult, Box<dyn std::error::Error>> {
    let result = disable_all_plugins_op().await;

    if !result.success {
        return Err(result.message.clone().into());
    }

    println!("{} {}", figures::TICK, result.message);

    log_event(AnalyticsEvent {
        event_name: "tengu_plugin_disabled_all_cli".to_string(),
        properties: HashMap::new(),
    });

    Ok(result)
}

/// CLI command: Update a plugin non-interactively
///
/// # Arguments
/// * `plugin` - Plugin name or plugin@marketplace identifier
/// * `scope` - Scope to update
pub async fn update_plugin_cli(
    plugin: &str,
    scope: &str,
) -> Result<PluginUpdateResult, Box<dyn std::error::Error>> {
    println!(
        "Checking for updates for plugin \"{}\" at {} scope...",
        plugin, scope
    );

    let result = update_plugin_op(plugin, scope).await;

    if !result.success {
        return Err(result.message.clone().into());
    }

    println!("{} {}", figures::TICK, result.message);

    if !result.already_up_to_date.unwrap_or(false) {
        let (name, marketplace) = parse_plugin_identifier(
            result
                .plugin_id
                .as_deref()
                .unwrap_or(plugin),
        );
        let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
        if let Some(n) = &name {
            properties.insert(
                "_PROTO_plugin_name".to_string(),
                serde_json::Value::String(n.clone()),
            );
        }
        if let Some(m) = &marketplace {
            properties.insert(
                "_PROTO_marketplace_name".to_string(),
                serde_json::Value::String(m.clone()),
            );
        }
        properties.insert(
            "old_version".to_string(),
            serde_json::Value::String(
                result
                    .old_version
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            ),
        );
        properties.insert(
            "new_version".to_string(),
            serde_json::Value::String(
                result
                    .new_version
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            ),
        );

        let telemetry = build_plugin_telemetry_fields(
            name.as_deref(),
            marketplace.as_deref(),
            &get_managed_plugin_names(),
        );
        properties.insert(
            "is_managed".to_string(),
            serde_json::Value::Bool(telemetry.is_managed),
        );

        log_event(AnalyticsEvent {
            event_name: "tengu_plugin_updated_cli".to_string(),
            properties,
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_plugin_command_error_not_found() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "plugin not found");
        assert_eq!(classify_plugin_command_error(&err), "not_found");
    }

    #[test]
    fn test_classify_plugin_command_error_permission() {
        let err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        assert_eq!(
            classify_plugin_command_error(&err),
            "permission_denied"
        );
    }

    #[test]
    fn test_classify_plugin_command_error_network() {
        let err = std::io::Error::new(std::io::ErrorKind::TimedOut, "network timeout");
        assert_eq!(classify_plugin_command_error(&err), "network_error");
    }

    #[test]
    fn test_classify_plugin_command_error_unknown() {
        let err = std::io::Error::new(std::io::ErrorKind::Other, "some other error");
        assert_eq!(classify_plugin_command_error(&err), "unknown");
    }

    #[test]
    fn test_build_plugin_telemetry_fields() {
        let fields = build_plugin_telemetry_fields(
            Some("test-plugin"),
            Some("test-marketplace"),
            &["managed-plugin".to_string()],
        );
        assert_eq!(fields.plugin_name, Some("test-plugin".to_string()));
        assert_eq!(
            fields.marketplace_name,
            Some("test-marketplace".to_string())
        );
        assert!(!fields.is_managed);
    }

    #[test]
    fn test_build_plugin_telemetry_fields_managed() {
        let fields = build_plugin_telemetry_fields(
            Some("managed-plugin"),
            None,
            &["managed-plugin".to_string()],
        );
        assert!(fields.is_managed);
    }

    #[test]
    fn test_figures_constants() {
        assert_eq!(figures::TICK, "\u{2713}");
        assert_eq!(figures::CROSS, "\u{2717}");
    }
}
