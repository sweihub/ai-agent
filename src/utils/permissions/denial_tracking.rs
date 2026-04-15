// Source: ~/claudecode/openclaudecode/src/utils/permissions/denialTracking.ts
#![allow(dead_code)]

//! Denial tracking infrastructure for permission classifiers.
//! Tracks consecutive denials and total denials to determine
//! when to fall back to prompting.

/// State for tracking permission denials.
#[derive(Debug, Clone, Copy)]
pub struct DenialTrackingState {
    pub consecutive_denials: u32,
    pub total_denials: u32,
}

/// Limits for denial tracking.
pub const DENIAL_LIMITS: DenialLimits = DenialLimits {
    max_consecutive: 3,
    max_total: 20,
};

pub struct DenialLimits {
    pub max_consecutive: u32,
    pub max_total: u32,
}

/// Creates a new denial tracking state with zero counts.
pub fn create_denial_tracking_state() -> DenialTrackingState {
    DenialTrackingState {
        consecutive_denials: 0,
        total_denials: 0,
    }
}

/// Records a denial, incrementing both consecutive and total counts.
pub fn record_denial(state: DenialTrackingState) -> DenialTrackingState {
    DenialTrackingState {
        consecutive_denials: state.consecutive_denials + 1,
        total_denials: state.total_denials + 1,
    }
}

/// Records a success, resetting consecutive denials.
pub fn record_success(state: DenialTrackingState) -> DenialTrackingState {
    if state.consecutive_denials == 0 {
        return state; // No change needed
    }
    DenialTrackingState {
        consecutive_denials: 0,
        total_denials: state.total_denials,
    }
}

/// Checks if we should fallback to prompting based on denial limits.
pub fn should_fallback_to_prompting(state: DenialTrackingState) -> bool {
    state.consecutive_denials >= DENIAL_LIMITS.max_consecutive
        || state.total_denials >= DENIAL_LIMITS.max_total
}
