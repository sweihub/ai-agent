// Source: ~/claudecode/openclaudecode/src/utils/renderOptions.ts

use std::sync::OnceLock;

/// Cached stdin override - computed once per process.
static STDIN_OVERRIDE: OnceLock<Option<std::fs::File>> = OnceLock::new();

/// Gets a File for /dev/tty when stdin is piped.
/// This allows interactive rendering even when stdin is a pipe.
/// Result is cached for the lifetime of the process.
fn get_stdin_override() -> Option<&'static std::fs::File> {
    STDIN_OVERRIDE
        .get_or_init(|| {
            // No override needed if stdin is already a TTY
            if atty::is(atty::Stream::Stdin) {
                return None;
            }

            // Skip in CI environments (localized: CI)
            if is_env_truthy(std::env::var("CI").ok()) {
                return None;
            }

            // Skip if running MCP (input hijacking breaks MCP)
            let args: Vec<String> = std::env::args().collect();
            if args.iter().any(|a| a == "mcp") {
                return None;
            }

            // No /dev/tty on Windows
            if cfg!(target_os = "windows") {
                return None;
            }

            // Try to open /dev/tty as an alternative input source
            std::fs::File::open("/dev/tty").ok()
        })
        .as_ref()
}

/// Check if an environment variable is truthy.
fn is_env_truthy(value: Option<String>) -> bool {
    match value {
        Some(v) => {
            let v = v.to_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        None => false,
    }
}

/// Render options for the TUI.
#[derive(Debug, Clone, Default)]
pub struct RenderOptions {
    /// Whether to exit on Ctrl+C.
    pub exit_on_ctrl_c: bool,
}

/// Returns base render options, including stdin override when needed.
/// Use this for all render calls to ensure piped input works correctly.
pub fn get_base_render_options(exit_on_ctrl_c: bool) -> RenderOptions {
    let _stdin = get_stdin_override();
    RenderOptions { exit_on_ctrl_c }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_env_truthy() {
        assert!(is_env_truthy(Some("1".to_string())));
        assert!(is_env_truthy(Some("true".to_string())));
        assert!(!is_env_truthy(None));
        assert!(!is_env_truthy(Some("0".to_string())));
    }

    #[test]
    fn test_get_base_render_options() {
        let opts = get_base_render_options(false);
        assert!(!opts.exit_on_ctrl_c);

        let opts = get_base_render_options(true);
        assert!(opts.exit_on_ctrl_c);
    }
}
