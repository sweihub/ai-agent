// Source: ~/claudecode/openclaudecode/src/utils/model/check1mAccess.ts

/// Check if extra usage is enabled based on the cached disabled reason.
/// Extra usage is considered enabled if there's no disabled reason,
/// or if the disabled reason indicates it's provisioned but temporarily unavailable.
fn is_extra_usage_enabled() -> bool {
    // In production, this would check getGlobalConfig().cachedExtraUsageDisabledReason
    // For now, return a conservative default
    false
}

/// Check if the user has access to Opus 1M context.
pub fn check_opus_1m_access() -> bool {
    if is_1m_context_disabled() {
        return false;
    }

    if is_claude_ai_subscriber() {
        return is_extra_usage_enabled();
    }

    // Non-subscribers (API/PAYG) have access
    true
}

/// Check if the user has access to Sonnet 1M context.
pub fn check_sonnet_1m_access() -> bool {
    if is_1m_context_disabled() {
        return false;
    }

    if is_claude_ai_subscriber() {
        return is_extra_usage_enabled();
    }

    // Non-subscribers (API/PAYG) have access
    true
}

/// Check if 1M context is disabled.
fn is_1m_context_disabled() -> bool {
    // Check environment variable (localized: CLAUDE_CODE_* -> AI_CODE_*)
    std::env::var("AI_CODE_1M_CONTEXT_DISABLED")
        .ok()
        .map(|v| {
            let v = v.to_lowercase();
            v == "1" || v == "true" || v == "yes"
        })
        .unwrap_or(false)
}

/// Check if the user is a Claude AI subscriber.
fn is_claude_ai_subscriber() -> bool {
    // In production, this would check auth state
    // For now, check an environment variable
    std::env::var("AI_IS_CLAUDE_AI_SUBSCRIBER")
        .ok()
        .map(|v| {
            let v = v.to_lowercase();
            v == "1" || v == "true" || v == "yes"
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1m_access_non_subscriber() {
        // Non-subscribers should have access unless 1M context is explicitly disabled
        std::env::remove_var("AI_IS_CLAUDE_AI_SUBSCRIBER");
        std::env::remove_var("AI_CODE_1M_CONTEXT_DISABLED");
        assert!(check_opus_1m_access());
        assert!(check_sonnet_1m_access());
    }
}
