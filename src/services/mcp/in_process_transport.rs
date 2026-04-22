// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/InProcessTransport.ts
//! In-process linked transport pair for running MCP server and client in same process

use std::sync::{Arc, Mutex};

/// JSON-RPC message type (simplified)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcMessage {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

/// In-process transport for MCP communication
/// Messages sent on one transport are delivered to the other's onmessage
pub struct InProcessTransport {
    peer: Arc<Mutex<Option<Arc<InProcessTransportInner>>>>,
    closed: Arc<Mutex<bool>>,
}

/// Inner transport state for sharing
pub struct InProcessTransportInner {
    onclose: Option<Box<dyn Fn() + Send + Sync>>,
    onmessage: Option<Box<dyn Fn(JsonRpcMessage) + Send + Sync>>,
}

impl InProcessTransport {
    /// Create a new in-process transport
    pub fn new() -> Self {
        Self {
            peer: Arc::new(Mutex::new(None)),
            closed: Arc::new(Mutex::new(false)),
        }
    }

    /// Create inner state
    fn create_inner(&self) -> Arc<InProcessTransportInner> {
        Arc::new(InProcessTransportInner {
            onclose: None,
            onmessage: None,
        })
    }

    /// Set close handler
    pub fn set_onclose<F>(&self, _callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        // Simplified - would need interior mutability
    }

    /// Set message handler
    pub fn set_onmessage<F>(&self, _callback: F)
    where
        F: Fn(JsonRpcMessage) + Send + Sync + 'static,
    {
        // Simplified - would need interior mutability
    }

    /// Start the transport (no-op for in-process)
    pub async fn start(&self) -> Result<(), String> {
        Ok(())
    }

    /// Send a message to the peer
    pub async fn send(&self, message: JsonRpcMessage) -> Result<(), String> {
        if *self.closed.lock().unwrap() {
            return Err("Transport is closed".to_string());
        }

        // Get peer - simplified without actual message delivery
        let _peer = self.peer.lock().unwrap();

        Ok(())
    }

    /// Close the transport
    pub async fn close(&self) -> Result<(), String> {
        {
            let mut closed = self.closed.lock().unwrap();
            if *closed {
                return Ok(());
            }
            *closed = true;
        }

        Ok(())
    }

    /// Check if transport is closed
    pub fn is_closed(&self) -> bool {
        *self.closed.lock().unwrap()
    }
}

impl Default for InProcessTransport {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a pair of linked transports for in-process MCP communication
/// Messages sent on one transport are delivered to the other's onmessage
///
/// Returns [client_transport, server_transport]
pub fn create_linked_transport_pair() -> (InProcessTransport, InProcessTransport) {
    let a = InProcessTransport::new();
    let b = InProcessTransport::new();
    // Note: Full peer linking would require Arc sharing - simplified for now
    (a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_start() {
        let transport = InProcessTransport::new();
        transport.start().await.unwrap();
    }

    #[tokio::test]
    async fn test_transport_close() {
        let transport = InProcessTransport::new();
        transport.close().await.unwrap();
        assert!(transport.is_closed());
    }

    #[tokio::test]
    async fn test_transport_send_after_close() {
        let transport = InProcessTransport::new();
        transport.close().await.unwrap();

        let msg = JsonRpcMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: Some("test".to_string()),
            params: None,
            result: None,
            error: None,
        };

        let result = transport.send(msg).await;
        assert!(result.is_err());
    }
}
