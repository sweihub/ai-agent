//! Message queue types

use std::collections::HashMap;

/// A message queue entry is a record with string keys and arbitrary values
pub type MessageQueueEntry = HashMap<String, serde_json::Value>;
