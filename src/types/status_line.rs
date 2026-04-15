// Source: ~/claudecode/openclaudecode/src/types/statusLine.ts

use serde::{Deserialize, Serialize};

/// A status line item represented as a flexible record type.
/// In TypeScript this is Record<string, unknown>, mapped to a HashMap in Rust.
pub type StatusLineItem = std::collections::HashMap<String, serde_json::Value>;
