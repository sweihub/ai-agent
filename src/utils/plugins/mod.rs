//! Plugin utilities - ported from ~/claudecode/openclaudecode/src/utils/plugins/
//!
//! This module provides plugin marketplace types, loading, installation, and management.

pub mod add_dir_plugin_settings;
pub mod cache_utils;
pub mod dependency_resolver;
pub mod fetch_telemetry;
pub mod git_availability;
pub mod headless_plugin_install;
pub mod hint_recommendation;
pub mod install_counts;
pub mod installed_plugins_manager;
pub mod load_plugin_agents;
pub mod load_plugin_commands;
pub mod load_plugin_hooks;
pub mod load_plugin_output_styles;
pub mod loader;
pub mod lsp_plugin_integration;
pub mod lsp_recommendation;
pub mod managed_plugins;
pub mod marketplace_helpers;
pub mod marketplace_manager;
pub mod mcp_plugin_integration;
pub mod mcpb_handler;
pub mod official_marketplace;
pub mod official_marketplace_gcs;
pub mod official_marketplace_startup_check;
pub mod orphaned_plugin_filter;
pub mod parse_marketplace_input;
pub mod plugin_autoupdate;
pub mod plugin_blocklist;
pub mod plugin_directories;
pub mod plugin_flagging;
pub mod plugin_identifier;
pub mod plugin_installation_helpers;
pub mod plugin_options_storage;
pub mod plugin_policy;
pub mod plugin_startup_check;
pub mod plugin_versioning;
pub mod reconciler;
pub mod refresh;
pub mod schemas;
pub mod types;
pub mod validate_plugin;
pub mod walk_plugin_markdown;
pub mod zip_cache;
pub mod zip_cache_adapters;

pub use loader::{
    cache_plugin, clear_plugin_cache, get_known_marketplace_names, get_marketplace_cache_only,
    get_plugin_by_id_cache_only, get_plugin_cache_path, get_versioned_cache_path,
    get_versioned_zip_cache_path, load_all_plugins, load_all_plugins_cache_only,
    parse_plugin_identifier,
};
pub use types::{
    KnownMarketplace, KnownMarketplacesFile, PluginId, PluginMarketplace, PluginMarketplaceEntry,
    PluginMarketplaceMetadata, PluginMarketplaceOwner, PluginSource,
};
