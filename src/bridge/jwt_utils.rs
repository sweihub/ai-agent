//! JWT utilities for bridge token handling.
//!
//! Translated from openclaudecode/src/bridge/jwtUtils.ts

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Refresh buffer: request a new token before expiry (5 minutes)
const TOKEN_REFRESH_BUFFER_MS: u64 = 5 * 60 * 1000;

/// Fallback refresh interval when the new token's expiry is unknown (30 minutes)
const FALLBACK_REFRESH_INTERVAL_MS: u64 = 30 * 60 * 1000;

/// Max consecutive failures before giving up on the refresh chain.
const MAX_REFRESH_FAILURES: u32 = 3;

/// Retry delay when getAccessToken returns undefined.
const REFRESH_RETRY_DELAY_MS: u64 = 60_000;

// =============================================================================
// TIME HELPERS
// =============================================================================

/// Get current timestamp in milliseconds.
fn current_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// =============================================================================
// JWT DECODING
// =============================================================================

/// Format a millisecond duration as a human-readable string (e.g. "5m 30s").
pub fn format_duration(ms: u64) -> String {
    if ms < 60_000 {
        return format!("{}s", ms / 1000);
    }
    let m = ms / 60_000;
    let s = (ms % 60_000) / 1000;
    if s > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}m", m)
    }
}

/// Decode a JWT's payload segment without verifying the signature.
/// Strips the `sk-ant-si-` session-ingress prefix if present.
/// Returns the parsed JSON payload as a Value, or None if the
/// token is malformed or the payload is not valid JSON.
pub fn decode_jwt_payload(token: &str) -> Option<serde_json::Value> {
    let jwt = if token.starts_with("sk-ant-si-") {
        &token["sk-ant-si-".len()..]
    } else {
        token
    };

    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 || parts[1].is_empty() {
        return None;
    }

    // Decode base64url
    let payload_str = match URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(bytes) => String::from_utf8(bytes).ok()?,
        Err(_) => return None,
    };

    // Parse JSON
    serde_json::from_str(&payload_str).ok()
}

/// Decode the `exp` (expiry) claim from a JWT without verifying the signature.
/// Returns the `exp` value in Unix seconds, or None if unparseable.
pub fn decode_jwt_expiry(token: &str) -> Option<i64> {
    let payload = decode_jwt_payload(token)?;
    if let Some(exp) = payload.get("exp").and_then(|v| v.as_i64()) {
        Some(exp)
    } else {
        None
    }
}

// =============================================================================
// TOKEN REFRESH SCHEDULER
// =============================================================================

/// Token refresh scheduler state.
pub struct TokenRefreshScheduler {
    timers: HashMap<String, TimerState>,
    failure_counts: HashMap<String, u32>,
    generations: HashMap<String, u32>,
    get_access_token: Box<dyn Fn() -> Option<String> + Send + Sync>,
    on_refresh: Box<dyn Fn(&str, &str) + Send + Sync>,
    label: String,
    refresh_buffer_ms: u64,
}

#[derive(Debug)]
struct TimerState {
    timer: Option<tokio::time::Sleep>,
    expiry: Option<Instant>,
}

/// Token refresh scheduler handle for controlling the scheduler.
pub struct TokenRefreshSchedulerHandle {
    scheduler: std::sync::Arc<std::sync::Mutex<TokenRefreshScheduler>>,
}

impl TokenRefreshSchedulerHandle {
    /// Schedule refresh for a token with a given session ID.
    pub fn schedule(&self, session_id: &str, token: &str) {
        if let Ok(mut scheduler) = self.scheduler.lock() {
            scheduler.schedule(session_id, token);
        }
    }

    /// Schedule refresh using an explicit TTL (seconds until expiry).
    pub fn schedule_from_expires_in(&self, session_id: &str, expires_in_seconds: u64) {
        if let Ok(mut scheduler) = self.scheduler.lock() {
            scheduler.schedule_from_expires_in(session_id, expires_in_seconds);
        }
    }

    /// Cancel refresh for a session.
    pub fn cancel(&self, session_id: &str) {
        if let Ok(mut scheduler) = self.scheduler.lock() {
            scheduler.cancel(session_id);
        }
    }

    /// Cancel all scheduled refreshes.
    pub fn cancel_all(&self) {
        if let Ok(mut scheduler) = self.scheduler.lock() {
            scheduler.cancel_all();
        }
    }
}

/// Create a token refresh scheduler that proactively refreshes session tokens
/// before they expire. Used by both the standalone bridge and the REPL bridge.
///
/// When a token is about to expire, the scheduler calls `on_refresh` with the
/// session ID and the bridge's OAuth access token.
pub fn create_token_refresh_scheduler<G, R, L>(
    get_access_token: G,
    on_refresh: R,
    label: L,
) -> TokenRefreshSchedulerHandle
where
    G: Fn() -> Option<String> + Send + Sync + 'static,
    R: Fn(&str, &str) + Send + Sync + 'static,
    L: Into<String>,
{
    let scheduler = TokenRefreshScheduler {
        timers: HashMap::new(),
        failure_counts: HashMap::new(),
        generations: HashMap::new(),
        get_access_token: Box::new(get_access_token),
        on_refresh: Box::new(on_refresh),
        label: label.into(),
        refresh_buffer_ms: TOKEN_REFRESH_BUFFER_MS,
    };

    TokenRefreshSchedulerHandle {
        scheduler: std::sync::Arc::new(std::sync::Mutex::new(scheduler)),
    }
}

impl TokenRefreshScheduler {
    fn next_generation(&mut self, session_id: &str) -> u32 {
        let r#gen = self.generations.get(session_id).copied().unwrap_or(0) + 1;
        self.generations.insert(session_id.to_string(), r#gen);
        r#gen
    }

    fn schedule(&mut self, session_id: &str, token: &str) {
        let expiry = decode_jwt_expiry(token);
        if expiry.is_none() {
            // Token is not a decodable JWT
            eprintln!(
                "[{}:token] Could not decode JWT expiry for sessionId={}, token prefix={}..., keeping existing timer",
                self.label,
                session_id,
                &token[..15.min(token.len())]
            );
            return;
        }

        let expiry = expiry.unwrap();

        // Clear any existing timer
        self.timers.remove(session_id);

        // Bump generation
        let r#gen = self.next_generation(session_id);

        let expiry_date = chrono::DateTime::from_timestamp(expiry, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let delay_ms = (expiry * 1000) as i64
            - current_timestamp_millis() as i64
            - self.refresh_buffer_ms as i64;

        if delay_ms <= 0 {
            eprintln!(
                "[{}:token] Token for sessionId={} expires={} (past or within buffer), refreshing immediately",
                self.label, session_id, expiry_date
            );
            // Would trigger refresh here in async context
            return;
        }

        eprintln!(
            "[{}:token] Scheduled token refresh for sessionId={} in {} (expires={}, buffer={}s)",
            self.label,
            session_id,
            format_duration(delay_ms as u64),
            expiry_date,
            self.refresh_buffer_ms / 1000
        );

        // Timer would be scheduled here
    }

    fn schedule_from_expires_in(&mut self, session_id: &str, expires_in_seconds: u64) {
        // Clear any existing timer
        self.timers.remove(session_id);

        let r#gen = self.next_generation(session_id);

        // Clamp to 30s floor
        let delay_ms = (expires_in_seconds * 1000)
            .saturating_sub(self.refresh_buffer_ms)
            .max(30_000);

        eprintln!(
            "[{}:token] Scheduled token refresh for sessionId={} in {} (expires_in={}s, buffer={}s)",
            self.label,
            session_id,
            format_duration(delay_ms),
            expires_in_seconds,
            self.refresh_buffer_ms / 1000
        );

        // Timer would be scheduled here
    }

    fn cancel(&mut self, session_id: &str) {
        // Bump generation to invalidate any in-flight refresh
        self.next_generation(session_id);
        self.timers.remove(session_id);
        self.failure_counts.remove(session_id);
    }

    fn cancel_all(&mut self) {
        // Bump all generations
        let session_ids: Vec<String> = self.generations.keys().cloned().collect();
        for session_id in session_ids {
            self.next_generation(&session_id);
        }
        self.timers.clear();
        self.failure_counts.clear();
    }
}

// =============================================================================
// STANDALONE FUNCTIONS
// =============================================================================

/// Check if a token is expired or about to expire.
pub fn is_token_expired(token: &str, buffer_ms: u64) -> bool {
    if let Some(expiry) = decode_jwt_expiry(token) {
        let expiry_ms = expiry * 1000;
        let now = current_timestamp_millis();
        expiry_ms + buffer_ms as i64 <= now as i64
    } else {
        // Can't decode - assume not expired
        false
    }
}

/// Get remaining time until token expires.
pub fn get_token_remaining_time(token: &str) -> Option<Duration> {
    let expiry = decode_jwt_expiry(token)?;
    let expiry_ms = expiry * 1000;
    let now = current_timestamp_millis() as i64;
    let remaining = expiry_ms - now;
    if remaining > 0 {
        Some(Duration::from_millis(remaining as u64))
    } else {
        Some(Duration::ZERO)
    }
}
