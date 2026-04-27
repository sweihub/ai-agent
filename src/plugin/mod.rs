//! Plugin module - ported from ~/claudecode/openclaudecode/src/types/plugin.ts
//!
//! This module provides the plugin types and infrastructure for the Rust SDK.

pub mod builtin_plugins;
pub mod commands;
pub mod loader;
pub mod mcp;
pub mod skills;
pub mod types;

// Explicit re-exports to avoid ambiguous glob re-exports
pub use builtin_plugins::{
    BUILTIN_MARKETPLACE_NAME_CONST, clear_builtin_plugins, get_builtin_plugin_definition,
    get_builtin_plugin_skill_definitions, get_builtin_plugins, is_builtin_plugin_id,
    register_builtin_plugin, BuiltinPluginResult, BuiltinPluginSummary,
};
pub use commands::{
    CommandFrontmatter, CommandHandler, CommandRegistry, ExecutablePluginCommand, PluginCommand,
    substitute_arguments,
};
pub use loader::*;
pub use mcp::*;
pub use skills::*;
pub use types::{
    CommandAvailability, CommandMetadata, CommandResult, CommandResultDisplay, CommandSource,
    LoadedPlugin, PluginAuthor, PluginComponent, PluginConfig, PluginError, PluginLoadResult,
    PluginManifest, PluginRepository, get_plugin_error_message,
};
