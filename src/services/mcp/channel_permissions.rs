// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/channelPermissions.ts
//! Permission prompts over channels (Telegram, iMessage, Discord)

use std::collections::HashMap;
use std::sync::Mutex;

/// GrowthBook runtime gate - separate from the channels gate
/// Default false; flip without a release
pub fn is_channel_permission_relay_enabled() -> bool {
    // TODO: Integrate with GrowthBook feature flag 'tengu_harbor_permissions'
    false
}

/// Channel permission response
#[derive(Debug, Clone)]
pub struct ChannelPermissionResponse {
    pub behavior: PermissionBehavior,
    /// Which channel server the reply came from (e.g., "plugin:telegram:tg")
    pub from_server: String,
}

/// Permission behavior
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionBehavior {
    Allow,
    Deny,
}

/// Channel permission callbacks
pub struct ChannelPermissionCallbacks {
    handlers: Mutex<HashMap<String, Vec<Box<dyn Fn(ChannelPermissionResponse) + Send + Sync>>>>,
    pending: Mutex<HashMap<String, (PermissionBehavior, String)>>,
}

impl ChannelPermissionCallbacks {
    pub fn new() -> Self {
        Self {
            handlers: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashMap::new()),
        }
    }

    /// Register a resolver for a request ID
    pub fn on_response<F>(&self, request_id: &str, handler: F)
    where
        F: Fn(ChannelPermissionResponse) + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock().unwrap();
        handlers
            .entry(request_id.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }

    /// Resolve a pending request from a structured channel event
    /// Returns true if the ID was pending
    pub fn resolve(&self, request_id: &str, behavior: PermissionBehavior, from_server: &str) -> bool {
        let mut pending = self.pending.lock().unwrap();

        if let Some((stored_behavior, stored_server)) = pending.remove(request_id) {
            if stored_behavior == behavior && stored_server == from_server {
                // Notify handlers
                let handlers = self.handlers.lock().unwrap();
                if let Some(h) = handlers.get(request_id) {
                    let response = ChannelPermissionResponse {
                        behavior,
                        from_server: from_server.to_string(),
                    };
                    for handler in h {
                        handler(response.clone());
                    }
                }
                return true;
            }
        }
        false
    }
}

impl Default for ChannelPermissionCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

// Reply format spec for channel servers: /^\s*(y|yes|n|no)\s+([a-km-z]{5})\s*$/i
// 5 lowercase letters, no 'l' (looks like 1/I)
const PERMISSION_REPLY_RE: &str = r"^\s*(y|yes|n|no)\s+([a-km-z]{5})\s*$";

// 25-letter alphabet: a-z minus 'l' (looks like 1/I). 25^5 ≈ 9.8M space.
const ID_ALPHABET: &str = "abcdefghijkmnopqrstuvwxyz";

/// Generate a permission request ID (5 lowercase letters, no 'l')
pub fn generate_permission_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();

    let mut id = String::new();
    let alphabet: Vec<char> = ID_ALPHABET.chars().collect();

    for i in 0..5 {
        let idx = ((nanos >> (i * 3)) % (alphabet.len() as u32)) as usize;
        id.push(alphabet[idx]);
    }

    id
}