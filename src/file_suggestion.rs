//! File suggestion type definition.

use std::collections::HashMap;

/// A file suggestion record
pub type FileSuggestion = HashMap<String, serde_json::Value>;
