// Source: ~/claudecode/openclaudecode/src/utils/bundledMode.ts
//! Detects if the current runtime is Bun or running as a Bun-compiled standalone executable.
//!
//! In Rust, these checks are adapted from the Bun-specific implementations.

#![allow(dead_code)]

/// Detects if the current runtime is Bun.
/// Returns true when:
/// - Running a JS file via the `bun` command
/// - Running a Bun-compiled standalone executable
///
/// In Rust builds, this always returns false since we're not running under Bun.
pub fn is_running_with_bun() -> bool {
    // https://bun.com/guides/util/detect-bun
    // In Rust, we check for BUN_VERSION env var as a proxy
    std::env::var("BUN_VERSION").is_ok()
}

/// Detects if running as a Bun-compiled standalone executable.
/// This checks for embedded files which are present in compiled binaries.
///
/// In Rust, we check for the presence of embedded resources or
/// the BUN_EMBEDDED environment variable set by Bun's bundler.
pub fn is_in_bundled_mode() -> bool {
    // In Bun: typeof Bun !== 'undefined' && Array.isArray(Bun.embeddedFiles) && Bun.embeddedFiles.length > 0
    // In Rust: check for BUN_EMBEDDED env var (set by Bun bundler) or
    // check if we have embedded files in the binary
    std::env::var("BUN_EMBEDDED").is_ok()
        || has_embedded_files()
}

/// Check if the binary has embedded files.
/// In a Bun-compiled binary, embedded files are accessible via Bun.embeddedFiles.
/// In Rust, this would typically be set at build time via build scripts.
fn has_embedded_files() -> bool {
    // Rust equivalent: check for a build-time constant or embedded resource indicator
    // For now, check an env var that would be set during a bundled build
    std::env::var("AI_CODE_BUNDLED").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_running_with_bun() {
        // In normal Rust test environment, this should be false
        assert!(!is_running_with_bun());
    }

    #[test]
    fn test_is_in_bundled_mode() {
        // In normal Rust test environment, this should be false
        assert!(!is_in_bundled_mode());
    }
}
