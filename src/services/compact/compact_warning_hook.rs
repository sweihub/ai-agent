//! Compact warning hook - triggered when compaction is needed

use serde::{Deserialize, Serialize};

/// Warning level for compaction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WarningLevel {
    /// No warning - everything is fine
    None,
    /// Info level - user should be aware
    Info,
    /// Warning level - user might want to compact
    Warning,
    /// Critical - immediate compaction recommended
    Critical,
}

/// Compact warning info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactWarningInfo {
    /// Current token count
    pub current_tokens: u64,
    /// Maximum tokens allowed
    pub max_tokens: u64,
    /// Utilization percentage (0-100)
    pub utilization: f64,
    /// Warning level
    pub level: WarningLevel,
    /// Suggested action
    pub suggestion: Option<String>,
}

/// Get warning level based on utilization
pub fn get_warning_level(utilization: f64) -> WarningLevel {
    if utilization >= 0.95 {
        WarningLevel::Critical
    } else if utilization >= 0.85 {
        WarningLevel::Warning
    } else if utilization >= 0.75 {
        WarningLevel::Info
    } else {
        WarningLevel::None
    }
}

/// Create compact warning info
pub fn create_compact_warning_info(current_tokens: u64, max_tokens: u64) -> CompactWarningInfo {
    let utilization = if max_tokens > 0 {
        (current_tokens as f64 / max_tokens as f64) * 100.0
    } else {
        0.0
    };

    let level = get_warning_level(utilization / 100.0);

    let suggestion = match level {
        WarningLevel::Critical => Some("Run /compact now to avoid losing context".to_string()),
        WarningLevel::Warning => Some("Consider running /compact soon".to_string()),
        WarningLevel::Info => Some("You may want to run /compact later".to_string()),
        WarningLevel::None => None,
    };

    CompactWarningInfo {
        current_tokens,
        max_tokens,
        utilization,
        level,
        suggestion,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_warning_level_critical() {
        assert_eq!(get_warning_level(0.96), WarningLevel::Critical);
    }

    #[test]
    fn test_get_warning_level_warning() {
        assert_eq!(get_warning_level(0.86), WarningLevel::Warning);
    }

    #[test]
    fn test_get_warning_level_info() {
        assert_eq!(get_warning_level(0.76), WarningLevel::Info);
    }

    #[test]
    fn test_get_warning_level_none() {
        assert_eq!(get_warning_level(0.5), WarningLevel::None);
    }

    #[test]
    fn test_create_compact_warning_info() {
        let info = create_compact_warning_info(80000, 100000);
        assert_eq!(info.current_tokens, 80000);
        assert_eq!(info.max_tokens, 100000);
        assert!((info.utilization - 80.0).abs() < 0.01);
    }
}
