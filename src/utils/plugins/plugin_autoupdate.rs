// Source: ~/claudecode/openclaudecode/src/utils/plugins/pluginAutoupdate.ts
#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Mutex;

use once_cell::sync::Lazy;

use super::installed_plugins_manager::{
    get_pending_updates_details, has_pending_updates, is_installation_relevant_to_current_project,
    load_installed_plugins_from_disk,
};
use super::plugin_identifier::parse_plugin_identifier;

/// Callback type for notifying when plugins have been updated.
type PluginAutoUpdateCallback = Box<dyn Fn(Vec<String>) + Send + Sync>;

static PLUGIN_UPDATE_CALLBACK: Lazy<Mutex<Option<PluginAutoUpdateCallback>>> =
    Lazy::new(|| Mutex::new(None));
static PENDING_NOTIFICATION: Lazy<Mutex<Option<Vec<String>>>> = Lazy::new(|| Mutex::new(None));

/// Register a callback to be notified when plugins are auto-updated.
pub fn on_plugins_auto_updated(
    callback: impl Fn(Vec<String>) + Send + Sync + 'static,
) -> Box<dyn FnOnce()> {
    let cb: PluginAutoUpdateCallback = Box::new(callback);

    {
        let mut pending = PENDING_NOTIFICATION.lock().unwrap();
        if let Some(ref updates) = *pending {
            if !updates.is_empty() {
                cb(updates.clone());
                *pending = None;
            }
        }
    }

    {
        let mut callback_lock = PLUGIN_UPDATE_CALLBACK.lock().unwrap();
        *callback_lock = Some(cb);
    }

    Box::new(|| {
        let mut callback_lock = PLUGIN_UPDATE_CALLBACK.lock().unwrap();
        *callback_lock = None;
    })
}

/// Check if pending updates came from autoupdate.
pub fn get_auto_updated_plugin_names() -> Vec<String> {
    if !has_pending_updates() {
        return Vec::new();
    }

    get_pending_updates_details()
        .into_iter()
        .map(|d| parse_plugin_identifier(&d.plugin_id).name)
        .collect()
}

/// Get the set of marketplaces that have autoUpdate enabled.
async fn get_auto_update_enabled_marketplaces() -> HashSet<String> {
    HashSet::new()
}

/// Update all project-relevant installed plugins from the given marketplaces.
pub async fn update_plugins_for_marketplaces(_marketplace_names: &HashSet<String>) -> Vec<String> {
    Vec::new()
}

/// Auto-update marketplaces and plugins in the background.
pub fn auto_update_marketplaces_and_plugins_in_background() {
    // Stub: disabled
}
