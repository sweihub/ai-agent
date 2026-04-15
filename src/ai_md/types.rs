// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
//! AI.md types

use serde::{Deserialize, Serialize};

/// Type of AI.md memory file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiMdType {
    /// Managed memory - global policy (e.g., /etc/ai-code/AI.md)
    Managed,
    /// User memory - private global (~/.ai/AI.md)
    User,
    /// Project memory - checked into codebase (./AI.md, .ai/AI.md)
    Project,
    /// Local memory - private project-specific (./AI.local.md)
    Local,
}

impl std::fmt::Display for AiMdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiMdType::Managed => write!(f, "managed"),
            AiMdType::User => write!(f, "user"),
            AiMdType::Project => write!(f, "project"),
            AiMdType::Local => write!(f, "local"),
        }
    }
}

/// Parsed frontmatter from AI.md file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiMdFrontmatter {
    /// Optional glob patterns for conditional rules
    #[serde(default)]
    pub paths: Option<Vec<String>>,
    /// Optional name field
    #[serde(default)]
    pub name: Option<String>,
    /// Optional description field
    #[serde(default)]
    pub description: Option<String>,
    /// Optional type field
    #[serde(default)]
    pub r#type: Option<String>,
}

/// AI.md file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMdFile {
    /// Absolute path to the file
    pub path: String,
    /// Type of memory file
    pub md_type: AiMdType,
    /// Parsed content (after stripping frontmatter)
    pub content: String,
    /// Raw content from disk (if different after processing)
    #[serde(default)]
    pub raw_content: Option<String>,
    /// Glob patterns from frontmatter for conditional rules
    #[serde(default)]
    pub globs: Option<Vec<String>>,
    /// Path of the file that included this one
    #[serde(default)]
    pub parent: Option<String>,
    /// Whether content differs from disk (partial view)
    #[serde(default)]
    pub content_differs_from_disk: bool,
}

/// Content with description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMdContent {
    /// File path
    pub path: String,
    /// Content text
    pub content: String,
    /// Type description
    pub type_description: String,
}

impl AiMdContent {
    /// Create a new AiMdContent
    pub fn new(path: String, content: String, md_type: AiMdType) -> Self {
        let type_description = match md_type {
            AiMdType::Managed => "(global policy instructions)".to_string(),
            AiMdType::User => "(user's private global instructions for all projects)".to_string(),
            AiMdType::Project => "(project instructions, checked into the codebase)".to_string(),
            AiMdType::Local => "(user's private project instructions, not checked in)".to_string(),
        };
        Self {
            path,
            content,
            type_description,
        }
    }
}

/// Result of parsing an AI.md file
#[derive(Debug)]
pub struct ParsedAiMd {
    pub frontmatter: AiMdFrontmatter,
    pub content: String,
}

/// File extensions allowed for @include directives
pub const TEXT_FILE_EXTENSIONS: &[&str] = &[
    // Markdown and text
    ".md",
    ".txt",
    ".text",
    // Data formats
    ".json",
    ".yaml",
    ".yml",
    ".toml",
    ".xml",
    ".csv",
    // Web
    ".html",
    ".htm",
    ".css",
    ".scss",
    ".sass",
    ".less",
    // JavaScript/TypeScript
    ".js",
    ".ts",
    ".tsx",
    ".jsx",
    ".mjs",
    ".cjs",
    ".mts",
    ".cts",
    // Python
    ".py",
    ".pyi",
    ".pyw",
    // Ruby
    ".rb",
    ".erb",
    ".rake",
    // Go
    ".go",
    // Rust
    ".rs",
    // Java/Kotlin/Scala
    ".java",
    ".kt",
    ".kts",
    ".scala",
    // C/C++
    ".c",
    ".cpp",
    ".cc",
    ".cxx",
    ".h",
    ".hpp",
    ".hxx",
    // C#
    ".cs",
    // Swift
    ".swift",
    // Shell
    ".sh",
    ".bash",
    ".zsh",
    ".fish",
    ".ps1",
    ".bat",
    ".cmd",
    // Config
    ".env",
    ".ini",
    ".cfg",
    ".conf",
    ".config",
    ".properties",
    // Database
    ".sql",
    ".graphql",
    ".gql",
    // Protocol
    ".proto",
    // Frontend frameworks
    ".vue",
    ".svelte",
    ".astro",
    // Templating
    ".ejs",
    ".hbs",
    ".pug",
    ".jade",
    // Other languages
    ".php",
    ".pl",
    ".pm",
    ".lua",
    ".r",
    ".R",
    ".dart",
    ".ex",
    ".exs",
    ".erl",
    ".hrl",
    ".clj",
    ".cljs",
    ".cljc",
    ".edn",
    ".hs",
    ".lhs",
    ".elm",
    ".ml",
    ".mli",
    ".f",
    ".f90",
    ".f95",
    ".for",
    // Build files
    ".cmake",
    ".make",
    ".makefile",
    ".gradle",
    ".sbt",
    // Documentation
    ".rst",
    ".adoc",
    ".asciidoc",
    ".org",
    ".tex",
    ".latex",
    // Lock files
    ".lock",
    // Misc
    ".log",
    ".diff",
    ".patch",
];

/// Check if a file extension is allowed for @include
pub fn is_allowed_extension(ext: &str) -> bool {
    TEXT_FILE_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}
