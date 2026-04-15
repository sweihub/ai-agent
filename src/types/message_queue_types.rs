// Source: ~/claudecode/openclaudecode/src/types/messageQueueTypes.ts

use serde::{Deserialize, Serialize};

/// A message queue entry represented as a flexible record type.
/// In TypeScript this is Record<string, unknown>, mapped to a HashMap in Rust.
pub type MessageQueueEntry = std::collections::HashMap<String, serde_json::Value>;
