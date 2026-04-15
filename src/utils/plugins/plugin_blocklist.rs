// Source: ~/claudecode/openclaudecode/src/utils/plugins/pluginBlocklist.ts
#![allow(dead_code)]

use super::installed_plugins_manager::load_installed_plugins_v2;
use super::marketplace_manager::{get_marketplace, load_known_marketplaces_config_safe};
use super::plugin_flagging::{add_flagged_plugin, get_flagged_plugins, load_flagged_plugins};

/// Detect plugins installed from a marketplace that are no longer listed there.
pub fn detect_delisted_plugins(
    installed_plugins: &super::installed_plugins_manager::InstalledPluginsFileV2,
    marketplace: &super::types::PluginMarketplace,
    marketplace_name: &str,
) -> Vec<String> {
    let marketplace_plugin_names: std::collections::HashSet<_> = marketplace
        .plugins
        .iter()
        .map(|p| p.name.as_str())
        .collect();

    let suffix = format!("@{}", marketplace_name);
    let mut delisted = Vec::new();

    for plugin_id in installed_plugins.plugins.keys() {
        if !plugin_id.ends_with(&suffix) {
            continue;
        }

        let plugin_name = &plugin_id[..plugin_id.len() - suffix.len()];
        if !marketplace_plugin_names.contains(plugin_name) {
            delisted.push(plugin_id.clone());
        }
    }

    delisted
}

/// Detect delisted plugins across all marketplaces, auto-uninstall them,
/// and record them as flagged.
pub async fn detect_and_uninstall_delisted_plugins() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    load_flagged_plugins().await?;

    let installed_plugins = load_installed_plugins_v2()?;
    let already_flagged = get_flagged_plugins();
    let known_marketplaces = load_known_marketplaces_config_safe().await?;
    let mut newly_flagged = Vec::new();

    for marketplace_name in known_marketplaces.keys() {
        let marketplace = match get_marketplace(marketplace_name).await {
            Ok(m) => m,
            Err(e) => {
                log::debug!(
                    "Failed to check for delisted plugins in \"{}\": {}",
                    marketplace_name,
                    e
                );
                continue;
            }
        };

        if !marketplace.force_remove_deleted_plugins.unwrap_or(false) {
            continue;
        }

        let delisted = detect_delisted_plugins(
            &installed_plugins,
            &marketplace,
            marketplace_name,
        );

        for plugin_id in delisted {
            if already_flagged.contains_key(&plugin_id) {
                continue;
            }

            let installations = installed_plugins
                .plugins
                .get(&plugin_id)
                .cloned()
                .unwrap_or_default();

            let has_user_install = installations.iter().any(|i| {
                matches!(
                    i.scope,
                    super::schemas::PluginScope::User
                        | super::schemas::PluginScope::Project
                        | super::schemas::PluginScope::Local
                )
            });

            if !has_user_install {
                continue;
            }

            add_flagged_plugin(&plugin_id).await?;
            newly_flagged.push(plugin_id);
        }
    }

    Ok(newly_flagged)
}
