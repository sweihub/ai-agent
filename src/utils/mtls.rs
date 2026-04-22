// Source: /data/home/swei/claudecode/openclaudecode/src/utils/mtls.ts
//! mTLS (mutual TLS) configuration utilities.

use crate::constants::env::ai_code;
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// mTLS configuration
#[derive(Debug, Clone, Default)]
pub struct MTLSConfig {
    pub cert: Option<String>,
    pub key: Option<String>,
    pub passphrase: Option<String>,
}

/// TLS configuration including mTLS and CA certificates
#[derive(Debug, Clone, Default)]
pub struct TLSConfig {
    pub cert: Option<String>,
    pub key: Option<String>,
    pub passphrase: Option<String>,
    pub ca: Option<Vec<String>>,
}

/// Check if mTLS is enabled
pub fn is_mtls_enabled() -> bool {
    std::env::var(ai_code::CLIENT_CERT).is_ok() || std::env::var(ai_code::CLIENT_KEY).is_ok()
}

/// Get mTLS configuration from environment variables
pub fn get_mtls_config() -> Option<MTLSConfig> {
    let mut config = MTLSConfig::default();

    if let Ok(cert_path) = std::env::var(ai_code::CLIENT_CERT) {
        if let Ok(cert) = std::fs::read_to_string(&cert_path) {
            config.cert = Some(cert);
        }
    }

    if let Ok(key_path) = std::env::var(ai_code::CLIENT_KEY) {
        if let Ok(key) = std::fs::read_to_string(&key_path) {
            config.key = Some(key);
        }
    }

    if let Ok(passphrase) = std::env::var(ai_code::CLIENT_KEY_PASSPHRASE) {
        config.passphrase = Some(passphrase);
    }

    if config.cert.is_none() && config.key.is_none() && config.passphrase.is_none() {
        None
    } else {
        Some(config)
    }
}

/// Get CA certificate
pub fn get_ca_cert() -> Option<String> {
    // Would load from caCerts config
    None
}

/// Get client certificate
pub fn get_client_cert() -> Option<String> {
    get_mtls_config().and_then(|c| c.cert)
}

/// Get client key
pub fn get_client_key() -> Option<String> {
    get_mtls_config().and_then(|c| c.key)
}

/// Configure mTLS
pub fn configure_mtls() {
    // Would configure global TLS settings
    // For now this is a stub
}

/// Clear mTLS cache
pub fn clear_mtls_cache() {
    // Would clear memoization cache
}
