// Source: ~/claudecode/openclaudecode/src/types/notebook.ts

use serde::{Deserialize, Serialize};

/// A notebook cell represented as a flexible record type.
/// In TypeScript this is Record<string, unknown>, mapped to a HashMap in Rust.
pub type NotebookCell = std::collections::HashMap<String, serde_json::Value>;
