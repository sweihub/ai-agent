//! Shell provider types and definitions.

use thiserror::Error;

/// Result of building a shell exec command
#[derive(Debug, Clone)]
pub struct ShellExecCommand {
    /// The full command string to execute
    pub command_string: String,
    /// Path to file containing the current working directory
    pub cwd_file_path: String,
}

/// Shell-related errors
#[derive(Debug, Error)]
pub enum ShellError {
    #[error("Failed to build command: {0}")]
    BuildError(String),

    #[error("Shell not found: {0}")]
    ShellNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
