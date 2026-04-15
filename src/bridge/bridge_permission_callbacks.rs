//! Bridge permission callbacks.
//!
//! Translated from openclaudecode/src/bridge/bridgePermissionCallbacks.ts

use serde::{Deserialize, Serialize};

/// Permission update from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionUpdate {
    pub tool: Option<String>,
    pub allow: Option<Vec<String>>,
    pub deny: Option<Vec<String>>,
}

/// Bridge permission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgePermissionResponse {
    pub behavior: BridgePermissionBehavior,
    #[serde(rename = "updatedInput")]
    pub updated_input: Option<serde_json::Value>,
    #[serde(rename = "updatedPermissions")]
    pub updated_permissions: Option<Vec<PermissionUpdate>>,
    pub message: Option<String>,
}

/// Permission behavior type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BridgePermissionBehavior {
    Allow,
    Deny,
}

/// Bridge permission callbacks interface
pub trait BridgePermissionCallbacks: Send + Sync {
    fn send_request(
        &self,
        request_id: &str,
        tool_name: &str,
        input: &serde_json::Value,
        tool_use_id: &str,
        description: &str,
        permission_suggestions: Option<&[PermissionUpdate]>,
        blocked_path: Option<&str>,
    );

    fn send_response(&self, request_id: &str, response: &BridgePermissionResponse);

    /// Cancel a pending control_request so the web app can dismiss its prompt.
    fn cancel_request(&self, request_id: &str);

    fn on_response(
        &self,
        request_id: &str,
        handler: Box<dyn Fn(BridgePermissionResponse) + Send + Sync>,
    ) -> Box<dyn Fn() + Send + Sync>;
}

/// Type predicate for validating a parsed control_response payload
/// as a BridgePermissionResponse. Checks the required `behavior`
/// discriminant rather than using an unsafe `as` cast.
pub fn is_bridge_permission_response(value: &serde_json::Value) -> bool {
    let obj = match value.as_object() {
        Some(o) => o,
        None => return false,
    };

    obj.get("behavior")
        .and_then(|v| v.as_str())
        .map(|s| s == "allow" || s == "deny")
        .unwrap_or(false)
}

/// Simple in-memory implementation of permission callbacks for testing
pub struct InMemoryBridgePermissionCallbacks {
    responses: std::sync::Mutex<std::collections::HashMap<String, BridgePermissionResponse>>,
    response_handlers: std::sync::Mutex<
        std::collections::HashMap<String, Vec<Box<dyn Fn(BridgePermissionResponse) + Send + Sync>>>,
    >,
}

impl InMemoryBridgePermissionCallbacks {
    pub fn new() -> Self {
        Self {
            responses: std::sync::Mutex::new(std::collections::HashMap::new()),
            response_handlers: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl Default for InMemoryBridgePermissionCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

impl BridgePermissionCallbacks for InMemoryBridgePermissionCallbacks {
    fn send_request(
        &self,
        request_id: &str,
        tool_name: &str,
        input: &serde_json::Value,
        tool_use_id: &str,
        description: &str,
        _permission_suggestions: Option<&[PermissionUpdate]>,
        _blocked_path: Option<&str>,
    ) {
        println!(
            "[Permission] Request: {} tool={} tool_use_id={} description={} input={}",
            request_id, tool_name, tool_use_id, description, input
        );
    }

    fn send_response(&self, request_id: &str, response: &BridgePermissionResponse) {
        let mut responses = self.responses.lock().unwrap();
        responses.insert(request_id.to_string(), response.clone());

        // Trigger any registered handlers
        let handlers = self.response_handlers.lock().unwrap();
        if let Some(handler_list) = handlers.get(request_id) {
            for handler in handler_list {
                handler(response.clone());
            }
        }
    }

    fn cancel_request(&self, request_id: &str) {
        println!("[Permission] Cancel request: {}", request_id);
    }

    fn on_response(
        &self,
        request_id: &str,
        handler: Box<dyn Fn(BridgePermissionResponse) + Send + Sync>,
    ) -> Box<dyn Fn() + Send + Sync> {
        let mut handlers = self.response_handlers.lock().unwrap();
        handlers
            .entry(request_id.to_string())
            .or_insert_with(Vec::new)
            .push(handler);

        Box::new(move || {
            // Cleanup handler when dropped
        })
    }
}
