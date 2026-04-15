use crate::services::mock_rate_limits::{
    get_mock_headerless_429_message, get_mock_headers, should_process_mock_limits,
};

pub fn process_rate_limit_headers(
    _headers: &mut std::collections::HashMap<String, String>,
) -> bool {
    should_process_mock_limits()
}

pub fn should_process_rate_limits(is_subscriber: bool) -> bool {
    is_subscriber || should_process_mock_limits()
}

pub fn check_mock_rate_limit_error(
    current_model: &str,
    _is_fast_mode_active: Option<bool>,
) -> Option<String> {
    if !should_process_mock_limits() {
        return None;
    }

    if let Some(headerless_message) = get_mock_headerless_429_message() {
        return Some(format!("429: {}", headerless_message));
    }

    let mock_headers = match get_mock_headers() {
        Some(h) => h,
        None => return None,
    };

    let status = mock_headers.status;
    let overage_status = mock_headers.overage_status;
    let rate_limit_type = mock_headers.claim.clone();

    let is_opus_limit = matches!(
        rate_limit_type,
        Some(crate::services::mock_rate_limits::MockClaim::SevenDayOpus)
    );
    let is_using_opus = current_model.contains("opus");

    if is_opus_limit && !is_using_opus {
        return None;
    }

    let should_throw_429 = status == Some(crate::services::mock_rate_limits::MockStatus::Rejected)
        && (overage_status.is_none()
            || overage_status == Some(crate::services::mock_rate_limits::MockStatus::Rejected));

    if should_throw_429 {
        Some("429: Rate limit exceeded".to_string())
    } else {
        None
    }
}

pub fn is_mock_rate_limit_error(status: u16) -> bool {
    should_process_mock_limits() && status == 429
}
