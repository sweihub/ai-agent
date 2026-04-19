// Source: ~/claudecode/openclaudecode/src/services/remoteManagedSettings/securityCheck.jsx
//! Security check for managed settings validation.
//!
//! In the TypeScript implementation, this returns { safe: true, warnings: [] }
//! as a stub — the full implementation would validate managed settings before
//! applying them (React-specific in TS). For the Rust CLI port, this remains
//! a simple stub that always returns safe=true.

use serde::{Deserialize, Serialize};

/// Result of the managed settings security check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheckResult {
    /// Whether the managed settings are safe to apply.
    pub safe: bool,
    /// Warnings collected during the security check.
    /// Empty array when safe=true.
    pub warnings: Vec<String>,
}

/// Check if managed settings are safe to apply.
/// In the full implementation, this would validate:
/// - Settings values don't reference unsafe paths
/// - No shell injection in command-based settings
/// - Feature flags don't grant unexpected permissions
/// For the CLI port, all managed settings are considered safe.
pub async fn check_managed_settings_security() -> SecurityCheckResult {
    SecurityCheckResult {
        safe: true,
        warnings: Vec::new(),
    }
}

/// Handle the result of a security check.
/// In the TS implementation this is a no-op. In the Rust CLI,
/// we log any warnings if present.
pub fn handle_security_check_result(result: &SecurityCheckResult) {
    if !result.safe {
        log::warn!(
            "[remoteManagedSettings] Managed settings failed security check: {}",
            result.warnings.join("; ")
        );
    } else if !result.warnings.is_empty() {
        log::info!(
            "[remoteManagedSettings] Managed settings safe with warnings: {}",
            result.warnings.join("; ")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_check_returns_safe() {
        let result = check_managed_settings_security().await;
        assert!(result.safe);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_handle_safe_result_no_log() {
        let result = SecurityCheckResult {
            safe: true,
            warnings: Vec::new(),
        };
        // Should not log anything
        handle_security_check_result(&result);
    }

    #[test]
    fn test_handle_warnings_log_info() {
        let result = SecurityCheckResult {
            safe: true,
            warnings: vec!["deprecated_setting".to_string()],
        };
        handle_security_check_result(&result);
    }
}
