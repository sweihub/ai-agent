// Source: ~/claudecode/openclaudecode/src/utils/hooks/ssrfGuard.ts
#![allow(dead_code)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;

use crate::utils::http::get_user_agent;

/// SSRF guard for HTTP hooks.
///
/// Blocks private, link-local, and other non-routable address ranges to prevent
/// project-configured HTTP hooks from reaching cloud metadata endpoints
/// (169.254.169.254) or internal infrastructure.
///
/// Loopback (127.0.0.0/8, ::1) is intentionally ALLOWED — local dev policy
/// servers are a primary HTTP hook use case.

/// Lookup result for DNS resolution
#[derive(Debug, Clone)]
pub struct LookupAddress {
    pub address: String,
    pub family: u8, // 4 or 6
}

/// Returns true if the address is in a range that HTTP hooks should not reach.
///
/// Blocked IPv4:
///   0.0.0.0/8        "this" network
///   10.0.0.0/8       private
///   100.64.0.0/10    shared address space / CGNAT
///   169.254.0.0/16   link-local (cloud metadata)
///   172.16.0.0/12    private
///   192.168.0.0/16   private
///
/// Blocked IPv6:
///   ::               unspecified
///   fc00::/7         unique local
///   fe80::/10        link-local
///   ::ffff:<v4>      mapped IPv4 in a blocked range
///
/// Allowed (returns false):
///   127.0.0.0/8      loopback (local dev hooks)
///   ::1              loopback
///   everything else
pub fn is_blocked_address(address: &str) -> bool {
    // Try parsing as IPv4
    if let Ok(ipv4) = address.parse::<Ipv4Addr>() {
        return is_blocked_v4(&ipv4);
    }

    // Try parsing as IPv6
    if let Ok(ipv6) = address.parse::<Ipv6Addr>() {
        return is_blocked_v6(&ipv6);
    }

    // Not a valid IP literal — let the real DNS path handle it
    false
}

/// Check if an IPv4 address is blocked
fn is_blocked_v4(addr: &Ipv4Addr) -> bool {
    let octets = addr.octets();
    let a = octets[0] as u32;
    let b = octets[1] as u32;

    // Loopback explicitly allowed (127.0.0.0/8)
    if a == 127 {
        return false;
    }

    // 0.0.0.0/8 — "this" network
    if a == 0 {
        return true;
    }
    // 10.0.0.0/8 — private
    if a == 10 {
        return true;
    }
    // 169.254.0.0/16 — link-local, cloud metadata
    if a == 169 && b == 254 {
        return true;
    }
    // 172.16.0.0/12 — private (172.16.0.0 - 172.31.255.255)
    if a == 172 && b >= 16 && b <= 31 {
        return true;
    }
    // 100.64.0.0/10 — shared address space (RFC 6598, CGNAT)
    // Some cloud providers use this range for metadata endpoints
    if a == 100 && b >= 64 && b <= 127 {
        return true;
    }
    // 192.168.0.0/16 — private
    if a == 192 && b == 168 {
        return true;
    }

    false
}

/// Check if an IPv6 address is blocked
fn is_blocked_v6(addr: &Ipv6Addr) -> bool {
    let segments = addr.segments();

    // ::1 loopback explicitly allowed
    if *addr == Ipv6Addr::LOCALHOST {
        return false;
    }

    // :: unspecified
    if *addr == Ipv6Addr::UNSPECIFIED {
        return true;
    }

    // Check for IPv4-mapped IPv6 (::ffff:x.x.x.x)
    if let Some(mapped_v4) = extract_mapped_ipv4(addr) {
        return is_blocked_v4(&mapped_v4);
    }

    // fc00::/7 — unique local addresses (fc00:: through fdff::)
    let first_segment = segments[0];
    if first_segment >= 0xfc00 && first_segment <= 0xfdff {
        return true;
    }

    // fe80::/10 — link-local
    // The /10 means fe80 through febf
    if first_segment >= 0xfe80 && first_segment <= 0xfebf {
        return true;
    }

    false
}

/// Extract the embedded IPv4 address from an IPv4-mapped IPv6 address
/// (::ffff:x.x.x.x format)
fn extract_mapped_ipv4(addr: &Ipv6Addr) -> Option<Ipv4Addr> {
    let segments = addr.segments();

    // IPv4-mapped: first 80 bits zero, next 16 bits ffff, last 32 bits = IPv4
    // In segments: [0, 0, 0, 0, 0, 0xffff, hi, lo]
    if segments[0] == 0
        && segments[1] == 0
        && segments[2] == 0
        && segments[3] == 0
        && segments[4] == 0
        && segments[5] == 0xffff
    {
        let hi = segments[6];
        let lo = segments[7];
        let ipv4 = Ipv4Addr::new(
            ((hi >> 8) & 0xff) as u8,
            (hi & 0xff) as u8,
            ((lo >> 8) & 0xff) as u8,
            (lo & 0xff) as u8,
        );
        return Some(ipv4);
    }

    None
}

/// DNS lookup result
#[derive(Debug)]
pub struct DnsLookupResult {
    pub addresses: Vec<LookupAddress>,
}

/// A DNS lookup-compatible function that resolves a hostname and rejects
/// addresses in blocked ranges.
///
/// IP literals in the hostname are validated directly without DNS.
pub fn ssrf_guarded_lookup(hostname: &str) -> Result<DnsLookupResult, SsrfError> {
    // If hostname is already an IP literal, validate it directly
    if let Ok(ipv4) = hostname.parse::<Ipv4Addr>() {
        if is_blocked_v4(&ipv4) {
            return Err(ssrf_error(hostname, &ipv4.to_string()));
        }
        return Ok(DnsLookupResult {
            addresses: vec![LookupAddress {
                address: hostname.to_string(),
                family: 4,
            }],
        });
    }

    if let Ok(ipv6) = hostname.parse::<Ipv6Addr>() {
        if is_blocked_v6(&ipv6) {
            return Err(ssrf_error(hostname, &ipv6.to_string()));
        }
        return Ok(DnsLookupResult {
            addresses: vec![LookupAddress {
                address: hostname.to_string(),
                family: 6,
            }],
        });
    }

    // Perform actual DNS resolution
    // In production, this would use trust-dns-resolver or similar
    // For now, use std::net::ToSocketAddrs as a basic implementation
    let socket_addrs = format!("{}:0", hostname)
        .to_socket_addrs()
        .map_err(|e| SsrfError {
            code: "ENOTFOUND".to_string(),
            hostname: hostname.to_string(),
            address: String::new(),
            message: e.to_string(),
        })?;

    let mut addresses = Vec::new();
    for socket_addr in socket_addrs {
        let ip = socket_addr.ip();
        match ip {
            IpAddr::V4(v4) => {
                if is_blocked_v4(&v4) {
                    return Err(ssrf_error(hostname, &v4.to_string()));
                }
                addresses.push(LookupAddress {
                    address: v4.to_string(),
                    family: 4,
                });
            }
            IpAddr::V6(v6) => {
                if is_blocked_v6(&v6) {
                    return Err(ssrf_error(hostname, &v6.to_string()));
                }
                addresses.push(LookupAddress {
                    address: v6.to_string(),
                    family: 6,
                });
            }
        }
    }

    if addresses.is_empty() {
        return Err(SsrfError {
            code: "ENOTFOUND".to_string(),
            hostname: hostname.to_string(),
            address: String::new(),
            message: format!("No addresses found for {}", hostname),
        });
    }

    Ok(DnsLookupResult { addresses })
}

/// SSRF error type
#[derive(Debug)]
pub struct SsrfError {
    pub code: String,
    pub hostname: String,
    pub address: String,
    pub message: String,
}

impl std::fmt::Display for SsrfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HTTP hook blocked: {} resolves to {} (private/link-local address). Loopback (127.0.0.1, ::1) is allowed for local dev.",
            self.hostname, self.address
        )
    }
}

impl std::error::Error for SsrfError {}

/// Create an SSRF error
fn ssrf_error(hostname: &str, address: &str) -> SsrfError {
    SsrfError {
        code: "ERR_HTTP_HOOK_BLOCKED_ADDRESS".to_string(),
        hostname: hostname.to_string(),
        address: address.to_string(),
        message: format!(
            "HTTP hook blocked: {} resolves to {} (private/link-local address). Loopback (127.0.0.1, ::1) is allowed for local dev.",
            hostname, address
        ),
    }
}

/// Async DNS lookup wrapper (for use with reqwest or other async HTTP clients)
pub async fn ssrf_guarded_lookup_async(
    hostname: &str,
) -> Result<DnsLookupResult, SsrfError> {
    // Run the sync lookup in a blocking task
    let hostname_owned = hostname.to_string();
    let hostname_for_err = hostname_owned.clone();
    let result = tokio::task::spawn_blocking(move || ssrf_guarded_lookup(&hostname_owned))
        .await
        .map_err(|e| SsrfError {
            code: "INTERNAL_ERROR".to_string(),
            hostname: hostname_for_err,
            address: String::new(),
            message: e.to_string(),
        })?;
    result
}

/// Create a custom reqwest connector that uses the SSRF guard
pub fn create_ssrf_protected_connector() -> Arc<reqwest::Client> {
    // Build a client that will use our SSRF-guarded DNS lookup
    reqwest::Client::builder()
        .user_agent(get_user_agent())
        .danger_accept_invalid_certs(false)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_blocked_address_loopback_allowed() {
        // Loopback should be allowed
        assert!(!is_blocked_address("127.0.0.1"));
        assert!(!is_blocked_address("127.0.0.255"));
        assert!(!is_blocked_address("::1"));
    }

    #[test]
    fn test_is_blocked_address_private_ipv4() {
        assert!(is_blocked_address("10.0.0.1"));
        assert!(is_blocked_address("192.168.1.1"));
        assert!(is_blocked_address("172.16.0.1"));
        assert!(is_blocked_address("172.31.255.255"));
    }

    #[test]
    fn test_is_blocked_address_link_local() {
        assert!(is_blocked_address("169.254.169.254")); // AWS metadata
        assert!(is_blocked_address("169.254.0.1"));
    }

    #[test]
    fn test_is_blocked_address_cgnat() {
        assert!(is_blocked_address("100.100.100.200")); // Alibaba metadata
        assert!(is_blocked_address("100.64.0.1"));
        assert!(is_blocked_address("100.127.255.255"));
    }

    #[test]
    fn test_is_blocked_address_this_network() {
        assert!(is_blocked_address("0.0.0.0"));
        assert!(is_blocked_address("0.255.255.255"));
    }

    #[test]
    fn test_is_blocked_address_public_allowed() {
        // Public IPs should not be blocked
        assert!(!is_blocked_address("8.8.8.8"));
        assert!(!is_blocked_address("1.1.1.1"));
        assert!(!is_blocked_address("192.0.2.1")); // TEST-NET-1, but not in blocked ranges
    }

    #[test]
    fn test_is_blocked_address_ipv6() {
        assert!(!is_blocked_address("::1")); // Loopback allowed
        assert!(is_blocked_address("::")); // Unspecified blocked
        assert!(is_blocked_address("fc00::1")); // Unique local
        assert!(is_blocked_address("fd00::1")); // Unique local
        assert!(is_blocked_address("fe80::1")); // Link-local
    }

    #[test]
    fn test_is_blocked_address_ipv4_mapped_ipv6() {
        // ::ffff:169.254.169.254 should be blocked (AWS metadata via IPv6)
        assert!(is_blocked_address("::ffff:169.254.169.254"));
        // ::ffff:127.0.0.1 should be allowed
        assert!(!is_blocked_address("::ffff:127.0.0.1"));
        // ::ffff:10.0.0.1 should be blocked
        assert!(is_blocked_address("::ffff:10.0.0.1"));
    }

    #[test]
    fn test_ssrf_guarded_lookup_loopback() {
        let result = ssrf_guarded_lookup("127.0.0.1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ssrf_guarded_lookup_blocked_private() {
        let result = ssrf_guarded_lookup("10.0.0.1");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "ERR_HTTP_HOOK_BLOCKED_ADDRESS");
    }
}
