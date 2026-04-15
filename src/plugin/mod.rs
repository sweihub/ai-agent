//! Plugin module - ported from ~/claudecode/openclaudecode/src/types/plugin.ts
//!
//! This module provides the plugin types and infrastructure for the Rust SDK.

pub mod commands;
pub mod loader;
pub mod mcp;
pub mod skills;
pub mod types;

// Explicit re-exports to avoid ambiguous glob re-exports
pub use commands::{
    substitute_arguments, CommandFrontmatter, CommandHandler, CommandRegistry,
    ExecutablePluginCommand, PluginCommand,
};
pub use loader::*;
pub use mcp::*;
pub use skills::*;
pub use types::{
    get_plugin_error_message, CommandAvailability, CommandMetadata, CommandResult,
    CommandResultDisplay, CommandSource, LoadedPlugin, PluginAuthor, PluginComponent, PluginConfig,
    PluginError, PluginLoadResult, PluginManifest, PluginRepository,
};
