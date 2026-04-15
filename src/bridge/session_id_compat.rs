//! Session ID tag translation helpers for the CCR v2 compat layer.
//!
//! Lives in its own file (rather than workSecret.rs) so that sessionHandle and
//! replBridgeTransport can import from workSecret without pulling in these retag functions.
//!
//! The isCseShimEnabled gate is injected via set_cse_shim_gate() to avoid
//! a static import of bridgeEnabled -> growthbook -> config. Callers that
//! already import bridgeEnabled register the gate.

use std::sync::OnceLock;

// =============================================================================
// CSE SHIM GATE
// =============================================================================

static CSE_SHIM_GATE: OnceLock<Box<dyn Fn() -> bool + Send + Sync>> = OnceLock::new();

/// Register the GrowthBook gate for the cse_ shim. Called from bridge
/// init code that already imports bridgeEnabled.
pub fn set_cse_shim_gate(gate: impl Fn() -> bool + Send + Sync + 'static) {
    let _ = CSE_SHIM_GATE.set(Box::new(gate));
}

fn is_cse_shim_enabled() -> bool {
    CSE_SHIM_GATE
        .get()
        .map(|gate| gate())
        // Default to true if not set (matches TypeScript behavior)
        .unwrap_or(true)
}

// =============================================================================
// SESSION ID TRANSLATION
// =============================================================================

/// Re-tag a `cse_*` session ID to `session_*` for use with the v1 compat API.
///
/// Worker endpoints (/v1/code/sessions/{id}/worker/*) want `cse_*`; that's
/// what the work poll delivers. Client-facing compat endpoints
/// (/v1/sessions/{id}, /v1/sessions/{id}/archive, /v1/sessions/{id}/events)
/// want `session_*`. Same UUID, different costume. No-op for IDs that aren't `cse_*`.
///
/// bridgeMain holds one sessionId variable for both worker registration and
/// session-management calls. It arrives as `cse_*` from the work poll under
/// the compat gate, so archiveSession/fetchSessionTitle need this re-tag.
pub fn to_compat_session_id(id: &str) -> String {
    if !id.starts_with("cse_") {
        return id.to_string();
    }
    if !is_cse_shim_enabled() {
        return id.to_string();
    }
    format!("session_{}", &id["cse_".len()..])
}

/// Re-tag a `session_*` session ID to `cse_*` for infrastructure-layer calls.
///
/// Inverse of to_compat_session_id. POST /v1/environments/{id}/bridge/reconnect
/// lives below the compat layer: once ccr_v2_compat_enabled is on server-side,
/// it looks sessions up by their infra tag (`cse_*`). createBridgeSession still
/// returns `session_*` and that's what bridge-pointer stores — so perpetual
/// reconnect passes the wrong costume and gets "Session not found" back.
/// Same UUID, wrong tag. No-op for IDs that aren't `session_*`.
pub fn to_infra_session_id(id: &str) -> String {
    if !id.starts_with("session_") {
        return id.to_string();
    }
    format!("cse_{}", &id["session_".len()..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_compat_session_id() {
        // Non-cse_ IDs pass through
        assert_eq!(to_compat_session_id("abc123"), "abc123");

        // cse_ gets retagged
        assert_eq!(to_compat_session_id("cse_abc123"), "session_abc123");
    }

    #[test]
    fn test_to_infra_session_id() {
        // Non-session_ IDs pass through
        assert_eq!(to_infra_session_id("abc123"), "abc123");

        // session_ gets retagged
        assert_eq!(to_infra_session_id("session_abc123"), "cse_abc123");
    }
}
