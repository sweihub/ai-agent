// Source: ~/claudecode/openclaudecode/src/types/fileSuggestion.ts

use serde::{Deserialize, Serialize};

/// A file suggestion represented as a flexible record type.
/// In TypeScript this is Record<string, unknown>, mapped to a HashMap in Rust.
pub type FileSuggestion = std::collections::HashMap<String, serde_json::Value>;
