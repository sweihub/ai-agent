// Source: /data/home/swei/claudecode/openclaudecode/src/ink/cursor.ts
//! Cursor utilities for text editing
//!
//! Translated from openclaudecode/src/utils/Cursor.ts

use once_cell::sync::Lazy;
use regex::Regex;

/// Kill ring for storing killed (cut) text that can be yanked (pasted) with Ctrl+Y.
/// This is global state that shares one kill ring across all input fields.
const KILL_RING_MAX_SIZE: usize = 10;

static KILL_RING: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
static KILL_RING_INDEX: std::sync::Mutex<usize> = std::sync::Mutex::new(0);
static LAST_ACTION_WAS_KILL: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

// Track yank state for yank-pop (alt-y)
static LAST_YANK_START: std::sync::Mutex<usize> = std::sync::Mutex::new(0);
static LAST_YANK_LENGTH: std::sync::Mutex<usize> = std::sync::Mutex::new(0);
static LAST_ACTION_WAS_YANK: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

/// Push text to the kill ring
pub fn push_to_kill_ring(text: &str, direction: &str) {
    if text.is_empty() {
        return;
    }

    let mut kill_ring = KILL_RING.lock().unwrap();
    let mut last_action_kill = LAST_ACTION_WAS_KILL.lock().unwrap();

    if *last_action_kill && !kill_ring.is_empty() {
        // Accumulate with the most recent kill
        if direction == "prepend" {
            kill_ring[0] = format!("{}{}", text, kill_ring[0]);
        } else {
            kill_ring[0] = format!("{}{}", kill_ring[0], text);
        }
    } else {
        // Add new entry to front of ring
        kill_ring.insert(0, text.to_string());
        if kill_ring.len() > KILL_RING_MAX_SIZE {
            kill_ring.pop();
        }
    }
    *last_action_kill = true;

    // Reset yank state when killing new text
    let mut last_action_yank = LAST_ACTION_WAS_YANK.lock().unwrap();
    *last_action_yank = false;
}

/// Get the last kill from the ring
pub fn get_last_kill() -> String {
    let kill_ring = KILL_RING.lock().unwrap();
    kill_ring.first().cloned().unwrap_or_default()
}

/// Get an item from the kill ring by index
pub fn get_kill_ring_item(index: usize) -> String {
    let kill_ring = KILL_RING.lock().unwrap();
    if kill_ring.is_empty() {
        return String::new();
    }
    let normalized_index = ((index % kill_ring.len()) + kill_ring.len()) % kill_ring.len();
    kill_ring.get(normalized_index).cloned().unwrap_or_default()
}

/// Get the size of the kill ring
pub fn get_kill_ring_size() -> usize {
    let kill_ring = KILL_RING.lock().unwrap();
    kill_ring.len()
}

/// Clear the kill ring
pub fn clear_kill_ring() {
    let mut kill_ring = KILL_RING.lock().unwrap();
    let mut kill_ring_index = KILL_RING_INDEX.lock().unwrap();
    let mut last_action_kill = LAST_ACTION_WAS_KILL.lock().unwrap();
    let mut last_action_yank = LAST_ACTION_WAS_YANK.lock().unwrap();
    let mut last_yank_start = LAST_YANK_START.lock().unwrap();
    let mut last_yank_length = LAST_YANK_LENGTH.lock().unwrap();

    kill_ring.clear();
    *kill_ring_index = 0;
    *last_action_kill = false;
    *last_action_yank = false;
    *last_yank_start = 0;
    *last_yank_length = 0;
}

/// Reset kill accumulation state
pub fn reset_kill_accumulation() {
    let mut last_action_kill = LAST_ACTION_WAS_KILL.lock().unwrap();
    *last_action_kill = false;
}

/// Record a yank for yank-pop functionality
pub fn record_yank(start: usize, length: usize) {
    let mut last_yank_start = LAST_YANK_START.lock().unwrap();
    let mut last_yank_length = LAST_YANK_LENGTH.lock().unwrap();
    let mut last_action_yank = LAST_ACTION_WAS_YANK.lock().unwrap();
    let mut kill_ring_index = KILL_RING_INDEX.lock().unwrap();

    *last_yank_start = start;
    *last_yank_length = length;
    *last_action_yank = true;
    *kill_ring_index = 0;
}

/// Check if yank-pop is possible
pub fn can_yank_pop() -> bool {
    let last_action_yank = LAST_ACTION_WAS_YANK.lock().unwrap();
    let kill_ring = KILL_RING.lock().unwrap();
    *last_action_yank && kill_ring.len() > 1
}

/// Perform yank-pop operation
pub fn yank_pop() -> Option<YankPopResult> {
    let last_action_yank = LAST_ACTION_WAS_YANK.lock().unwrap();
    let kill_ring = KILL_RING.lock().unwrap();

    if !*last_action_yank || kill_ring.len() <= 1 {
        return None;
    }
    drop(last_action_yank);
    drop(kill_ring);

    let mut kill_ring_index = KILL_RING_INDEX.lock().unwrap();
    let last_yank_start = LAST_YANK_START.lock().unwrap();
    let last_yank_length = LAST_YANK_LENGTH.lock().unwrap();

    // Cycle to next item in kill ring
    let kill_ring = KILL_RING.lock().unwrap();
    *kill_ring_index = (*kill_ring_index + 1) % kill_ring.len();
    let text = kill_ring.get(*kill_ring_index).cloned().unwrap_or_default();

    Some(YankPopResult {
        text,
        start: *last_yank_start,
        length: *last_yank_length,
    })
}

/// Update the yank length
pub fn update_yank_length(length: usize) {
    let mut last_yank_length = LAST_YANK_LENGTH.lock().unwrap();
    *last_yank_length = length;
}

/// Reset yank state
pub fn reset_yank_state() {
    let mut last_action_yank = LAST_ACTION_WAS_YANK.lock().unwrap();
    *last_action_yank = false;
}

/// Result from yank-pop operation
#[derive(Debug, Clone)]
pub struct YankPopResult {
    pub text: String,
    pub start: usize,
    pub length: usize,
}

// Pre-compiled regex patterns for Vim word detection
pub static VIM_WORD_CHAR_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\p{L}\p{N}\p{M}_]$").unwrap());
pub static WHITESPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s").unwrap());

/// Check if character is a Vim word character (letter, digit, underscore)
pub fn is_vim_word_char(ch: &str) -> bool {
    VIM_WORD_CHAR_REGEX.is_match(ch)
}

/// Check if character is whitespace
pub fn is_vim_whitespace(ch: &str) -> bool {
    WHITESPACE_REGEX.is_match(ch)
}

/// Check if character is Vim punctuation
pub fn is_vim_punctuation(ch: &str) -> bool {
    !ch.is_empty() && !is_vim_whitespace(ch) && !is_vim_word_char(ch)
}
