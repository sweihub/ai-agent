// Source: ~/claudecode/openclaudecode/src/utils/plugins/walkPluginMarkdown.ts
#![allow(dead_code)]

use std::path::Path;

/// Options for walking a plugin directory.
#[derive(Debug, Clone, Default)]
pub struct WalkPluginMarkdownOpts {
    pub stop_at_skill_dir: Option<bool>,
    pub log_label: Option<String>,
}

/// Error type for walk_plugin_markdown operations.
#[derive(Debug)]
pub struct WalkPluginMarkdownError {
    pub message: String,
    pub path: String,
}

impl std::fmt::Display for WalkPluginMarkdownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "walk_plugin_markdown error at {}: {}",
            self.path, self.message
        )
    }
}

impl std::error::Error for WalkPluginMarkdownError {}

/// Recursively walk a plugin directory, invoking on_file for each .md file.
/// Stub implementation - full implementation would recursively scan directories.
pub async fn walk_plugin_markdown<F, Fut>(
    _root_dir: &Path,
    _on_file: F,
    _opts: WalkPluginMarkdownOpts,
) -> std::io::Result<()>
where
    F: Fn(String, Vec<String>) -> Fut + Send + Sync + Clone,
    Fut: std::future::Future<Output = ()> + Send,
{
    // Stub: simplified implementation
    Ok(())
}
