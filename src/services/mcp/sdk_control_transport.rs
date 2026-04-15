// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/SdkControlTransport.ts
//! SDK control transport - internal MCP transport for SDK-controlled connections

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// SDK control transport configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SdkControlTransportConfig {
    pub server_name: String,
    #[serde(rename = "transportType")]
    pub transport_type: String,
    #[serde(default)]
    pub init_timeout_ms: Option<u64>,
    #[serde(default)]
    pub request_timeout_ms: Option<u64>,
}

/// Request message sent over the transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportRequest {
    pub id: u64,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Response message received from the transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportResponse {
    pub id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<TransportError>,
}

/// Error in transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportNotification {
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Message types that can be sent/received
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransportMessage {
    Request(TransportRequest),
    Response(TransportResponse),
    Notification(TransportNotification),
}

/// Pending request awaiting response
struct PendingRequest {
    request: TransportRequest,
    response_tx: tokio::sync::oneshot::Sender<Result<TransportResponse, String>>,
}

/// SDK control transport handle - manages communication with internal SDK connections
#[derive(Clone)]
pub struct SdkControlTransport {
    pub server_name: String,
    pub connected: bool,
    inner: Arc<Mutex<TransportInner>>,
}

struct TransportInner {
    config: SdkControlTransportConfig,
    connected: bool,
    request_id: u64,
    pending_requests: HashMap<u64, PendingRequest>,
    message_log: Vec<TransportMessage>,
    max_log_size: usize,
    notification_handlers: Vec<Arc<dyn Fn(TransportNotification) + Send + Sync>>,
}

impl SdkControlTransport {
    /// Create a new SDK control transport
    pub fn new(config: &SdkControlTransportConfig) -> Self {
        log::debug!(
            "Creating SDK control transport for server: {}, type: {}",
            config.server_name,
            config.transport_type
        );

        Self {
            server_name: config.server_name.clone(),
            connected: false,
            inner: Arc::new(Mutex::new(TransportInner {
                config: config.clone(),
                connected: false,
                request_id: 0,
                pending_requests: HashMap::new(),
                message_log: Vec::new(),
                max_log_size: 1000,
                notification_handlers: Vec::new(),
            })),
        }
    }

    /// Initialize the transport connection
    pub async fn connect(&mut self) -> Result<(), String> {
        {
            let mut inner = self.inner.lock().await;
            log::debug!("Connecting SDK transport for server: {}", self.server_name);
            inner.connected = true;
        }
        self.connected = true;

        // Log successful connection
        log::info!(
            "SDK transport connected for server: {}",
            self.server_name
        );

        Ok(())
    }

    /// Disconnect the transport
    pub async fn disconnect(&mut self) {
        {
            let mut inner = self.inner.lock().await;
            inner.connected = false;
        }
        self.connected = false;

        // Fail all pending requests
        let mut inner = self.inner.lock().await;
        for (_, pending) in inner.pending_requests.drain() {
            let _ = pending
                .response_tx
                .send(Err("Transport disconnected".to_string()));
        }

        log::info!(
            "SDK transport disconnected for server: {}",
            self.server_name
        );
    }

    /// Check if the transport is connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Send a request and wait for response
    pub async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<TransportResponse, String> {
        let (request, response_rx) = {
            let mut inner = self.inner.lock().await;

            if !inner.connected {
                return Err("Transport is not connected".to_string());
            }

            inner.request_id += 1;
            let id = inner.request_id;

            let request = TransportRequest {
                id,
                method: method.to_string(),
                params,
            };

            let (tx, rx) = tokio::sync::oneshot::channel();
            inner.pending_requests.insert(
                id,
                PendingRequest {
                    request: request.clone(),
                    response_tx: tx,
                },
            );

            // Log the message
            inner.log_message(TransportMessage::Request(request.clone()));

            (request, rx)
        };
        let timeout_ms = self
            .inner
            .lock()
            .await
            .config
            .request_timeout_ms
            .unwrap_or(30_000);

        match tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            response_rx,
        )
        .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => Err(format!("Request cancelled: {}", e)),
            Err(_) => {
                // Remove pending request on timeout
                let mut inner = self.inner.lock().await;
                inner.pending_requests.remove(&request.id);
                Err(format!(
                    "Request timed out after {}ms for method: {}",
                    timeout_ms, request.method
                ))
            }
        }
    }

    /// Receive a response from the SDK (called by the SDK runtime)
    pub async fn receive_response(&self, response: TransportResponse) -> Result<(), String> {
        let mut inner = self.inner.lock().await;

        if let Some(pending) = inner.pending_requests.remove(&response.id) {
            let _ = pending.response_tx.send(Ok(response.clone()));
            inner.log_message(TransportMessage::Response(response));
            Ok(())
        } else {
            Err(format!(
                "No pending request found for response id: {}",
                response.id
            ))
        }
    }

    /// Receive a notification from the SDK
    pub async fn receive_notification(&self, notification: TransportNotification) {
        let handlers = {
            let mut inner = self.inner.lock().await;
            inner.log_message(TransportMessage::Notification(notification.clone()));
            inner.notification_handlers.clone()
        };

        for handler in handlers {
            handler(notification.clone());
        }
    }

    /// Send a notification to the SDK
    pub async fn send_notification(&self, method: &str, params: Option<serde_json::Value>) {
        let inner = self.inner.lock().await;

        if !inner.connected {
            log::warn!(
                "Cannot send notification - transport not connected for server: {}",
                self.server_name
            );
            return;
        }

        let notification = TransportNotification {
            method: method.to_string(),
            params,
        };

        drop(inner);

        let mut inner = self.inner.lock().await;
        inner.log_message(TransportMessage::Notification(notification));
        drop(inner);

        // In actual implementation, this would be sent over the wire
        log::debug!(
            "Sending notification to SDK transport: {}",
            method
        );
    }

    /// Initialize the MCP server (send initialize request)
    pub async fn initialize(
        &self,
        init_params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let response = self.send_request("initialize", init_params).await?;

        response
            .result
            .ok_or_else(|| "Initialize response missing result".to_string())
    }

    /// Send initialized notification
    pub async fn send_initialized(&self) {
        self.send_notification("notifications/initialized", None)
            .await;
    }

    /// Register a notification handler
    pub async fn on_notification(
        &self,
        handler: impl Fn(TransportNotification) + Send + Sync + 'static,
    ) {
        let mut inner = self.inner.lock().await;
        inner
            .notification_handlers
            .push(Arc::new(handler));
    }

    /// Get the message log
    pub async fn get_message_log(&self) -> Vec<TransportMessage> {
        let inner = self.inner.lock().await;
        inner.message_log.clone()
    }

    /// Clear the message log
    pub async fn clear_message_log(&self) {
        let mut inner = self.inner.lock().await;
        inner.message_log.clear();
    }

    /// Get server name
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// Get transport type from config
    pub async fn transport_type(&self) -> String {
        let inner = self.inner.lock().await;
        inner.config.transport_type.clone()
    }
}

impl TransportInner {
    fn log_message(&mut self, message: TransportMessage) {
        self.message_log.push(message);
        // Trim old messages if over limit
        if self.message_log.len() > self.max_log_size {
            let excess = self.message_log.len() - self.max_log_size;
            self.message_log.drain(..excess);
        }
    }
}

impl std::fmt::Debug for SdkControlTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SdkControlTransport")
            .field("server_name", &self.server_name)
            .field("connected", &self.connected)
            .finish()
    }
}

/// Create an SDK control transport
pub fn create_sdk_control_transport(config: &SdkControlTransportConfig) -> SdkControlTransport {
    SdkControlTransport::new(config)
}

/// Create a default SDK control transport for a server
pub fn create_default_sdk_transport(server_name: &str) -> SdkControlTransport {
    SdkControlTransport::new(&SdkControlTransportConfig {
        server_name: server_name.to_string(),
        transport_type: "stdio".to_string(),
        init_timeout_ms: Some(10_000),
        request_timeout_ms: Some(30_000),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transport() {
        let config = SdkControlTransportConfig {
            server_name: "test-server".to_string(),
            transport_type: "stdio".to_string(),
            ..Default::default()
        };
        let transport = create_sdk_control_transport(&config);
        assert_eq!(transport.server_name(), "test-server");
        assert!(!transport.is_connected());
    }

    #[test]
    fn test_create_default_transport() {
        let transport = create_default_sdk_transport("my-server");
        assert_eq!(transport.server_name(), "my-server");
    }

    #[test]
    fn test_transport_config_defaults() {
        let config = SdkControlTransportConfig {
            server_name: "test".to_string(),
            transport_type: "stdio".to_string(),
            ..Default::default()
        };
        assert!(config.init_timeout_ms.is_none());
        assert!(config.request_timeout_ms.is_none());
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let config = SdkControlTransportConfig {
            server_name: "test".to_string(),
            transport_type: "stdio".to_string(),
            ..Default::default()
        };
        let mut transport = SdkControlTransport::new(&config);
        assert!(!transport.is_connected());

        transport.connect().await.unwrap();
        assert!(transport.is_connected());

        transport.disconnect().await;
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_send_request_not_connected() {
        let config = SdkControlTransportConfig {
            server_name: "test".to_string(),
            transport_type: "stdio".to_string(),
            ..Default::default()
        };
        let transport = SdkControlTransport::new(&config);
        let result = transport.send_request("test", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not connected"));
    }

    #[test]
    fn test_debug_format() {
        let config = SdkControlTransportConfig {
            server_name: "test".to_string(),
            transport_type: "stdio".to_string(),
            ..Default::default()
        };
        let transport = SdkControlTransport::new(&config);
        let debug_str = format!("{:?}", transport);
        assert!(debug_str.contains("SdkControlTransport"));
        assert!(debug_str.contains("test"));
    }

    #[tokio::test]
    async fn test_clear_message_log() {
        let config = SdkControlTransportConfig {
            server_name: "test".to_string(),
            transport_type: "stdio".to_string(),
            ..Default::default()
        };
        let mut transport = SdkControlTransport::new(&config);
        transport.connect().await.unwrap();
        transport.send_notification("test", None).await;
        let log = transport.get_message_log().await;
        assert!(!log.is_empty());
        transport.clear_message_log().await;
        let log = transport.get_message_log().await;
        assert!(log.is_empty());
    }
}
