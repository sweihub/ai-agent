//! REPL Bridge Handle types.
//!
//! Translated from openclaudecode/src/bridge/replBridgeHandle.ts

use crate::session_history::SDKMessage;

/// Handle for interacting with the REPL bridge.
/// This is the main interface for sending/receiving messages through the bridge.
pub trait ReplBridgeHandle: Send + Sync {
    /// Get the bridge session ID (session_* or cse_* form).
    fn bridge_session_id(&self) -> &str;

    /// Get the environment ID (empty for env-less v2 bridge).
    fn environment_id(&self) -> &str;

    /// Get the session ingress URL.
    fn session_ingress_url(&self) -> &str;

    /// Write messages to the bridge.
    fn write_messages(&self, messages: Vec<SDKMessage>);

    /// Write SDK messages directly to the bridge.
    fn write_sdk_messages(&self, messages: Vec<SDKMessage>);

    /// Send a control request.
    fn send_control_request(&self, request: crate::bridge::BridgeControlRequest);

    /// Send a control response.
    fn send_control_response(&self, response: crate::bridge::BridgeControlResponse);

    /// Send a control cancel request.
    fn send_control_cancel_request(&self, request_id: &str);

    /// Send a result message.
    fn send_result(&self);

    /// Tear down the bridge connection.
    async fn teardown(&self);
}

/// Bridge connection state.
#[derive(Debug, Clone, PartialEq)]
pub enum BridgeState {
    Ready,
    Connected,
    Reconnecting,
    Failed,
}

impl std::fmt::Display for BridgeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeState::Ready => write!(f, "ready"),
            BridgeState::Connected => write!(f, "connected"),
            BridgeState::Reconnecting => write!(f, "reconnecting"),
            BridgeState::Failed => write!(f, "failed"),
        }
    }
}

/// Bridge control request types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BridgeControlRequest {
    /// Request to use a tool.
    #[serde(rename = "control_request")]
    ToolUse {
        request_id: String,
        request: ToolUseRequest,
    },
    /// Request to begin a new conversation turn.
    #[serde(rename = "control_request")]
    TurnStart {
        request_id: String,
        request: TurnStartRequest,
    },
}

/// Tool use request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolUseRequest {
    pub name: String,
    pub input: serde_json::Value,
}

/// Turn start request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TurnStartRequest {
    pub turn_id: String,
}

/// Bridge control response types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BridgeControlResponse {
    /// Response to a tool use request.
    #[serde(rename = "control_response")]
    ToolUse {
        request_id: String,
        response: ToolUseResponse,
    },
    /// Response indicating the turn is complete.
    #[serde(rename = "control_response")]
    TurnEnd {
        request_id: String,
        response: TurnEndResponse,
    },
}

/// Tool use response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolUseResponse {
    pub content: Vec<serde_json::Value>,
}

/// Turn end response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TurnEndResponse {
    /// The model that produced the response.
    #[serde(default)]
    pub model: Option<String>,
    /// Total number of tokens used.
    #[serde(default)]
    pub tokens: Option<u32>,
}

/// Session state for reporting to CCR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    Idle,
    Running,
    RequiresAction,
}
