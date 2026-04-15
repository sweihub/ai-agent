//! Double press detection utility
//!
//! Translates useDoublePress.ts - detects double press patterns within a timeout window

/// Default timeout for double press detection in milliseconds
pub const DOUBLE_PRESS_TIMEOUT_MS: u64 = 800;

/// State for tracking double press
pub struct DoublePressState {
    last_press_time: u64,
    timeout_handle: Option<u64>,
}

impl DoublePressState {
    /// Create a new double press state
    pub fn new() -> Self {
        Self {
            last_press_time: 0,
            timeout_handle: None,
        }
    }
}

impl Default for DoublePressState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a press action
#[derive(Debug, Clone, PartialEq)]
pub enum PressResult {
    /// First press detected
    FirstPress,
    /// Double press detected
    DoublePress,
}

/// Handle a press event and determine if it's a double press
///
/// # Arguments
/// * `state` - The double press state to update
/// * `timeout_ms` - Maximum time between presses to count as double press
/// * `on_first_press` - Callback for first press
/// * `on_double_press` - Callback for double press
///
/// # Returns
/// The result of the press action
pub fn handle_double_press<F1, F2>(
    state: &mut DoublePressState,
    timeout_ms: u64,
    on_first_press: F1,
    on_double_press: F2,
) -> PressResult
where
    F1: FnOnce(),
    F2: FnOnce(),
{
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let time_since_last_press = now.saturating_sub(state.last_press_time);
    let is_double_press = time_since_last_press <= timeout_ms && state.timeout_handle.is_some();

    if is_double_press {
        // Double press detected
        state.timeout_handle = None;
        on_double_press();
        PressResult::DoublePress
    } else {
        // First press
        on_first_press();
        state.last_press_time = now;
        state.timeout_handle = Some(now);
        PressResult::FirstPress
    }
}

/// Clear the double press state
pub fn clear_double_press(state: &mut DoublePressState) {
    state.last_press_time = 0;
    state.timeout_handle = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_press_state_new() {
        let state = DoublePressState::new();
        assert_eq!(state.last_press_time, 0);
        assert!(state.timeout_handle.is_none());
    }

    #[test]
    fn test_first_press() {
        let mut state = DoublePressState::new();
        let mut first_called = false;
        let mut double_called = false;

        let result = handle_double_press(
            &mut state,
            DOUBLE_PRESS_TIMEOUT_MS,
            || first_called = true,
            || double_called = true,
        );

        assert_eq!(result, PressResult::FirstPress);
        assert!(first_called);
        assert!(!double_called);
    }
}
