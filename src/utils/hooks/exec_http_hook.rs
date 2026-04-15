// Source: ~/claudecode/openclaudecode/src/utils/hooks/execHttpHook.ts
#![allow(dead_code)]

use std::collections::HashSet;
use std::time::Duration;

use reqwest::Client;
use url::Url;

use crate::utils::hooks::ssrf_guard::ssrf_guarded_lookup;

/// Default HTTP hook timeout: 10 minutes
const DEFAULT_HTTP_HOOK_TIMEOUT_MS: u64 = 10 * 60 * 1000;

/// Represents an HTTP hook configuration
pub struct HttpHook {
    /// URL to POST to
    pub url: String,
    /// Optional timeout in seconds
    pub timeout: Option<u64>,
    /// Optional headers to include
    pub headers: Option<std::collections::HashMap<String, String>>,
    /// Allowed env vars for interpolation
    pub allowed_env_vars: Option<Vec<String>>,
}

/// Result of an HTTP hook execution
pub struct HttpHookResult {
    pub ok: bool,
    pub status_code: Option<u16>,
    pub body: String,
    pub error: Option<String>,
    pub aborted: bool,
}

/// HTTP hook policy from settings
struct HttpHookPolicy {
    allowed_urls: Option<Vec<String>>,
    allowed_env_vars: Option<Vec<String>>,
}

/// Get HTTP hook allowlist restrictions from settings
fn get_http_hook_policy() -> HttpHookPolicy {
    // In a real implementation, this would read from merged settings
    HttpHookPolicy {
        allowed_urls: None, // None means no restriction
        allowed_env_vars: None,
    }
}

/// Match a URL against a pattern with * as a wildcard (any characters)
fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    // Escape regex special chars, then replace * with .*
    let escaped = regex::escape(pattern);
    let regex_str = escaped.replace("\\*", ".*");
    match regex::Regex::new(&format!("^{}$", regex_str)) {
        Ok(re) => re.is_match(url),
        Err(_) => false,
    }
}

/// Strip CR, LF, and NUL bytes from a header value to prevent HTTP header injection
fn sanitize_header_value(value: &str) -> String {
    value.replace(|c: char| c == '\r' || c == '\n' || c == '\0', "")
}

/// Interpolate $VAR_NAME and ${VAR_NAME} patterns in a string using process.env,
/// but only for variable names present in the allowlist
fn interpolate_env_vars(value: &str, allowed_env_vars: &HashSet<String>) -> String {
    let re = regex::Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}|\$([A-Z_][A-Z0-9_]*)").unwrap();

    let interpolated = re.replace_all(value, |caps: &regex::Captures| {
        let var_name = caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str());
        if let Some(name) = var_name {
            if !allowed_env_vars.contains(name) {
                log_for_debugging(&format!(
                    "Hooks: env var ${} not in allowedEnvVars, skipping interpolation",
                    name
                ));
                return String::new();
            }
            // Get env var value
            if let Ok(val) = std::env::var(name) {
                return val;
            }
        }
        String::new()
    });

    sanitize_header_value(&interpolated)
}

/// Execute an HTTP hook by POSTing the hook input JSON to the configured URL
pub async fn exec_http_hook(
    hook: &HttpHook,
    _hook_event: &str,
    json_input: &str,
) -> HttpHookResult {
    // Enforce URL allowlist before any I/O
    let policy = get_http_hook_policy();
    if let Some(ref allowed_urls) = policy.allowed_urls {
        let matched = allowed_urls
            .iter()
            .any(|p| url_matches_pattern(&hook.url, p));
        if !matched {
            let msg = format!(
                "HTTP hook blocked: {} does not match any pattern in allowedHttpHookUrls",
                hook.url
            );
            log_for_debugging(&msg);
            return HttpHookResult {
                ok: false,
                status_code: None,
                body: String::new(),
                error: Some(msg),
                aborted: false,
            };
        }
    }

    let timeout_ms = hook.timeout.map_or(DEFAULT_HTTP_HOOK_TIMEOUT_MS, |t| t * 1000);

    // Build headers with env var interpolation in values
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );

    if let Some(ref hook_headers) = hook.headers {
        // Intersect hook's allowed_env_vars with policy allowlist when policy is set
        let hook_vars = hook.allowed_env_vars.clone().unwrap_or_default();
        let effective_vars = if let Some(ref policy_vars) = policy.allowed_env_vars {
            hook_vars
                .into_iter()
                .filter(|v| policy_vars.contains(v))
                .collect::<Vec<_>>()
        } else {
            hook_vars
        };
        let allowed_env_vars: HashSet<String> = effective_vars.into_iter().collect();

        for (name, value) in hook_headers {
            let interpolated = interpolate_env_vars(value, &allowed_env_vars);
            if let Ok(header_value) = reqwest::header::HeaderValue::from_str(&interpolated) {
                if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(name.as_bytes()) {
                    headers.insert(header_name, header_value);
                }
            }
        }
    }

    // Build client with timeout and optional proxy configuration
    let mut client_builder = Client::builder().timeout(Duration::from_millis(timeout_ms));

    // Configure proxy if available (would read from HTTP_PROXY/HTTPS_PROXY env vars)
    if let Ok(proxy_url) = std::env::var("HTTP_PROXY") {
        if let Ok(proxy) = reqwest::Proxy::http(&proxy_url) {
            client_builder = client_builder.proxy(proxy);
        }
    }
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY") {
        if let Ok(proxy) = reqwest::Proxy::https(&proxy_url) {
            client_builder = client_builder.proxy(proxy);
        }
    }

    // Check if env proxy is active
    let env_proxy_active = is_env_proxy_active() && !should_bypass_proxy(&hook.url);

    if env_proxy_active {
        log_for_debugging(&format!(
            "Hooks: HTTP hook POST to {} (via env-var proxy)",
            hook.url
        ));
    } else {
        log_for_debugging(&format!("Hooks: HTTP hook POST to {}", hook.url));
    }

    let client = match client_builder.build() {
        Ok(c) => c,
        Err(e) => {
            return HttpHookResult {
                ok: false,
                status_code: None,
                body: String::new(),
                error: Some(format!("Failed to build HTTP client: {}", e)),
                aborted: false,
            };
        }
    };

    // Make the POST request
    let response = client
        .post(&hook.url)
        .headers(headers)
        .body(json_input.to_string())
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();

            log_for_debugging(&format!(
                "Hooks: HTTP hook response status {}, body length {}",
                status,
                body.len()
            ));

            HttpHookResult {
                ok: status >= 200 && status < 300,
                status_code: Some(status),
                body,
                error: None,
                aborted: false,
            }
        }
        Err(e) => {
            if e.is_timeout() {
                return HttpHookResult {
                    ok: false,
                    status_code: None,
                    body: String::new(),
                    error: None,
                    aborted: true,
                };
            }

            let error_msg = e.to_string();
            log_for_debugging(&format!("Hooks: HTTP hook error: {}", error_msg));
            HttpHookResult {
                ok: false,
                status_code: None,
                body: String::new(),
                error: Some(error_msg),
                aborted: false,
            }
        }
    }
}

/// Check if environment proxy is active (HTTP_PROXY or HTTPS_PROXY set)
fn is_env_proxy_active() -> bool {
    std::env::var("HTTP_PROXY").is_ok() || std::env::var("HTTPS_PROXY").is_ok()
}

/// Check if URL should bypass proxy (respects NO_PROXY)
fn should_bypass_proxy(url: &str) -> bool {
    if let Ok(no_proxy) = std::env::var("NO_PROXY") {
        if let Ok(parsed) = Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                for pattern in no_proxy.split(',') {
                    let pattern = pattern.trim();
                    if pattern.is_empty() {
                        continue;
                    }
                    // Check if host matches pattern (supports wildcard prefixes)
                    if pattern.starts_with('.') && host.ends_with(pattern) {
                        return true;
                    }
                    if host == pattern {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Get sandbox proxy config (simplified)
async fn get_sandbox_proxy_config() -> Option<ProxyConfig> {
    // In the TS version, this dynamically imports SandboxManager
    // and checks if sandboxing is enabled
    None
}

struct ProxyConfig {
    host: String,
    port: u16,
    protocol: String,
}

/// Log for debugging
fn log_for_debugging(msg: &str) {
    log::debug!("{}", msg);
}
