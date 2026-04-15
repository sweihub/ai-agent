//! Trusted device token source for bridge (remote-control) sessions.
//!
//! Translated from openclaudecode/src/bridge/trustedDevice.ts
//!
//! Bridge sessions have SecurityTier=ELEVATED on the server (CCR v2).
//! The server gates ConnectBridgeWorker on its own flag, this CLI-side
//! flag controls whether the CLI sends X-Trusted-Device-Token at all.

use crate::constants::env::ai;
use std::sync::{Arc, RwLock};

use reqwest;

// =============================================================================
// CONSTANTS
// =============================================================================

const TRUSTED_DEVICE_GATE: &str = "tengu_sessions_elevated_auth_enforcement";
const ENROLLMENT_TIMEOUT_MS: u64 = 10_000;

// =============================================================================
// STORAGE TRAIT (for dependency injection)
// =============================================================================

/// Trait for secure storage operations.
pub trait SecureStorage: Send + Sync {
    fn read(&self) -> Option<StorageData>;
    fn update(&self, data: &StorageData) -> Result<(), String>;
}

#[derive(Clone, Default)]
pub struct StorageData {
    pub trusted_device_token: Option<String>,
    pub device_id: Option<String>,
    // Add other fields as needed
}

// =============================================================================
// STATE
// =============================================================================

/// Gate check function type.
pub type GateFn = Box<dyn Fn(&str) -> bool + Send + Sync>;

/// Auth token getter function type.
pub type AuthTokenGetterFn = Box<dyn Fn() -> Option<String> + Send + Sync>;

/// Base URL getter function type.
pub type BaseUrlGetterFn = Box<dyn Fn() -> String + Send + Sync>;

static GATE_GETTER: std::sync::OnceLock<GateFn> = std::sync::OnceLock::new();
static AUTH_TOKEN_GETTER: std::sync::OnceLock<AuthTokenGetterFn> = std::sync::OnceLock::new();
static BASE_URL_GETTER: std::sync::OnceLock<BaseUrlGetterFn> = std::sync::OnceLock::new();
static STORAGE: std::sync::OnceLock<Arc<dyn SecureStorage>> = std::sync::OnceLock::new();

// Cached token storage
static CACHED_TOKEN: std::sync::OnceLock<RwLock<Option<String>>> = std::sync::OnceLock::new();

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Register the gate check function.
pub fn register_gate_check(gate: impl Fn(&str) -> bool + Send + Sync + 'static) {
    let _ = GATE_GETTER.set(Box::new(gate));
}

/// Register the auth token getter function.
pub fn register_auth_token_getter(getter: impl Fn() -> Option<String> + Send + Sync + 'static) {
    let _ = AUTH_TOKEN_GETTER.set(Box::new(getter));
}

/// Register the base URL getter function.
pub fn register_base_url_getter(getter: impl Fn() -> String + Send + Sync + 'static) {
    let _ = BASE_URL_GETTER.set(Box::new(getter));
}

/// Register the secure storage implementation.
pub fn register_secure_storage(storage: Arc<dyn SecureStorage>) {
    let _ = STORAGE.set(storage);
}

// =============================================================================
// GATE CHECK
// =============================================================================

fn is_gate_enabled() -> bool {
    GATE_GETTER
        .get()
        .map(|gate| gate(TRUSTED_DEVICE_GATE))
        // Default to false if not set
        .unwrap_or(false)
}

// =============================================================================
// TOKEN READ/WRITE
// =============================================================================

/// Get the stored trusted device token.
/// Uses env var override for testing/canary, falls back to secure storage.
/// Memoized for performance.
pub fn get_trusted_device_token() -> Option<String> {
    // Check env var first
    if let Ok(env_token) = std::env::var(ai::CLAUDE_TRUSTED_DEVICE_TOKEN) {
        if !env_token.is_empty() {
            return Some(env_token);
        }
    }

    if !is_gate_enabled() {
        return None;
    }

    // Use cached token if available
    if let Some(cached) = CACHED_TOKEN.get() {
        if let Ok(token) = cached.read() {
            return token.clone();
        }
    }

    // Read from storage
    let token = STORAGE
        .get()
        .and_then(|s| s.read())
        .and_then(|data| data.trusted_device_token);

    // Cache it
    if let Some(ref t) = token {
        if let Some(cache) = CACHED_TOKEN.get() {
            if let Ok(mut guard) = cache.write() {
                *guard = Some(t.clone());
            }
        }
    }

    token
}

/// Clear the cached trusted device token.
pub fn clear_trusted_device_token_cache() {
    if let Some(cache) = CACHED_TOKEN.get() {
        if let Ok(mut guard) = cache.write() {
            *guard = None;
        }
    }
}

/// Clear the stored trusted device token from secure storage and the cache.
/// Called before enrollTrustedDevice during /login so a stale token from the
/// previous account isn't sent while enrollment is in-flight.
pub fn clear_trusted_device_token() {
    if !is_gate_enabled() {
        return;
    }

    if let Some(storage) = STORAGE.get() {
        if let Some(mut data) = storage.read() {
            data.trusted_device_token = None;
            let _ = storage.update(&data);
        }
    }

    clear_trusted_device_token_cache();
}

/// Enroll this device via POST /auth/trusted_devices and persist the token
/// to storage. Best-effort — returns on failure so callers don't block.
pub async fn enroll_trusted_device() {
    // Check gate
    if !is_gate_enabled() {
        log_debug("[trusted-device] Gate is off, skipping enrollment");
        return;
    }

    // Check env var override
    if std::env::var(ai::CLAUDE_TRUSTED_DEVICE_TOKEN).is_ok() {
        log_debug(
            "[trusted-device] CLAUDE_TRUSTED_DEVICE_TOKEN env var is set, skipping enrollment",
        );
        return;
    }

    // Get access token
    let access_token = match AUTH_TOKEN_GETTER.get() {
        Some(getter) => getter(),
        None => {
            log_debug("[trusted-device] No auth token getter registered, skipping enrollment");
            return;
        }
    };

    let access_token = match access_token {
        Some(t) => t,
        None => {
            log_debug("[trusted-device] No OAuth token, skipping enrollment");
            return;
        }
    };

    // Get base URL
    let base_url = match BASE_URL_GETTER.get() {
        Some(getter) => getter(),
        None => {
            log_debug("[trusted-device] No base URL getter registered, skipping enrollment");
            return;
        }
    };

    let client = reqwest::Client::new();

    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "unknown".to_string());

    let platform = std::env::consts::OS;
    let display_name = format!("Claude Code on {} · {}", hostname, platform);

    match client
        .post(&format!("{}/api/auth/trusted_devices", base_url))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_millis(ENROLLMENT_TIMEOUT_MS))
        .json(&serde_json::json!({ "display_name": display_name }))
        .send()
        .await
    {
        Ok(response) => {
            if response.status() != 200 && response.status() != 201 {
                log_debug(&format!(
                    "[trusted-device] Enrollment failed {}",
                    response.status()
                ));
                return;
            }

            // Parse response
            match response.json::<serde_json::Value>().await {
                Ok(data) => {
                    let token = data.get("device_token").and_then(|v| v.as_str());
                    let device_id = data.get("device_id").and_then(|v| v.as_str());

                    match token {
                        Some(token) => {
                            // Persist to storage
                            if let Some(storage) = STORAGE.get() {
                                if let Some(mut data) = storage.read() {
                                    data.trusted_device_token = Some(token.to_string());
                                    if let Some(id) = device_id {
                                        data.device_id = Some(id.to_string());
                                    }
                                    match storage.update(&data) {
                                        Ok(_) => {
                                            clear_trusted_device_token_cache();
                                            log_debug(&format!(
                                                "[trusted-device] Enrolled device_id={}",
                                                device_id.unwrap_or("unknown")
                                            ));
                                        }
                                        Err(e) => {
                                            log_debug(&format!(
                                                "[trusted-device] Storage write failed: {}",
                                                e
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        None => {
                            log_debug(
                                "[trusted-device] Enrollment response missing device_token field",
                            );
                        }
                    }
                }
                Err(e) => {
                    log_debug(&format!("[trusted-device] Failed to parse response: {}", e));
                }
            }
        }
        Err(e) => {
            log_debug(&format!(
                "[trusted-device] Enrollment request failed: {}",
                e
            ));
        }
    }
}

// =============================================================================
// DEBUG LOGGING
// =============================================================================

fn log_debug(msg: &str) {
    // Simple debug logging - could be replaced with proper logging
    eprintln!("{}", msg);
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_returns_none_when_gate_disabled() {
        // By default gate returns false, so token should be None
        assert_eq!(get_trusted_device_token(), None);
    }

    #[test]
    fn test_clear_token_cache() {
        clear_trusted_device_token_cache();
        // Should not panic
    }
}
