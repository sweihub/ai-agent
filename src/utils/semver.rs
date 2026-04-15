// Source: /data/home/swei/claudecode/openclaudecode/src/utils/semver.ts
//! Semantic version utilities.

use std::cmp::Ordering;

/// Parse a semantic version string
pub fn parse_semver(version: &str) -> Option<Semver> {
    let version = version.trim();

    // Remove 'v' prefix if present
    let version = version.strip_prefix('v').unwrap_or(version);

    let parts: Vec<&str> = version.split('.').collect();

    if parts.is_empty() {
        return None;
    }

    let major = parts[0].parse().ok()?;
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts
        .get(2)
        .and_then(|s| s.split('-').next()?.parse().ok())
        .unwrap_or(0);

    Some(Semver {
        major,
        minor,
        patch,
    })
}

/// A semantic version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Semver {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Semver {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Compare two semantic versions
    pub fn compare(&self, other: &Semver) -> Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }

    /// Check if this version is compatible with another (same major version)
    pub fn is_compatible(&self, other: &Semver) -> bool {
        self.major == other.major
    }

    /// Check if this version is greater than another
    pub fn gt(&self, other: &Semver) -> bool {
        self.compare(other) == Ordering::Greater
    }

    /// Check if this version is less than another
    pub fn lt(&self, other: &Semver) -> bool {
        self.compare(other) == Ordering::Less
    }

    /// Check if this version is equal to another
    pub fn eq(&self, other: &Semver) -> bool {
        self.compare(other) == Ordering::Equal
    }
}

impl std::fmt::Display for Semver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for Semver {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare(other))
    }
}

impl Ord for Semver {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
}
