// Source: ~/claudecode/openclaudecode/src/utils/plugins/addDirPluginSettings.ts

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Extra known marketplace configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ExtraKnownMarketplace {
    pub source: String,
    pub repo: String,
}

/// Settings JSON structure.
#[derive(Debug, Deserialize)]
struct ParsedSettings {
    #[serde(default)]
    enabled_plugins: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    extra_known_marketplaces: Option<HashMap<String, ExtraKnownMarketplace>>,
}

const SETTINGS_FILES: &[&str] = &["settings.json", "settings.local.json"];

/// Returns a merged record of enabled_plugins from all --add-dir directories.
///
/// Within each directory, settings.local.json is processed after settings.json
/// (local wins within that dir). Across directories, later CLI-order wins on
/// conflict.
///
/// This has the lowest priority -- callers must spread their standard settings
/// on top to let user/project/local/flag/policy override.
pub fn get_add_dir_enabled_plugins() -> HashMap<String, serde_json::Value> {
    let mut result: HashMap<String, serde_json::Value> = HashMap::new();

    for dir in get_additional_directories_for_claude_md() {
        for file in SETTINGS_FILES {
            let path = dir.join(".ai").join(file);
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(parsed) = serde_json::from_str::<ParsedSettings>(&content) {
                    if let Some(enabled_plugins) = parsed.enabled_plugins {
                        result.extend(enabled_plugins);
                    }
                }
            }
        }
    }

    result
}

/// Returns a merged record of extra_known_marketplaces from all --add-dir directories.
///
/// Same priority rules as get_add_dir_enabled_plugins: settings.local.json wins
/// within each dir, and callers spread standard settings on top.
pub fn get_add_dir_extra_marketplaces() -> HashMap<String, ExtraKnownMarketplace> {
    let mut result: HashMap<String, ExtraKnownMarketplace> = HashMap::new();

    for dir in get_additional_directories_for_claude_md() {
        for file in SETTINGS_FILES {
            let path = dir.join(".ai").join(file);
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(parsed) = serde_json::from_str::<ParsedSettings>(&content) {
                    if let Some(extra_marketplaces) = parsed.extra_known_marketplaces {
                        result.extend(extra_marketplaces);
                    }
                }
            }
        }
    }

    result
}

/// Get additional directories for AI config (localized from claudeMd).
fn get_additional_directories_for_claude_md() -> Vec<PathBuf> {
    // In production, this would read from the bootstrap state.
    // For now, check environment variable.
    if let Ok(dirs_str) = std::env::var("AI_CODE_ADDITIONAL_DIRS") {
        dirs_str
            .split('|')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_when_no_dirs() {
        #[allow(unused_unsafe)]
        unsafe {
            std::env::remove_var("AI_CODE_ADDITIONAL_DIRS");
        }
        assert!(get_add_dir_enabled_plugins().is_empty());
        assert!(get_add_dir_extra_marketplaces().is_empty());
    }
}
