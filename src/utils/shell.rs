//! Shell utilities and shell provider abstractions.

pub mod bash_provider;
pub mod powershell_provider;
pub mod shell_provider;
pub mod shell_tool_utils;

pub use bash_provider::BashShellProvider;
pub use powershell_provider::{build_powershell_args, PowerShellProvider};
pub use shell_provider::{ShellError, ShellExecCommand};
pub use shell_tool_utils::{ShellType, SHELL_TYPES};
