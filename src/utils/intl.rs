// Source: /data/home/swei/claudecode/openclaudecode/src/utils/intl.ts
//! Shared Intl object instances with lazy initialization.
//!
//! Intl constructors are expensive (~0.05-0.1ms each), so we cache instances
//! for reuse across the codebase instead of creating new ones each time.
//! Lazy initialization ensures we only pay the cost when actually needed.

use crate::constants::env::system;
use std::sync::OnceLock;

// Segmenters for Unicode text processing (lazily initialized)
static GRAPHEME_SEGMENTER: OnceLock<icu_segmenter::GraphemeClusterSegmenter> = OnceLock::new();
static WORD_SEGMENTER: OnceLock<icu_segmenter::WordSegmenter> = OnceLock::new();

// RelativeTimeFormat cache (keyed by style:numeric)
static RTF_CACHE: std::sync::RwLock<std::collections::HashMap<String, once_cell::sync::Lazy<chrono::Locales>>>> = 
    std::sync::RwLock::new(std::collections::HashMap::new());

// Timezone is constant for the process lifetime
static CACHED_TIMEZONE: OnceLock<String> = OnceLock::new();

// System locale language subtag (e.g. 'en', 'ja') is constant for the process
// lifetime. None = not yet computed; Some(None) = computed but unavailable
static CACHED_SYSTEM_LOCALE_LANGUAGE: OnceLock<Option<String>> = OnceLock::new();

/// Get the grapheme segmenter (lazy initialized)
pub fn get_grapheme_segmenter() -> &'static icu_segmenter::GraphemeClusterSegmenter {
    GRAPHEME_SEGMENTER.get_or_init(icu_segmenter::GraphemeClusterSegmenter::new)
}

/// Extract the first grapheme cluster from a string.
/// Returns empty string for empty strings.
pub fn first_grapheme(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    
    let segmenter = get_grapheme_segmenter();
    let graphemes: Vec<&str> = segmenter.segment(text).collect();
    graphemes.first().map(|s| s.to_string()).unwrap_or_default()
}

/// Extract the last grapheme cluster from a string.
/// Returns empty string for empty strings.
pub fn last_grapheme(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    
    let segmenter = get_grapheme_segmenter();
    let graphemes: Vec<&str> = segmenter.segment(text).collect();
    graphemes.last().map(|s| s.to_string()).unwrap_or_default()
}

/// Get the word segmenter (lazy initialized)
pub fn get_word_segmenter() -> &'static icu_segmenter::WordSegmenter {
    WORD_SEGMENTER.get_or_init(icu_segmenter::WordSegmenter::new)
}

/// Get cached timezone
pub fn get_time_zone() -> &'static str {
    CACHED_TIMEZONE.get_or_init(|| {
        // Use local timezone
        match chrono::Local::now().timezone().name().to_str() {
            name if !name.is_empty() => name.to_string(),
            _ => "UTC".to_string(),
        }
    })
}

/// Get system locale language (e.g., 'en', 'ja')
pub fn get_system_locale_language() -> Option<&'static str> {
    CACHED_SYSTEM_LOCALE_LANGUAGE.get_or_init(|| {
        // Try to get locale from environment
        std::env::var(system::LANG)
            .or_else(|_| std::env::var(system::LC_ALL))
            .ok()
            .and_then(|lang| {
                // Parse locale like "en_US.UTF-8"
                lang.split('_').next().map(|s| s.to_string())
            })
    }).as_deref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_grapheme() {
        assert_eq!(first_grapheme("hello"), "h");
        assert_eq!(first_grapheme(""), "");
    }

    #[test]
    fn test_last_grapheme() {
        assert_eq!(last_grapheme("hello"), "o");
        assert_eq!(last_grapheme(""), "");
    }

    #[test]
    fn test_get_time_zone() {
        let tz = get_time_zone();
        assert!(!tz.is_empty());
    }
}