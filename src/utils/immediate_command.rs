//! Immediate command execution utilities
//!
//! Whether inference-config commands (/model, /fast, /effort) should execute
//! immediately (during a running query) rather than waiting for the current
//! turn to finish.

use crate::constants::env::ai;

/// Whether inference-config commands should execute immediately.
///
/// Always enabled for ants; gated by experiment for external users.
pub fn should_inference_config_command_be_immediate() -> bool {
    // For SDK, check environment variable
    if std::env::var(ai::USER_TYPE).unwrap_or_default() == "ant" {
        return true;
    }

    // TODO: Implement feature gate check for external users
    // This would require the growthbook integration
    false
}
