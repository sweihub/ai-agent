// Source: ~/claudecode/openclaudecode/src/utils/permissions/dangerousPatterns.ts
#![allow(dead_code)]

//! Pattern lists for dangerous shell-tool allow-rule prefixes.
//!
//! An allow rule like `Bash(python:*)` or `PowerShell(node:*)` lets the model
//! run arbitrary code via that interpreter, bypassing the auto-mode classifier.
//! These lists feed the is_dangerous_bash_permission and is_dangerous_power_shell_permission
//! predicates in permission_setup.

/// Cross-platform code-execution entry points present on both Unix and Windows.
/// Shared to prevent the two lists drifting apart on interpreter additions.
pub const CROSS_PLATFORM_CODE_EXEC: &[&str] = &[
    // Interpreters
    "python", "python3", "python2", "node", "deno", "tsx", "ruby", "perl", "php", "lua",
    // Package runners
    "npx", "bunx", "npm run", "yarn run", "pnpm run", "bun run",
    // Shells reachable from both (Git Bash / WSL on Windows, native on Unix)
    "bash", "sh", // Remote arbitrary-command wrapper (native OpenSSH on Win10+)
    "ssh",
];

/// Dangerous bash patterns for auto-mode.
/// Includes cross-platform code exec plus Unix-specific patterns.
pub fn dangerous_bash_patterns() -> Vec<&'static str> {
    let mut patterns: Vec<&'static str> = CROSS_PLATFORM_CODE_EXEC.to_vec();
    patterns.extend(&["zsh", "fish", "eval", "exec", "env", "xargs", "sudo"]);

    // Ant-only patterns (empirical-risk based on sandbox data)
    if std::env::var("USER_TYPE").as_deref() == Ok("ant") {
        patterns.extend(&[
            "fa run", // Cluster code launcher
            "coo",    // Network/exfil
            "gh", "gh api", "curl", "wget",
            // git config core.sshCommand / hooks install = arbitrary code
            "git", // Cloud resource writes
            "kubectl", "aws", "gcloud", "gsutil",
        ]);
    }

    patterns
}
