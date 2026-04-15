// Source: /data/home/swei/claudecode/openclaudecode/src/tools/PowerShellTool/commonParameters.ts
//! PowerShell Common Parameters
//!
//! PowerShell Common Parameters (available on all cmdlets via [CmdletBinding()]).
//! Source: about_CommonParameters (PowerShell docs) + Get-Command output.
//!
//! Shared between pathValidation.ts (merges into per-cmdlet known-param sets)
//! and readOnlyValidation.ts (merges into safeFlags check). Split out to break
//! what would otherwise be an import cycle between those two files.
//!
//! Stored lowercase with leading dash — callers `.toLowerCase()` their input.

use std::collections::HashSet;

/// Common switch parameters
pub const COMMON_SWITCHES: &[&str] = &["-verbose", "-debug"];

/// Common value parameters
pub const COMMON_VALUE_PARAMS: &[&str] = &[
    "-erroraction",
    "-warningaction",
    "-informationaction",
    "-progressaction",
    "-errorvariable",
    "-warningvariable",
    "-informationvariable",
    "-outvariable",
    "-outbuffer",
    "-pipelinevariable",
];

/// All common parameters
pub fn common_parameters() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    for s in COMMON_SWITCHES {
        set.insert(*s);
    }
    for s in COMMON_VALUE_PARAMS {
        set.insert(*s);
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_switches() {
        assert!(COMMON_SWITCHES.contains(&"-verbose"));
        assert!(COMMON_SWITCHES.contains(&"-debug"));
    }

    #[test]
    fn test_common_value_params() {
        assert!(COMMON_VALUE_PARAMS.contains(&"-erroraction"));
        assert!(COMMON_VALUE_PARAMS.contains(&"-outvariable"));
    }

    #[test]
    fn test_common_parameters() {
        let params = common_parameters();
        assert!(params.contains("-verbose"));
        assert!(params.contains("-debug"));
        assert!(params.contains("-erroraction"));
        assert!(params.contains("-warningaction"));
        assert!(params.contains("-outvariable"));
    }

    #[test]
    fn test_common_parameters_count() {
        let params = common_parameters();
        // 2 switches + 10 value params = 12
        assert_eq!(params.len(), 12);
    }

    #[test]
    fn test_common_parameters_case_sensitive() {
        let params = common_parameters();
        assert!(params.contains("-verbose"));
        assert!(!params.contains("-VERBOSE"));
    }
}