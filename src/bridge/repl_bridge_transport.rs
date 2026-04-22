//! REPL Bridge Transport abstraction.
//!
//! Translated from openclaudecode/src/bridge/replBridgeTransport.ts
//!
//! This module defines the transport abstraction used by the REPL bridge.
//! The SDK can use custom transports by implementing this trait.

use crate::bridge::SDKMessage;
use crate::bridge::repl_bridge_handle::SessionState;
use std::pin::Pin;
use std::sync::Arc;

/// Callback types for transport events.
pub type OnDataCallback = Arc<dyn Fn(String) + Send + Sync>;
pub type OnCloseCallback = Arc<dyn Fn(Option<u16>) + Send + Sync>;
pub type OnConnectCallback = Arc<dyn Fn() + Send + Sync>;

/// Transport abstraction for replBridge.
///
/// This trait defines the interface that transport implementations must provide.
/// It covers exactly the surface that replBridge uses against transports.
///
/// - v1: HybridTransport (WS reads + POST writes to Session-Ingress)
/// - v2: SSETransport (reads) + CCRClient (writes to CCR v2 /worker/*)
///
/// The v2 write path goes through CCRClient.writeEvent -> SerialBatchEventUploader,
/// NOT through SSETransport.write() — SSETransport.write() targets the
/// Session-Ingress POST URL shape, which is wrong for CCR v2.
pub trait ReplBridgeTransport: Send + Sync {
    /// Write a single message to the transport.
    fn write(&self, message: SDKMessage) -> BoxFuture<'_>;

    /// Write multiple messages in batch.
    fn write_batch(&self, messages: Vec<SDKMessage>) -> BoxFuture<'_>;

    /// Close the transport.
    fn close(&self);

    /// Check if the transport is connected (write-ready).
    fn is_connected_status(&self) -> bool;

    /// Get a human-readable state label for debugging.
    fn get_state_label(&self) -> String;

    /// Set callback for incoming data.
    fn set_on_data(&self, callback: OnDataCallback);

    /// Set callback for transport close.
    fn set_on_close(&self, callback: OnCloseCallback);

    /// Set callback for transport connect.
    fn set_on_connect(&self, callback: OnConnectCallback);

    /// Connect the transport.
    fn connect(&self);

    /// Get the high-water mark of the underlying read stream's event sequence numbers.
    /// This is used before swapping transports so the new one can resume from
    /// where the old one left off.
    ///
    /// v1 returns 0 — Session-Ingress WS doesn't use SSE sequence numbers;
    /// replay-on-reconnect is handled by the server-side message cursor.
    fn get_last_sequence_num(&self) -> u64;

    /// Get the monotonic count of batches dropped via maxConsecutiveFailures.
    /// Snapshot before writeBatch() and compare after to detect silent drops
    /// (writeBatch() resolves normally even when batches were dropped).
    /// v2 returns 0 — the v2 write path doesn't set maxConsecutiveFailures.
    fn dropped_batch_count(&self) -> u64;

    /// Report session state (v2 only; v1 is a no-op).
    /// `requires_action` tells the backend a permission prompt is pending.
    fn report_state(&self, state: SessionState);

    /// Report external metadata (v2 only; v1 is a no-op).
    fn report_metadata(&self, metadata: std::collections::HashMap<String, serde_json::Value>);

    /// Report delivery status (v2 only; v1 is a no-op).
    /// Populates CCR's processing_at/processed_at columns.
    fn report_delivery(&self, event_id: &str, status: DeliveryStatus);

    /// Drain the write queue before close() (v2 only; v1 resolves
    /// immediately — HybridTransport POSTs are already awaited per-write).
    fn flush(&self) -> BoxFuture<'_>;
}

/// Boxed future type for trait methods.
type BoxFuture<'a> = Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>>;

/// Delivery status for report_delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    Processing,
    Processed,
}

/// Builder for creating transport instances.
#[derive(Clone)]
pub struct ReplBridgeTransportBuilder {
    session_url: String,
    ingress_token: String,
    session_id: String,
    initial_sequence_num: Option<u64>,
    epoch: Option<u64>,
    heartbeat_interval_ms: Option<u64>,
    heartbeat_jitter_fraction: Option<f64>,
    outbound_only: Option<bool>,
}

impl ReplBridgeTransportBuilder {
    pub fn new(session_url: String, ingress_token: String, session_id: String) -> Self {
        Self {
            session_url,
            ingress_token,
            session_id,
            initial_sequence_num: None,
            epoch: None,
            heartbeat_interval_ms: None,
            heartbeat_jitter_fraction: None,
            outbound_only: None,
        }
    }

    pub fn with_initial_sequence_num(mut self, seq: u64) -> Self {
        self.initial_sequence_num = Some(seq);
        self
    }

    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.epoch = Some(epoch);
        self
    }

    pub fn with_heartbeat_interval_ms(mut self, interval: u64) -> Self {
        self.heartbeat_interval_ms = Some(interval);
        self
    }

    pub fn with_heartbeat_jitter_fraction(mut self, fraction: f64) -> Self {
        self.heartbeat_jitter_fraction = Some(fraction);
        self
    }

    pub fn with_outbound_only(mut self, outbound_only: bool) -> Self {
        self.outbound_only = Some(outbound_only);
        self
    }

    /// Build the transport.
    /// In the SDK, this returns an error as the actual transport implementations
    /// are CLI-specific. The SDK user should provide their own transport.
    pub fn build(&self) -> Result<Box<dyn ReplBridgeTransport>, BridgeTransportError> {
        Err(BridgeTransportError::TransportNotAvailable {
            message: "Transport implementation not available in SDK. Provide your own transport."
                .to_string(),
        })
    }
}

/// Errors that can occur with bridge transport.
#[derive(Debug)]
pub enum BridgeTransportError {
    TransportNotAvailable { message: String },
    ConnectionFailed { message: String },
    InitializationFailed { message: String },
}

impl std::fmt::Display for BridgeTransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeTransportError::TransportNotAvailable { message } => {
                write!(f, "Transport not available: {}", message)
            }
            BridgeTransportError::ConnectionFailed { message } => {
                write!(f, "Connection failed: {}", message)
            }
            BridgeTransportError::InitializationFailed { message } => {
                write!(f, "Initialization failed: {}", message)
            }
        }
    }
}

impl std::error::Error for BridgeTransportError {}
