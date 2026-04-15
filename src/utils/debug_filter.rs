//! Debug filter utilities
//!
//! Translated from openclaudecode/src/utils/debugFilter.ts

use once_cell::sync::Lazy;
use regex::Regex;

/// Debug filter configuration
#[derive(Debug, Clone)]
pub struct DebugFilter {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub is_exclusive: bool,
}

/// Parse debug filter string into a filter configuration
/// Examples:
/// - "api,hooks" -> include only api and hooks categories
/// - "!1p,!file" -> exclude logging and file categories
/// - None/empty -> no filtering (show all)
pub fn parse_debug_filter(filter_string: Option<&str>) -> Option<DebugFilter> {
    let filter_string = filter_string?.trim();
    if filter_string.is_empty() {
        return None;
    }

    let filters: Vec<&str> = filter_string
        .split(',')
        .map(|f| f.trim())
        .filter(|f| !f.is_empty())
        .collect();

    // If no valid filters remain, return None
    if filters.is_empty() {
        return None;
    }

    // Check for mixed inclusive/exclusive filters
    let has_exclusive: bool = filters.iter().any(|f| f.starts_with('!'));
    let has_inclusive: bool = filters.iter().any(|f| !f.starts_with('!'));

    if has_exclusive && has_inclusive {
        // Mixed filters - show all messages
        return None;
    }

    // Clean up filters (remove ! prefix) and normalize
    let clean_filters: Vec<String> = filters
        .iter()
        .map(|f| f.trim_start_matches('!').to_lowercase())
        .collect();

    Some(DebugFilter {
        include: if has_exclusive {
            vec![]
        } else {
            clean_filters.clone()
        },
        exclude: if has_exclusive { clean_filters } else { vec![] },
        is_exclusive: has_exclusive,
    })
}

/// Extract debug categories from a message
/// Supports multiple patterns:
/// - "category: message" -> ["category"]
/// - "[CATEGORY] message" -> ["category"]
/// - "MCP server \"name\": message" -> ["mcp", "name"]
/// - "[ANT-ONLY] 1P event: tengu_timer" -> ["ant-only", "1p"]
///
/// Returns lowercase categories for case-insensitive matching
pub fn extract_debug_categories(message: &str) -> Vec<String> {
    let mut categories: Vec<String> = Vec::new();

    // Pattern 3: MCP server "servername" - Check this first to avoid false positives
    static MCP_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"^MCP server ["']([^"']+)["']"#).unwrap());
    if let Some(mcp_match) = MCP_REGEX.captures(message) {
        if let Some(mcp_name) = mcp_match.get(1) {
            categories.push("mcp".to_string());
            categories.push(mcp_name.as_str().to_lowercase());
        }
    } else {
        // Pattern 1: "category: message" (simple prefix) - only if not MCP pattern
        static PREFIX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([^:[]+):").unwrap());
        if let Some(prefix_match) = PREFIX_REGEX.captures(message) {
            if let Some(prefix) = prefix_match.get(1) {
                categories.push(prefix.as_str().trim().to_lowercase());
            }
        }
    }

    // Pattern 2: [CATEGORY] at the start
    static BRACKET_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\[([^\]]+)]").unwrap());
    if let Some(bracket_match) = BRACKET_REGEX.captures(message) {
        if let Some(bracket) = bracket_match.get(1) {
            categories.push(bracket.as_str().trim().to_lowercase());
        }
    }

    // Pattern 4: Check for additional categories in the message
    if message.to_lowercase().contains("1p event:") {
        categories.push("1p".to_string());
    }

    // Pattern 5: Look for secondary categories after the first pattern
    static SECONDARY_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r":\s*([^:]+?)(?:\s+(?:type|mode|status|event))?:").unwrap());
    if let Some(secondary_match) = SECONDARY_REGEX.captures(message) {
        if let Some(secondary) = secondary_match.get(1) {
            let secondary = secondary.as_str().trim().to_lowercase();
            // Only add if it's a reasonable category name (not too long, no spaces)
            if secondary.len() < 30 && !secondary.contains(' ') {
                categories.push(secondary);
            }
        }
    }

    // Remove duplicates
    categories.sort();
    categories.dedup();
    categories
}

/// Check if debug message should be shown based on filter
pub fn should_show_debug_categories(categories: &[String], filter: &Option<DebugFilter>) -> bool {
    // No filter means show everything
    let filter = match filter {
        Some(f) => f,
        None => return true,
    };

    // If no categories found, handle based on filter mode
    if categories.is_empty() {
        // In exclusive mode, uncategorized messages are excluded by default for security
        // In inclusive mode, uncategorized messages are excluded (must match a category)
        return false;
    }

    if filter.is_exclusive {
        // Exclusive mode: show if none of the categories are in the exclude list
        !categories.iter().any(|cat| filter.exclude.contains(cat))
    } else {
        // Inclusive mode: show if any of the categories are in the include list
        categories.iter().any(|cat| filter.include.contains(cat))
    }
}

/// Main function to check if a debug message should be shown
/// Combines extraction and filtering
pub fn should_show_debug_message(message: &str, filter: &Option<DebugFilter>) -> bool {
    // Fast path: no filter means show everything
    if filter.is_none() {
        return true;
    }

    // Only extract categories if we have a filter
    let categories = extract_debug_categories(message);
    should_show_debug_categories(&categories, filter)
}
