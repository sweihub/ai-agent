use std::env;

pub fn get_env(key: &str) -> Option<String> {
    env::var(key).ok()
}

pub fn get_env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn get_env_bool(key: &str) -> bool {
    env::var(key)
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

pub fn get_env_int(key: &str) -> Option<i64> {
    env::var(key).ok().and_then(|v| v.parse().ok())
}

pub fn set_env(key: &str, value: &str) {
    env::set_var(key, value);
}

pub fn remove_env(key: &str) {
    env::remove_var(key);
}

pub fn is_env_truthy(key: &str) -> bool {
    env::var(key)
        .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
        .unwrap_or(false)
}

/// Check if an env var is explicitly set to a falsy value (0, false, no, off)
/// Returns false if the var is undefined
pub fn is_env_defined_falsy(key: &str) -> bool {
    env::var(key)
        .ok()
        .map(|v| {
            let normalized = v.to_lowercase();
            normalized == "0" || normalized == "false" || normalized == "no" || normalized == "off"
        })
        .unwrap_or(false)
}

/// Get USER_TYPE environment variable
pub fn get_user_type() -> Option<String> {
    env::var("USER_TYPE").ok()
}

/// Check if running in ant-internal build
pub fn is_ant_user() -> bool {
    get_user_type().as_deref() == Some("ant")
}

/// Check if running in test mode
pub fn is_test_mode() -> bool {
    env::var("NODE_ENV")
        .map(|v| v == "test")
        .unwrap_or(false)
}

pub fn get_platform() -> String {
    env::consts::OS.to_string()
}

pub fn is_windows() -> bool {
    cfg!(windows)
}

pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}
