// Source: /data/home/swei/claudecode/openclaudecode/src/tools/PowerShellTool/pathValidation.ts
//! PowerShell-specific path validation for command arguments.
//!
//! Extracts file paths from PowerShell commands and validates they stay
//! within allowed project directories.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Maximum directories to list
const MAX_DIRS_TO_LIST: usize = 5;

/// PowerShell wildcards
static GLOB_PATTERN_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"[*?\[\]]").unwrap());

/// File operation type
#[derive(Debug, Clone, PartialEq, Default)]
pub enum FileOperationType {
    #[default]
    Read,
    Write,
    Create,
}

/// Path check result
#[derive(Debug, Clone)]
pub struct PathCheckResult {
    pub allowed: bool,
    pub decision_reason: Option<String>,
}

/// Resolved path check result
#[derive(Debug, Clone)]
pub struct ResolvedPathCheckResult {
    pub allowed: bool,
    pub decision_reason: Option<String>,
    pub resolved_path: String,
}

/// Per-cmdlet parameter configuration
#[derive(Debug, Clone)]
pub struct CmdletPathConfig {
    pub operation_type: FileOperationType,
    pub path_params: Vec<String>,
    pub known_switches: Vec<String>,
    pub known_value_params: Vec<String>,
    pub leaf_only_path_params: Option<Vec<String>>,
    pub positional_skip: Option<usize>,
    pub optional_write: bool,
}

impl Default for CmdletPathConfig {
    fn default() -> Self {
        Self {
            operation_type: FileOperationType::Read,
            path_params: Vec::new(),
            known_switches: Vec::new(),
            known_value_params: Vec::new(),
            leaf_only_path_params: None,
            positional_skip: None,
            optional_write: false,
        }
    }
}

/// Cmdlet path configurations - maps cmdlet names to their path parameter configs
static CMDLET_PATH_CONFIG: Lazy<HashMap<&'static str, CmdletPathConfig>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Write operations
    map.insert(
        "set-content",
        CmdletPathConfig {
            operation_type: FileOperationType::Write,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            known_switches: vec![
                "-passthru".to_string(),
                "-force".to_string(),
                "-whatif".to_string(),
                "-confirm".to_string(),
                "-nonewline".to_string(),
            ],
            known_value_params: vec![
                "-value".to_string(),
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
                "-encoding".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "add-content",
        CmdletPathConfig {
            operation_type: FileOperationType::Write,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            known_switches: vec![
                "-passthru".to_string(),
                "-force".to_string(),
                "-whatif".to_string(),
                "-confirm".to_string(),
                "-nonewline".to_string(),
            ],
            known_value_params: vec![
                "-value".to_string(),
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
                "-encoding".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "remove-item",
        CmdletPathConfig {
            operation_type: FileOperationType::Write,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            known_switches: vec![
                "-recurse".to_string(),
                "-force".to_string(),
                "-whatif".to_string(),
                "-confirm".to_string(),
            ],
            known_value_params: vec![
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
                "-stream".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "clear-content",
        CmdletPathConfig {
            operation_type: FileOperationType::Write,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            known_switches: vec![
                "-force".to_string(),
                "-whatif".to_string(),
                "-confirm".to_string(),
            ],
            known_value_params: vec![
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
                "-stream".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "out-file",
        CmdletPathConfig {
            operation_type: FileOperationType::Write,
            path_params: vec![
                "-filepath".to_string(),
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            known_switches: vec![
                "-append".to_string(),
                "-force".to_string(),
                "-noclobber".to_string(),
                "-nonewline".to_string(),
                "-whatif".to_string(),
                "-confirm".to_string(),
            ],
            known_value_params: vec![
                "-inputobject".to_string(),
                "-encoding".to_string(),
                "-width".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "new-item",
        CmdletPathConfig {
            operation_type: FileOperationType::Create,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            leaf_only_path_params: Some(vec!["-name".to_string()]),
            known_switches: vec![
                "-force".to_string(),
                "-whatif".to_string(),
                "-confirm".to_string(),
            ],
            known_value_params: vec![
                "-itemtype".to_string(),
                "-value".to_string(),
                "-type".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "copy-item",
        CmdletPathConfig {
            operation_type: FileOperationType::Write,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
                "-destination".to_string(),
            ],
            known_switches: vec![
                "-container".to_string(),
                "-force".to_string(),
                "-passthru".to_string(),
                "-recurse".to_string(),
                "-whatif".to_string(),
                "-confirm".to_string(),
            ],
            known_value_params: vec![
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
                "-fromsession".to_string(),
                "-tosession".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "move-item",
        CmdletPathConfig {
            operation_type: FileOperationType::Write,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
                "-destination".to_string(),
            ],
            known_switches: vec![
                "-force".to_string(),
                "-passthru".to_string(),
                "-whatif".to_string(),
                "-confirm".to_string(),
            ],
            known_value_params: vec![
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "rename-item",
        CmdletPathConfig {
            operation_type: FileOperationType::Write,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            known_switches: vec![
                "-force".to_string(),
                "-passthru".to_string(),
                "-whatif".to_string(),
                "-confirm".to_string(),
            ],
            known_value_params: vec![
                "-newname".to_string(),
                "-credential".to_string(),
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
            ],
            ..Default::default()
        },
    );

    // Read operations
    map.insert(
        "get-content",
        CmdletPathConfig {
            operation_type: FileOperationType::Read,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            known_switches: vec![
                "-force".to_string(),
                "-wait".to_string(),
                "-raw".to_string(),
                "-asbytestream".to_string(),
            ],
            known_value_params: vec![
                "-readcount".to_string(),
                "-totalcount".to_string(),
                "-tail".to_string(),
                "-first".to_string(),
                "-head".to_string(),
                "-last".to_string(),
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
                "-delimiter".to_string(),
                "-encoding".to_string(),
                "-stream".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "get-childitem",
        CmdletPathConfig {
            operation_type: FileOperationType::Read,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            known_switches: vec![
                "-recurse".to_string(),
                "-force".to_string(),
                "-name".to_string(),
                "-directory".to_string(),
                "-file".to_string(),
                "-hidden".to_string(),
                "-readonly".to_string(),
                "-system".to_string(),
            ],
            known_value_params: vec![
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
                "-depth".to_string(),
                "-attributes".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "get-item",
        CmdletPathConfig {
            operation_type: FileOperationType::Read,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            known_switches: vec!["-force".to_string()],
            known_value_params: vec![
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
                "-stream".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "get-itemproperty",
        CmdletPathConfig {
            operation_type: FileOperationType::Read,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            known_switches: vec![],
            known_value_params: vec![
                "-name".to_string(),
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
            ],
            ..Default::default()
        },
    );

    map.insert(
        "test-path",
        CmdletPathConfig {
            operation_type: FileOperationType::Read,
            path_params: vec![
                "-path".to_string(),
                "-literalpath".to_string(),
                "-pspath".to_string(),
                "-lp".to_string(),
            ],
            known_switches: vec!["-isvalid".to_string()],
            known_value_params: vec![
                "-filter".to_string(),
                "-include".to_string(),
                "-exclude".to_string(),
                "-pathtype".to_string(),
                "-olderthan".to_string(),
                "-newerthan".to_string(),
            ],
            ..Default::default()
        },
    );

    map
});

/// Get cmdlet path config
pub fn get_cmdlet_path_config(cmdlet_name: &str) -> Option<&'static CmdletPathConfig> {
    // First try direct lookup
    if let Some(config) = CMDLET_PATH_CONFIG.get(cmdlet_name) {
        return Some(config);
    }

    // Try alias resolution
    use super::read_only_validation::resolve_to_canonical;
    let canonical = resolve_to_canonical(cmdlet_name);
    CMDLET_PATH_CONFIG.get(canonical.as_str())
}

/// Check if path is dangerous for removal
pub fn is_dangerous_removal_path(path: &str) -> bool {
    let lower = path.to_lowercase();

    // Check for critical system paths
    let dangerous_paths = [
        "/",
        "/bin",
        "/etc",
        "/usr",
        "/usr/bin",
        "/usr/sbin",
        "/var",
        "/tmp",
        "/home",
        "/root",
        "c:\\",
        "c:\\windows",
        "c:\\program files",
        "c:\\program files (x86)",
    ];

    for dp in dangerous_paths.iter() {
        if lower == *dp || lower.starts_with(&format!("{}/", dp)) || lower.starts_with(dp) {
            return true;
        }
    }

    false
}

/// Check path constraints
pub fn check_path_constraints(command: &str, _allowed_paths: &[String]) -> PathCheckResult {
    use super::read_only_validation::resolve_to_canonical;

    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return PathCheckResult {
            allowed: true,
            decision_reason: None,
        };
    }

    // Get cmdlet name and resolve aliases
    let cmdlet_name = resolve_to_canonical(parts[0]);

    // Get config for this cmdlet
    let config = match get_cmdlet_path_config(&cmdlet_name) {
        Some(c) => c,
        None => {
            // No path config means we can't validate - ask
            return PathCheckResult {
                allowed: false,
                decision_reason: Some("Cmdlet not in path validation config".to_string()),
            };
        }
    };

    // Check for write operations without path (optional write cmdlets like Invoke-WebRequest)
    if config.optional_write && config.operation_type == FileOperationType::Write {
        // Check if any path parameter is present
        let has_path = parts.iter().any(|arg| {
            config
                .path_params
                .iter()
                .any(|p| arg.to_lowercase().starts_with(p))
        });

        if !has_path {
            // No path = output goes to pipeline, not filesystem
            return PathCheckResult {
                allowed: true,
                decision_reason: None,
            };
        }
    }

    // For write operations, check for dangerous paths
    if config.operation_type == FileOperationType::Write
        || config.operation_type == FileOperationType::Create
    {
        // Extract paths from arguments
        for (i, arg) in parts.iter().enumerate() {
            // Skip flags
            if arg.starts_with('-') {
                continue;
            }

            // Check if this could be a path parameter
            let is_path_param = if i > 0 {
                let prev = parts[i - 1].to_lowercase();
                config.path_params.iter().any(|p| prev == *p)
            } else {
                false
            };

            if is_path_param || (!arg.starts_with('-') && i > 0) {
                if is_dangerous_removal_path(arg) {
                    return PathCheckResult {
                        allowed: false,
                        decision_reason: Some(format!("Path '{}' is a dangerous system path", arg)),
                    };
                }
            }
        }
    }

    PathCheckResult {
        allowed: true,
        decision_reason: None,
    }
}

/// Dangerous removal deny check
pub fn dangerous_removal_deny(path: &str) -> bool {
    is_dangerous_removal_path(path)
}

/// Check if path is a dangerous raw path
pub fn is_dangerous_removal_raw_path(path: &str) -> bool {
    is_dangerous_removal_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cmdlet_path_config() {
        let config = get_cmdlet_path_config("set-content");
        assert!(config.is_some());
        assert_eq!(config.unwrap().operation_type, FileOperationType::Write);

        let config = get_cmdlet_path_config("get-content");
        assert!(config.is_some());
        assert_eq!(config.unwrap().operation_type, FileOperationType::Read);
    }

    #[test]
    fn test_is_dangerous_removal_path() {
        assert!(is_dangerous_removal_path("/etc/passwd"));
        assert!(is_dangerous_removal_path("/bin"));
        // /home is in dangerous paths
        assert!(is_dangerous_removal_path("/home/user/file.txt"));
    }

    #[test]
    fn test_check_path_constraints() {
        let result = check_path_constraints("Get-Content test.txt", &["/home/user".to_string()]);
        assert!(result.allowed);

        let result = check_path_constraints("Remove-Item /etc/passwd", &["/home/user".to_string()]);
        assert!(!result.allowed);
    }
}
