use crate::constants::env::ai;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct MockHeaders {
    pub status: Option<MockStatus>,
    pub reset: Option<String>,
    pub claim: Option<MockClaim>,
    pub overage_status: Option<MockStatus>,
    pub overage_reset: Option<String>,
    pub overage_disabled_reason: Option<String>,
    pub fallback: Option<String>,
    pub fallback_percentage: Option<String>,
    pub retry_after: Option<String>,
    pub five_h_utilization: Option<String>,
    pub five_h_reset: Option<String>,
    pub five_h_surpassed_threshold: Option<String>,
    pub seven_d_utilization: Option<String>,
    pub seven_d_reset: Option<String>,
    pub seven_d_surpassed_threshold: Option<String>,
    pub overage_utilization: Option<String>,
    pub overage_surpassed_threshold: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MockStatus {
    Allowed,
    AllowedWarning,
    Rejected,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MockClaim {
    FiveHour,
    SevenDay,
    SevenDayOpus,
    SevenDaySonnet,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MockScenario {
    Normal,
    SessionLimitReached,
    ApproachingWeeklyLimit,
    WeeklyLimitReached,
    OverageActive,
    OverageWarning,
    OverageExhausted,
    OutOfCredits,
    OrgZeroCreditLimit,
    OrgSpendCapHit,
    MemberZeroCreditLimit,
    SeatTierZeroCreditLimit,
    OpusLimit,
    OpusWarning,
    SonnetLimit,
    SonnetWarning,
    FastModeLimit,
    FastModeShortLimit,
    ExtraUsageRequired,
    Clear,
}

pub type MockHeaderKey = &'static str;

static MOCK_HEADERS: Lazy<Mutex<MockHeaders>> = Lazy::new(|| Mutex::new(MockHeaders::default()));
static MOCK_ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static MOCK_HEADERLESS_429_MESSAGE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

const DEFAULT_MOCK_SUBSCRIPTION: &str = "max";

#[derive(Debug, Clone)]
struct ExceededLimit {
    limit_type: MockClaim,
    resets_at: i64,
}

static EXCEEDED_LIMITS: Lazy<Mutex<Vec<ExceededLimit>>> = Lazy::new(|| Mutex::new(Vec::new()));

impl Default for MockHeaders {
    fn default() -> Self {
        Self {
            status: None,
            reset: None,
            claim: None,
            overage_status: None,
            overage_reset: None,
            overage_disabled_reason: None,
            fallback: None,
            fallback_percentage: None,
            retry_after: None,
            five_h_utilization: None,
            five_h_reset: None,
            five_h_surpassed_threshold: None,
            seven_d_utilization: None,
            seven_d_reset: None,
            seven_d_surpassed_threshold: None,
            overage_utilization: None,
            overage_surpassed_threshold: None,
        }
    }
}

fn is_ant_user() -> bool {
    std::env::var(ai::USER_TYPE).ok() == Some("ant".to_string())
}

pub fn set_mock_header(key: MockHeaderKey, value: Option<&str>) {
    if !is_ant_user() {
        return;
    }

    let mut enabled = MOCK_ENABLED.lock().unwrap();
    *enabled = true;
    drop(enabled);

    let mut headers = MOCK_HEADERS.lock().unwrap();

    match key {
        "status" => {
            headers.status = value.and_then(|v| match v {
                "allowed" => Some(MockStatus::Allowed),
                "allowed_warning" => Some(MockStatus::AllowedWarning),
                "rejected" => Some(MockStatus::Rejected),
                _ => None,
            });
            update_retry_after(&mut headers);
        }
        "reset" => {
            if let Some(v) = value {
                let hours: f64 = v.parse().unwrap_or(0.0);
                if !hours.is_nan() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    headers.reset = Some((now + (hours * 3600.0) as u64).to_string());
                } else {
                    headers.reset = Some(v.to_string());
                }
            } else {
                headers.reset = None;
            }
            update_retry_after(&mut headers);
        }
        "claim" => {
            headers.claim = value.and_then(|v| match v {
                "five_hour" => Some(MockClaim::FiveHour),
                "seven_day" => Some(MockClaim::SevenDay),
                "seven_day_opus" => Some(MockClaim::SevenDayOpus),
                "seven_day_sonnet" => Some(MockClaim::SevenDaySonnet),
                _ => None,
            });
            if headers.claim.is_none() {
                let mut limits = EXCEEDED_LIMITS.lock().unwrap();
                limits.clear();
            }
        }
        "overage-status" => {
            headers.overage_status = value.and_then(|v| match v {
                "allowed" => Some(MockStatus::Allowed),
                "allowed_warning" => Some(MockStatus::AllowedWarning),
                "rejected" => Some(MockStatus::Rejected),
                _ => None,
            });
            update_retry_after(&mut headers);
        }
        "overage-reset" => {
            headers.overage_reset = value.map(|v| v.to_string());
        }
        "overage-disabled-reason" => {
            headers.overage_disabled_reason = value.map(|v| v.to_string());
        }
        "fallback" => {
            headers.fallback = value.map(|v| v.to_string());
        }
        "fallback-percentage" => {
            headers.fallback_percentage = value.map(|v| v.to_string());
        }
        "retry-after" => {
            headers.retry_after = value.map(|v| v.to_string());
        }
        "5h-utilization" => {
            headers.five_h_utilization = value.map(|v| v.to_string());
        }
        "5h-reset" => {
            headers.five_h_reset = value.map(|v| v.to_string());
        }
        "5h-surpassed-threshold" => {
            headers.five_h_surpassed_threshold = value.map(|v| v.to_string());
        }
        "7d-utilization" => {
            headers.seven_d_utilization = value.map(|v| v.to_string());
        }
        "7d-reset" => {
            headers.seven_d_reset = value.map(|v| v.to_string());
        }
        "7d-surpassed-threshold" => {
            headers.seven_d_surpassed_threshold = value.map(|v| v.to_string());
        }
        _ => {}
    }

    let headers_ref = headers.clone();
    drop(headers);

    if headers_ref.status.is_none()
        && headers_ref.reset.is_none()
        && headers_ref.claim.is_none()
        && headers_ref.overage_status.is_none()
    {
        let mut enabled = MOCK_ENABLED.lock().unwrap();
        *enabled = false;
    }
}

fn update_retry_after(headers: &mut MockHeaders) {
    if headers.status == Some(MockStatus::Rejected)
        && (headers.overage_status.is_none()
            || headers.overage_status == Some(MockStatus::Rejected))
    {
        if let Some(ref reset) = headers.reset {
            if let Ok(reset_ts) = reset.parse::<i64>() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                let seconds_until_reset = std::cmp::max(0, reset_ts - now);
                headers.retry_after = Some(seconds_until_reset.to_string());
                return;
            }
        }
    }
    headers.retry_after = None;
}

pub fn add_exceeded_limit(limit_type: MockClaim, hours_from_now: f64) {
    if !is_ant_user() {
        return;
    }

    let mut enabled = MOCK_ENABLED.lock().unwrap();
    *enabled = true;
    drop(enabled);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let resets_at = now + (hours_from_now * 3600.0) as i64;

    let mut limits = EXCEEDED_LIMITS.lock().unwrap();
    limits.retain(|l| l.limit_type != limit_type);
    limits.push(ExceededLimit {
        limit_type: limit_type.clone(),
        resets_at,
    });

    let mut headers = MOCK_HEADERS.lock().unwrap();
    if !limits.is_empty() {
        headers.status = Some(MockStatus::Rejected);
    }
}

pub fn set_mock_early_warning(claim_abbrev: &str, utilization: f64, hours_from_now: Option<f64>) {
    if !is_ant_user() {
        return;
    }

    let mut enabled = MOCK_ENABLED.lock().unwrap();
    *enabled = true;
    drop(enabled);

    clear_mock_early_warning();

    let default_hours = if claim_abbrev == "5h" {
        4.0
    } else {
        5.0 * 24.0
    };
    let hours = hours_from_now.unwrap_or(default_hours);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let resets_at = (now + (hours * 3600.0) as i64).to_string();

    let mut headers = MOCK_HEADERS.lock().unwrap();

    match claim_abbrev {
        "5h" => {
            headers.five_h_utilization = Some(utilization.to_string());
            headers.five_h_reset = Some(resets_at.clone());
            headers.five_h_surpassed_threshold = Some(utilization.to_string());
        }
        "7d" => {
            headers.seven_d_utilization = Some(utilization.to_string());
            headers.seven_d_reset = Some(resets_at.clone());
            headers.seven_d_surpassed_threshold = Some(utilization.to_string());
        }
        "overage" => {
            headers.overage_utilization = Some(utilization.to_string());
            headers.overage_surpassed_threshold = Some(utilization.to_string());
        }
        _ => {}
    }

    if headers.status.is_none() {
        headers.status = Some(MockStatus::Allowed);
    }
}

pub fn clear_mock_early_warning() {
    let mut headers = MOCK_HEADERS.lock().unwrap();
    headers.five_h_utilization = None;
    headers.five_h_reset = None;
    headers.five_h_surpassed_threshold = None;
    headers.seven_d_utilization = None;
    headers.seven_d_reset = None;
    headers.seven_d_surpassed_threshold = None;
}

pub fn set_mock_rate_limit_scenario(scenario: MockScenario) {
    if !is_ant_user() {
        return;
    }

    match scenario {
        MockScenario::Clear => {
            let mut headers = MOCK_HEADERS.lock().unwrap();
            *headers = MockHeaders::default();
            drop(headers);

            let mut msg = MOCK_HEADERLESS_429_MESSAGE.lock().unwrap();
            *msg = None;
            drop(msg);

            let mut enabled = MOCK_ENABLED.lock().unwrap();
            *enabled = false;
            return;
        }
        _ => {}
    }

    let mut enabled = MOCK_ENABLED.lock().unwrap();
    *enabled = true;
    drop(enabled);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let five_hours_from_now = (now + 5 * 3600).to_string();
    let seven_days_from_now = (now + 7 * 24 * 3600).to_string();

    let mut headers = MOCK_HEADERS.lock().unwrap();
    *headers = MockHeaders::default();
    drop(headers);

    let mut msg = MOCK_HEADERLESS_429_MESSAGE.lock().unwrap();
    *msg = None;
    drop(msg);

    let preserve_overage = matches!(
        scenario,
        MockScenario::OverageActive | MockScenario::OverageWarning | MockScenario::OverageExhausted
    );

    if !preserve_overage {
        let mut limits = EXCEEDED_LIMITS.lock().unwrap();
        limits.clear();
    }

    let mut headers = MOCK_HEADERS.lock().unwrap();

    match scenario {
        MockScenario::Normal => {
            headers.status = Some(MockStatus::Allowed);
            headers.reset = Some(five_hours_from_now);
        }
        MockScenario::SessionLimitReached => {
            let mut limits = EXCEEDED_LIMITS.lock().unwrap();
            limits.push(ExceededLimit {
                limit_type: MockClaim::FiveHour,
                resets_at: five_hours_from_now.parse().unwrap_or(0),
            });
            headers.status = Some(MockStatus::Rejected);
        }
        MockScenario::ApproachingWeeklyLimit => {
            headers.status = Some(MockStatus::AllowedWarning);
            headers.reset = Some(seven_days_from_now.clone());
            headers.claim = Some(MockClaim::SevenDay);
        }
        MockScenario::WeeklyLimitReached => {
            let mut limits = EXCEEDED_LIMITS.lock().unwrap();
            limits.push(ExceededLimit {
                limit_type: MockClaim::SevenDay,
                resets_at: seven_days_from_now.parse().unwrap_or(0),
            });
            headers.status = Some(MockStatus::Rejected);
        }
        MockScenario::ExtraUsageRequired => {
            let mut msg = MOCK_HEADERLESS_429_MESSAGE.lock().unwrap();
            *msg = Some("Extra usage is required for long context requests.".to_string());
        }
        _ => {
            headers.status = Some(MockStatus::Allowed);
        }
    }
}

pub fn get_mock_headerless_429_message() -> Option<String> {
    if !is_ant_user() {
        return None;
    }

    if let Ok(val) = std::env::var(ai::CLAUDE_MOCK_HEADERLESS_429) {
        if !val.is_empty() {
            return Some(val);
        }
    }

    let enabled = MOCK_ENABLED.lock().unwrap();
    if !*enabled {
        return None;
    }
    drop(enabled);

    let msg = MOCK_HEADERLESS_429_MESSAGE.lock().unwrap();
    msg.clone()
}

pub fn get_mock_headers() -> Option<MockHeaders> {
    let enabled = MOCK_ENABLED.lock().unwrap();
    if !*enabled || !is_ant_user() {
        return None;
    }
    drop(enabled);

    let headers = MOCK_HEADERS.lock().unwrap();
    let has_content =
        headers.status.is_some() || headers.reset.is_some() || headers.claim.is_some();

    if has_content {
        Some(headers.clone())
    } else {
        None
    }
}

pub fn get_mock_status() -> String {
    let enabled = MOCK_ENABLED.lock().unwrap();
    if !*enabled {
        return "No mock headers active (using real limits)".to_string();
    }
    drop(enabled);

    let headers = MOCK_HEADERS.lock().unwrap();

    let mut lines = vec!["Active mock headers:".to_string()];

    if let Some(ref status) = headers.status {
        let status_str = match status {
            MockStatus::Allowed => "allowed",
            MockStatus::AllowedWarning => "allowed_warning",
            MockStatus::Rejected => "rejected",
        };
        lines.push(format!("  Status: {}", status_str));
    }

    if let Some(ref reset) = headers.reset {
        lines.push(format!("  Reset: {}", reset));
    }

    lines.join("\n")
}

pub fn clear_mock_headers() {
    let mut headers = MOCK_HEADERS.lock().unwrap();
    *headers = MockHeaders::default();
    drop(headers);

    let mut limits = EXCEEDED_LIMITS.lock().unwrap();
    limits.clear();

    let mut msg = MOCK_HEADERLESS_429_MESSAGE.lock().unwrap();
    *msg = None;

    let mut enabled = MOCK_ENABLED.lock().unwrap();
    *enabled = false;
}

pub fn should_process_mock_limits() -> bool {
    is_ant_user() && {
        let enabled = MOCK_ENABLED.lock().unwrap();
        *enabled || std::env::var(ai::CLAUDE_MOCK_HEADERLESS_429).is_ok()
    }
}
