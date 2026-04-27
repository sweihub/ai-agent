// Source: /data/home/swei/claudecode/openclaudecode/src/tools/PowerShellTool/
//! PowerShell tool module

pub mod command_semantics;
pub mod common_parameters;
pub mod destructive_command_warning;
pub mod git_safety;
pub mod mode_validation;
pub mod path_validation;
pub mod powershell_security;
pub mod powershell_tool;
pub mod prompt;
pub mod read_only_validation;
pub mod tool_name;

pub use command_semantics::interpret_command_result;
pub use common_parameters::common_parameters;
pub use destructive_command_warning::get_destructive_command_warning;
pub use git_safety::{is_dot_git_path_ps, is_git_internal_path_ps};
pub use mode_validation::{
    PermissionBehavior, PermissionModeResult, check_permission_mode, is_symlink_creating_command,
};
pub use path_validation::{
    check_path_constraints, dangerous_removal_deny, get_cmdlet_path_config,
    is_dangerous_removal_path,
};
pub use powershell_security::{
    PowerShellSecurityResult, SecurityBehavior, powershell_command_is_safe,
};
pub use powershell_tool::PowerShellTool;
pub use prompt::{get_default_timeout_ms, get_max_timeout_ms};
pub use read_only_validation::{
    has_sync_security_concerns, is_cwd_changing_cmdlet, is_external_command_safe,
    is_read_only_command, resolve_to_canonical,
};
pub use tool_name::POWERSHELL_TOOL_NAME;
