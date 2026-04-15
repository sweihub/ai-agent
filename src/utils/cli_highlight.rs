// Source: ~/claudecode/openclaudecode/src/utils/cliHighlight.ts
//! CLI highlight utilities.
//!
//! Provides lazy-loading of cli-highlight and highlight.js for syntax
//! highlighting in terminal output. All callers are telemetry (OTel counter
//! attributes, permission-dialog unary events) — none block on this, they
//! fire-and-forget or the consumer already handles Promise.

#![allow(dead_code)]

use std::path::Path;

/// Result of loading CLI highlight support.
pub struct CliHighlight {
    /// Highlight a string of code for terminal output.
    pub highlight: fn(&str, &str) -> Option<String>,
    /// Check if a language is supported.
    pub supports_language: fn(&str) -> bool,
}

/// One-time loaded state for CLI highlight.
static mut LOADED_GET_LANGUAGE: Option<fn(&str) -> Option<String>> = None;

/// Get the CLI highlight promise (lazy-loaded).
/// Returns None if highlighting failed to load.
pub fn get_cli_highlight() -> Option<&'static CliHighlight> {
    static HIGHLIGHT: std::sync::OnceLock<Option<CliHighlight>> = std::sync::OnceLock::new();
    HIGHLIGHT
        .get_or_init(|| {
            // In a Rust build, we don't have cli-highlight. Instead, we provide
            // a simplified highlight implementation or return None.
            // The original TypeScript version dynamically imports cli-highlight.
            None
        })
        .as_ref()
}

/// Get the language name for a file path.
/// e.g. "foo/bar.ts" -> "TypeScript".
///
/// Reads highlight.js's language registry. All callers are telemetry
/// (OTel counter attributes, permission-dialog unary events) — none block
/// on this, they fire-and-forget or the consumer already handles async.
pub async fn get_language_name(file_path: &str) -> String {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if ext.is_empty() {
        return "unknown".to_string();
    }

    // Map common extensions to language names.
    // This is a simplified version of what highlight.js provides.
    match ext.to_lowercase().as_str() {
        "js" | "jsx" | "mjs" | "cjs" => "JavaScript",
        "ts" | "tsx" | "mts" | "cts" => "TypeScript",
        "rs" => "Rust",
        "py" | "pyw" => "Python",
        "go" => "Go",
        "java" => "Java",
        "c" | "h" => "C",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "C++",
        "rb" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "kt" | "kts" => "Kotlin",
        "scala" => "Scala",
        "hs" => "Haskell",
        "ml" | "mli" => "OCaml",
        "cs" => "C#",
        "fs" | "fsx" => "F#",
        "vb" => "Visual Basic",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" => "SCSS",
        "sass" => "Sass",
        "less" => "Less",
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "xml" => "XML",
        "md" | "markdown" | "mdx" => "Markdown",
        "sql" => "SQL",
        "sh" | "bash" | "zsh" => "Bash",
        "fish" => "Fish",
        "ps1" => "PowerShell",
        "dockerfile" => "Dockerfile",
        "makefile" => "Makefile",
        "lua" => "Lua",
        "r" => "R",
        "dart" => "Dart",
        "ex" | "exs" => "Elixir",
        "erl" | "hrl" => "Erlang",
        "clj" | "cljs" => "Clojure",
        "groovy" | "gvy" => "Groovy",
        "perl" | "pl" | "pm" => "Perl",
        "proto" => "Protocol Buffers",
        "graphql" | "gql" => "GraphQL",
        "diff" | "patch" => "Diff",
        "ini" | "cfg" => "INI",
        "csv" => "CSV",
        "tf" | "hcl" => "HCL",
        "vue" => "Vue",
        "svelte" => "Svelte",
        _ => "unknown",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_language_name_typescript() {
        assert_eq!(get_language_name("foo/bar.ts").await, "TypeScript");
    }

    #[tokio::test]
    async fn test_get_language_name_rust() {
        assert_eq!(get_language_name("src/main.rs").await, "Rust");
    }

    #[tokio::test]
    async fn test_get_language_name_no_extension() {
        assert_eq!(get_language_name("Makefile").await, "unknown");
    }

    #[tokio::test]
    async fn test_get_language_name_empty_path() {
        assert_eq!(get_language_name("").await, "unknown");
    }
}
