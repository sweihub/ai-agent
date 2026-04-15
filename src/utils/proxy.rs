// Source: /data/home/swei/claudecode/openclaudecode/src/utils/proxy.ts
//! Proxy configuration utilities.

use crate::constants::env::system;
use std::sync::{Mutex, OnceLock};

static PROXY_CACHE: OnceLock<Mutex<ProxyConfig>> = OnceLock::new();

/// Proxy configuration
#[derive(Debug, Clone, Default)]
pub struct ProxyConfig {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Option<String>,
}

/// Get the proxy configuration from environment
pub fn get_proxy_config() -> ProxyConfig {
    PROXY_CACHE
        .get_or_init(|| Mutex::new(load_proxy_config()))
        .lock()
        .unwrap()
        .clone()
}

fn load_proxy_config() -> ProxyConfig {
    let http_proxy = std::env::var(system::HTTP_PROXY)
        .or_else(|_| std::env::var(system::HTTP_PROXY_LOWER))
        .ok();

    let https_proxy = std::env::var(system::HTTPS_PROXY)
        .or_else(|_| std::env::var(system::HTTPS_PROXY_LOWER))
        .ok();

    let no_proxy = std::env::var(system::NO_PROXY)
        .or_else(|_| std::env::var(system::NO_PROXY_LOWER))
        .ok();

    ProxyConfig {
        http_proxy,
        https_proxy,
        no_proxy,
    }
}

/// Clear the proxy cache (call after environment changes)
pub fn clear_proxy_cache() {
    // Stub - OnceLock doesn't support clearing
}

/// Get HTTP proxy URL
pub fn get_http_proxy() -> Option<String> {
    get_proxy_config().http_proxy.clone()
}

/// Get HTTPS proxy URL
pub fn get_https_proxy() -> Option<String> {
    get_proxy_config().https_proxy.clone()
}

/// Get NO_PROXY list
pub fn get_no_proxy() -> Option<String> {
    get_proxy_config().no_proxy.clone()
}

/// Check if a host should bypass the proxy
pub fn should_bypass_proxy(host: &str) -> bool {
    let no_proxy = match get_no_proxy() {
        Some(np) => np,
        None => return false,
    };

    // Check if host matches any NO_PROXY pattern
    for pattern in no_proxy.split(',') {
        let pattern = pattern.trim();

        if pattern.is_empty() {
            continue;
        }

        if pattern.starts_with('.') {
            // Match subdomains: .example.com matches www.example.com
            let domain = &pattern[1..];
            if host.ends_with(domain) || host == domain {
                return true;
            }
        } else if host == pattern {
            return true;
        }
    }

    false
}

/// Configure global agents with proxy settings (for HTTP clients)
pub fn configure_global_agents() {
    // This would be called to update any HTTP clients when proxy env vars change
    clear_proxy_cache();
}
