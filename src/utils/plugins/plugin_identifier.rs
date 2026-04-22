// Source: ~/claudecode/openclaudecode/src/utils/plugins/pluginIdentifier.ts
#![allow(dead_code)]

use super::schemas::PluginScope;

/// Parsed plugin identifier with name and optional marketplace.
pub struct ParsedPluginIdentifier {
    pub name: String,
    pub marketplace: Option<String>,
}

/// Parse a plugin identifier string into name and marketplace components.
pub fn parse_plugin_identifier(plugin: &str) -> ParsedPluginIdentifier {
    if let Some(at_pos) = plugin.find('@') {
        let name = plugin[..at_pos].to_string();
        let marketplace = plugin[at_pos + 1..].to_string();
        ParsedPluginIdentifier {
            name,
            marketplace: Some(marketplace),
        }
    } else {
        ParsedPluginIdentifier {
            name: plugin.to_string(),
            marketplace: None,
        }
    }
}

/// Build a plugin ID from name and marketplace.
pub fn build_plugin_id(name: &str, marketplace: Option<&str>) -> String {
    match marketplace {
        Some(m) => format!("{}@{}", name, m),
        None => name.to_string(),
    }
}

/// Check if a marketplace name is an official marketplace.
pub fn is_official_marketplace_name(marketplace: Option<&str>) -> bool {
    match marketplace {
        Some(m) => {
            let lowercase = m.to_lowercase();
            super::schemas::allowed_official_marketplace_names()
                .iter()
                .any(|&s| s == lowercase)
        }
        None => false,
    }
}

/// Map from installable plugin scope to editable setting source.
pub fn scope_to_setting_source(
    scope: &PluginScope,
) -> Result<&'static str, Box<dyn std::error::Error + Send + Sync>> {
    match scope {
        PluginScope::User => Ok("userSettings"),
        PluginScope::Project => Ok("projectSettings"),
        PluginScope::Local => Ok("localSettings"),
        PluginScope::Managed => Err("Cannot install plugins to managed scope".into()),
    }
}

/// Convert an editable setting source to its corresponding plugin scope.
pub fn setting_source_to_scope(source: &str) -> Option<PluginScope> {
    match source {
        "userSettings" => Some(PluginScope::User),
        "projectSettings" => Some(PluginScope::Project),
        "localSettings" => Some(PluginScope::Local),
        "policySettings" => Some(PluginScope::Managed),
        _ => None,
    }
}
