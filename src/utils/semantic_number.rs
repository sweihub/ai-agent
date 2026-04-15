//! Semantic number utilities for parsing numbers from strings.

use std::num::ParseFloatError;
use std::num::ParseIntError;

/// Parse a number from a string that might have suffixes like K, M, B, etc.
pub fn parse_semantic_number(s: &str) -> Result<f64, ParseFloatError> {
    let s = s.trim().to_uppercase();

    let multiplier: f64;
    let number_part: &str;

    if s.ends_with("K") {
        multiplier = 1_000.0;
        number_part = &s[..s.len() - 1];
    } else if s.ends_with("M") {
        multiplier = 1_000_000.0;
        number_part = &s[..s.len() - 1];
    } else if s.ends_with("B") {
        multiplier = 1_000_000_000.0;
        number_part = &s[..s.len() - 1];
    } else if s.ends_with("T") {
        multiplier = 1_000_000_000_000.0;
        number_part = &s[..s.len() - 1];
    } else {
        multiplier = 1.0;
        number_part = &s;
    }

    number_part.parse::<f64>().map(|n| n * multiplier)
}

/// Parse a byte size string (e.g., "10KB", "5MB", "1GB")
pub fn parse_byte_size(s: &str) -> Result<u64, ParseIntError> {
    let s = s.trim().to_uppercase();

    let multiplier: u64;
    let number_part: &str;

    if s.ends_with("KB") || s.ends_with("K") {
        multiplier = 1024;
        number_part = &s[..s.len() - if s.ends_with("KB") { 2 } else { 1 }];
    } else if s.ends_with("MB") || s.ends_with("M") {
        multiplier = 1024 * 1024;
        number_part = &s[..s.len() - if s.ends_with("MB") { 2 } else { 1 }];
    } else if s.ends_with("GB") || s.ends_with("G") {
        multiplier = 1024 * 1024 * 1024;
        number_part = &s[..s.len() - if s.ends_with("GB") { 2 } else { 1 }];
    } else if s.ends_with("TB") || s.ends_with("T") {
        multiplier = 1024 * 1024 * 1024 * 1024;
        number_part = &s[..s.len() - if s.ends_with("TB") { 2 } else { 1 }];
    } else if s.ends_with("B") {
        multiplier = 1;
        number_part = &s[..s.len() - 1];
    } else {
        multiplier = 1;
        number_part = &s;
    }

    number_part.parse::<u64>().map(|n| n * multiplier)
}

/// Format a number with suffixes (K, M, B, T)
pub fn format_with_suffix(n: f64) -> String {
    if n >= 1_000_000_000_000.0 {
        format!("{:.1}T", n / 1_000_000_000_000.0)
    } else if n >= 1_000_000_000.0 {
        format!("{:.1}B", n / 1_000_000_000.0)
    } else if n >= 1_000_000.0 {
        format!("{:.1}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.1}K", n / 1_000.0)
    } else {
        format!("{:.0}", n)
    }
}
