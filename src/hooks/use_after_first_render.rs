// Source: ~/claudecode/openclaudecode/src/hooks/useAfterFirstRender.ts
#![allow(dead_code)]

use std::process;
use std::time::Instant;

use crate::env_utils::is_env_truthy;

/// Runs exit logic after first render if environment variables indicate.
///
/// Translation of the React `useEffect` with empty dependency array.
/// In Rust this is a plain initialization function.
pub fn after_first_render_init() {
    if std::env::var("USER_TYPE").as_deref() == Ok("ant")
        && is_env_truthy(
            &std::env::var("AI_CODE_EXIT_AFTER_FIRST_RENDER").unwrap_or_default(),
        )
    {
        let elapsed_ms = Instant::now().elapsed().as_millis();
        eprintln!("\nStartup time: {elapsed_ms}ms\n");
        process::exit(0);
    }
}
