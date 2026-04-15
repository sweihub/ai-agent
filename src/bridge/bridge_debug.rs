//! Bridge debug utilities for fault injection.
//!
//! Translated from openclaudecode/src/bridge/bridgeDebug.ts
//!
//! Ant-only fault injection for manually testing bridge recovery paths.

use std::sync::{Arc, Mutex, RwLock};

// =============================================================================
// TYPES
// =============================================================================

/// One-shot fault to inject on the next matching API call.
#[derive(Debug, Clone)]
pub struct BridgeFault {
    pub method: BridgeFaultMethod,
    /// Fatal errors go through handleErrorStatus -> BridgeFatalError.
    /// Transient errors surface as plain rejections (5xx / network).
    pub kind: BridgeFaultKind,
    pub status: u16,
    pub error_type: Option<String>,
    /// Remaining injections. Decremented on consume; removed at 0.
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeFaultMethod {
    PollForWork,
    RegisterBridgeEnvironment,
    ReconnectSession,
    HeartbeatWork,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeFaultKind {
    Fatal,
    Transient,
}

/// Debug handle for bridge operations
pub trait BridgeDebugHandle: Send + Sync {
    /// Invoke the transport's permanent-close handler directly.
    fn fire_close(&self, code: u16);
    /// Call reconnectEnvironmentWithSession
    fn force_reconnect(&self);
    /// Queue a fault for the next N calls to the named api method.
    fn inject_fault(&self, fault: BridgeFault);
    /// Abort the at-capacity sleep so an injected poll fault lands immediately.
    fn wake_poll_loop(&self);
    /// env/session IDs for the debug.log grep.
    fn describe(&self) -> String;
}

// =============================================================================
// STATE
// =============================================================================

static DEBUG_HANDLE: std::sync::OnceLock<Arc<dyn BridgeDebugHandle>> = std::sync::OnceLock::new();
static FAULT_QUEUE: std::sync::OnceLock<Mutex<Vec<BridgeFault>>> = std::sync::OnceLock::new();
static DEBUG_HANDLE_MUT: std::sync::OnceLock<RwLock<Option<Arc<dyn BridgeDebugHandle>>>> =
    std::sync::OnceLock::new();

// =============================================================================
// FUNCTIONS
// =============================================================================

/// Register the debug handle.
pub fn register_bridge_debug_handle(h: Arc<dyn BridgeDebugHandle>) {
    let _ = DEBUG_HANDLE.set(h.clone());
    let _ = DEBUG_HANDLE_MUT.set(RwLock::new(Some(h)));
}

/// Clear the debug handle and fault queue.
pub fn clear_bridge_debug_handle() {
    // Clear fault queue
    if let Some(queue) = FAULT_QUEUE.get() {
        if let Ok(mut faults) = queue.lock() {
            faults.clear();
        }
    }
    // Clear handle
    if let Some(handle) = DEBUG_HANDLE_MUT.get() {
        if let Ok(mut guard) = handle.write() {
            *guard = None;
        }
    }
}

/// Get the debug handle.
pub fn get_bridge_debug_handle() -> Option<Arc<dyn BridgeDebugHandle>> {
    DEBUG_HANDLE.get().cloned()
}

/// Queue a fault for injection.
pub fn inject_bridge_fault(fault: BridgeFault) {
    let queue = FAULT_QUEUE.get_or_init(|| Mutex::new(Vec::new()));

    if let Ok(mut faults) = queue.lock() {
        eprintln!(
            "[bridge:debug] Queued fault: {:?} {}/{}{} ×{}",
            fault.method,
            fault.kind.as_str(),
            fault.status,
            fault
                .error_type
                .as_ref()
                .map(|e| format!("/{}", e))
                .unwrap_or_default(),
            fault.count
        );
        faults.push(fault);
    }
}

/// Consume a fault for the given method if one is queued.
pub fn consume_fault(method: &BridgeFaultMethod) -> Option<BridgeFault> {
    let queue = FAULT_QUEUE.get()?;

    let mut faults = match queue.lock() {
        Ok(f) => f,
        Err(_) => return None,
    };

    let idx = faults.iter().position(|f| &f.method == method)?;

    let mut fault = faults.remove(idx);
    fault.count -= 1;

    Some(fault)
}

/// Throw a fault as an error.
pub fn throw_fault(fault: &BridgeFault, context: &str) -> Result<(), String> {
    eprintln!(
        "[bridge:debug] Injecting {} fault into {}: status={} errorType={}",
        fault.kind.as_str(),
        context,
        fault.status,
        fault.error_type.as_deref().unwrap_or("none")
    );

    if fault.kind == BridgeFaultKind::Fatal {
        Err(format!("[injected] {} {}", context, fault.status))
    } else {
        // Transient: mimic a request failure
        Err(format!("[injected transient] {} {}", context, fault.status))
    }
}

impl BridgeFaultKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            BridgeFaultKind::Fatal => "fatal",
            BridgeFaultKind::Transient => "transient",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_fault() {
        let fault = BridgeFault {
            method: BridgeFaultMethod::PollForWork,
            kind: BridgeFaultKind::Fatal,
            status: 404,
            error_type: Some("not_found".to_string()),
            count: 1,
        };

        inject_bridge_fault(fault);

        // Should be able to consume it
        let consumed = consume_fault(&BridgeFaultMethod::PollForWork);
        assert!(consumed.is_some());

        // Should be gone now
        let consumed2 = consume_fault(&BridgeFaultMethod::PollForWork);
        assert!(consumed2.is_none());
    }
}
