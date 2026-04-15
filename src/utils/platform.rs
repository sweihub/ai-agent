// Source: /data/home/swei/claudecode/openclaudecode/src/utils/platform.ts
#![allow(dead_code)]

use std::collections::HashMap;

/// Supported platforms
pub const SUPPORTED_PLATFORMS: &[&str] = &["macos", "linux", "windows"];

/// Platform enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    Macos,
    Linux,
    Windows,
    FreeBSD,
    Unknown,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Macos => "macos",
            Platform::Linux => "linux",
            Platform::Windows => "windows",
            Platform::FreeBSD => "freebsd",
            Platform::Unknown => "unknown",
        }
    }
}

impl From<&str> for Platform {
    fn from(s: &str) -> Self {
        match s {
            "macos" => Platform::Macos,
            "linux" => Platform::Linux,
            "windows" => Platform::Windows,
            "freebsd" => Platform::FreeBSD,
            _ => Platform::Unknown,
        }
    }
}

/// Get the current platform as a string
pub fn get_platform() -> &'static str {
    detect_platform()
}

pub fn detect_platform() -> &'static str {
    #[cfg(target_os = "windows")]
    return "windows";
    #[cfg(target_os = "macos")]
    return "macos";
    #[cfg(target_os = "linux")]
    return "linux";
    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    return "freebsd";
    #[cfg(target_os = "android")]
    return "android";
    #[cfg(target_os = "ios")]
    return "ios";
    #[allow(unreachable_code)]
    "unknown"
}

pub fn is_windows() -> bool {
    detect_platform() == "windows"
}
pub fn is_mac() -> bool {
    detect_platform() == "macos"
}
pub fn is_linux() -> bool {
    detect_platform() == "linux"
}

pub fn detect_arch() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    return "x86_64";
    #[cfg(target_arch = "aarch64")]
    return "aarch64";
    #[cfg(target_arch = "arm")]
    return "arm";
    #[cfg(target_arch = "riscv64")]
    return "riscv64";
    #[allow(unreachable_code)]
    "unknown"
}

pub fn is_64bit() -> bool {
    std::mem::size_of::<usize>() == 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform() {
        let p = detect_platform();
        assert!(!p.is_empty());
    }

    #[test]
    fn test_64bit() {
        let _ = is_64bit();
    }
}
