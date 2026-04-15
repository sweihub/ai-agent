use crate::constants::env::ai;
use serde::{Deserialize, Serialize};

pub const SESSION_TOKEN_PATH: &str = "/run/ccr/session_token";
const SYSTEM_CA_BUNDLE: &str = "/etc/ssl/certs/ca-certificates.crt";

const NO_PROXY_LIST: &[&str] = &[
    "localhost",
    "127.0.0.1",
    "::1",
    "169.254.0.0/16",
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
];

#[derive(Debug, Clone)]
pub struct UpstreamProxyConfig {
    pub enabled: bool,
    pub session_token: Option<String>,
    pub ca_cert_path: Option<String>,
    pub relay_port: u16,
}

impl Default for UpstreamProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            session_token: None,
            ca_cert_path: None,
            relay_port: 3129,
        }
    }
}

pub fn is_upstream_proxy_enabled() -> bool {
    std::env::var(ai::UPSTREAM_PROXY).is_ok()
}

pub fn get_session_token_path() -> String {
    SESSION_TOKEN_PATH.to_string()
}

pub fn get_system_ca_bundle() -> String {
    SYSTEM_CA_BUNDLE.to_string()
}

pub fn should_bypass_proxy(host: &str) -> bool {
    NO_PROXY_LIST.iter().any(|pattern| {
        if pattern.contains('/') {
            if let Ok(ipnet) = pattern.parse::<std::net::IpNet>() {
                if let Ok(addr) = host.parse::<std::net::IpAddr>() {
                    return ipnet.contains(&addr);
                }
            }
            false
        } else {
            host == *pattern || host.ends_with(&format!(".{}", pattern))
        }
    })
}
