/**
 * Example 25: Auto-Compact Context API Demo
 *
 * Demonstrates the auto-compact context management API:
 * - Query effective context window size
 * - Get auto-compact threshold
 * - Calculate token warning states
 * - Check if auto-compact is enabled
 * - Monitor compaction tracking state
 *
 * Run: cargo run --example 25_auto_compact
 *
 * Environment variables from .env:
 * - AI_BASE_URL: LLM server URL
 * - AI_AUTH_TOKEN: API authentication token
 * - AI_MODEL: Model name
 *
 * Optional overrides:
 * - AI_CODE_AUTO_COMPACT_WINDOW: Override context window size
 * - CLAUDE_AUTOCOMPACT_PCT_OVERRIDE: Override threshold as percentage (0-100)
 * - DISABLE_COMPACT: Disable all compaction
 * - DISABLE_AUTO_COMPACT: Disable auto-compact only
 */
use ai_agent::{
    AutoCompactTrackingState, calculate_token_warning_state, get_auto_compact_threshold,
    get_effective_context_window_size, is_auto_compact_enabled,
};

fn main() {
    println!("=== Example 25: Auto-Compact Context API Demo ===\n");

    // Default model for demo
    let model = "claude-sonnet-4-6";

    // ============================================
    // 1. Query effective context window size
    // ============================================
    println!("--- 1. Effective Context Window ---");
    let effective_window = get_effective_context_window_size(model);
    println!("Model: {}", model);
    println!("Effective context window: {} tokens", effective_window);
    println!("(This accounts for reserved output tokens for summary generation)\n");

    // ============================================
    // 2. Get auto-compact threshold
    // ============================================
    println!("--- 2. Auto-Compact Threshold ---");
    let threshold = get_auto_compact_threshold(model);
    println!("Auto-compact threshold: {} tokens", threshold);
    println!(
        "Tokens buffer for compaction: {} tokens",
        threshold.saturating_sub(effective_window)
    );
    println!("\nTip: Set AI_CODE_AUTO_COMPACT_WINDOW=100000 to test with smaller window\n");

    // ============================================
    // 3. Check if auto-compact is enabled
    // ============================================
    println!("--- 3. Auto-Compact Status ---");
    let enabled = is_auto_compact_enabled();
    println!(
        "Auto-compact enabled: {}",
        if enabled { "Yes" } else { "No" }
    );
    println!("\nDisable with: DISABLE_COMPACT=1 or DISABLE_AUTO_COMPACT=1\n");

    // ============================================
    // 4. Calculate token warning states
    // ============================================
    println!("--- 4. Token Warning States ---");

    // Simulate different token usage levels
    let test_usages: Vec<u32> = vec![
        50_000,                                 // Low usage
        100_000,                                // Medium usage
        threshold.saturating_sub(20_000),       // Near warning
        threshold.saturating_sub(1),            // At auto-compact threshold
        effective_window.saturating_sub(3_000), // At blocking limit
    ];

    for usage in test_usages {
        let state = calculate_token_warning_state(usage, model);
        println!("\nToken usage: {} tokens", usage);
        println!("  - Percent left: {:.1}%", state.percent_left);
        println!(
            "  - Warning threshold: {}",
            if state.is_above_warning_threshold {
                "YES"
            } else {
                "No"
            }
        );
        println!(
            "  - Error threshold: {}",
            if state.is_above_error_threshold {
                "YES"
            } else {
                "No"
            }
        );
        println!(
            "  - Auto-compact trigger: {}",
            if state.is_above_auto_compact_threshold {
                "YES"
            } else {
                "No"
            }
        );
        println!(
            "  - Blocking limit: {}",
            if state.is_at_blocking_limit {
                "YES"
            } else {
                "No"
            }
        );
    }

    // ============================================
    // 5. Auto-compact tracking state
    // ============================================
    println!("\n--- 5. Auto-Compact Tracking State ---");

    // Create initial tracking state
    let mut tracking = AutoCompactTrackingState::new();
    println!("Initial state:");
    println!("  - Compacted: {}", tracking.compacted);
    println!("  - Turn counter: {}", tracking.turn_counter);
    println!("  - Turn ID: {}", tracking.turn_id);
    println!(
        "  - Consecutive failures: {}",
        tracking.consecutive_failures
    );

    // Simulate turns passing
    for i in 1..=3 {
        tracking.turn_counter = i;
        tracking.turn_id = uuid::Uuid::new_v4().to_string();
        println!("\nAfter turn {}: turn_id = {}", i, &tracking.turn_id[..8]);
    }

    // Simulate a compaction
    tracking.compacted = true;
    println!("\nAfter compaction: compacted = {}", tracking.compacted);

    // Simulate failure
    tracking.consecutive_failures = 1;
    println!(
        "After failure: consecutive_failures = {}",
        tracking.consecutive_failures
    );

    // Reset on success
    tracking.compacted = false;
    tracking.consecutive_failures = 0;
    println!(
        "After success reset: compacted = {}, failures = {}\n",
        tracking.compacted, tracking.consecutive_failures
    );

    // ============================================
    // 6. Environment variable overrides
    // ============================================
    println!("--- 6. Environment Overrides ---");
    println!("AI_CODE_AUTO_COMPACT_WINDOW - Override context window size");
    println!("CLAUDE_AUTOCOMPACT_PCT_OVERRIDE - Override threshold (0-100%)");
    println!("DISABLE_COMPACT - Disable all compaction");
    println!("DISABLE_AUTO_COMPACT - Disable auto-compact only");
    println!("AI_CODE_BLOCKING_LIMIT_OVERRIDE - Override blocking limit");

    println!("\n=== done ===");
}
