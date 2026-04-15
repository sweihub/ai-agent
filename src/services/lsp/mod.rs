//! LSP (Language Server Protocol) service module

pub mod lsp_client;
pub mod types;

// Config module stub
mod config;

// Additional stubs
mod lsp_diagnostic_registry;
mod lsp_server_instance;
mod lsp_server_manager;
mod manager;
mod passive_feedback;

pub use lsp_client::*;
pub use types::*;
