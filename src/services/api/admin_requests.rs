// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/adminRequests.ts
//! Admin requests module
//! Handles admin requests like limit increase and seat upgrade

use crate::constants::oauth::get_oauth_config;
use crate::session_history::{get_oauth_headers, prepare_api_request};

/// Admin request type
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRequestType {
    LimitIncrease,
    SeatUpgrade,
}

/// Admin request status
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRequestStatus {
    Pending,
    Approved,
    Dismissed,
}

/// Seat upgrade details
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminRequestSeatUpgradeDetails {
    pub message: Option<String>,
    pub current_seat_tier: Option<String>,
}

/// Admin request create params
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "request_type")]
pub enum AdminRequestCreateParams {
    #[serde(rename = "limit_increase")]
    LimitIncrease { details: Option<serde_json::Value> },
    #[serde(rename = "seat_upgrade")]
    SeatUpgrade {
        details: AdminRequestSeatUpgradeDetails,
    },
}

/// Admin request
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminRequest {
    pub uuid: String,
    pub status: AdminRequestStatus,
    pub requester_uuid: Option<String>,
    pub created_at: String,
    #[serde(flatten)]
    pub params: AdminRequestParams,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "request_type")]
pub enum AdminRequestParams {
    #[serde(rename = "limit_increase")]
    LimitIncrease { details: Option<serde_json::Value> },
    #[serde(rename = "seat_upgrade")]
    SeatUpgrade {
        details: AdminRequestSeatUpgradeDetails,
    },
}

/// Admin request eligibility response
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminRequestEligibilityResponse {
    pub request_type: AdminRequestType,
    pub is_allowed: bool,
}

/// Create an admin request (limit increase or seat upgrade).
/// For Team/Enterprise users who don't have billing/admin permissions,
/// this creates a request that their admin can act on.
/// If a pending request of the same type already exists for this user,
/// returns the existing request instead of creating a new one.
pub async fn create_admin_request(
    params: AdminRequestCreateParams,
) -> Result<AdminRequest, String> {
    let (access_token, org_uuid) = prepare_api_request().await.map_err(|e| e.to_string())?;

    let mut headers = get_oauth_headers(&access_token);
    headers.insert("x-organization-uuid".to_string(), org_uuid.clone());

    let config = get_oauth_config();
    let url = format!(
        "{}/api/oauth/organizations/{}/admin_requests",
        config.base_api_url, org_uuid
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .headers(
            headers
                .into_iter()
                .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
                .collect(),
        )
        .json(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response
        .json::<AdminRequest>()
        .await
        .map_err(|e| e.to_string())
}

/// Get pending admin request of a specific type for the current user.
/// Returns the pending request if one exists, otherwise null.
pub async fn get_my_admin_requests(
    request_type: AdminRequestType,
    statuses: Vec<AdminRequestStatus>,
) -> Result<Option<Vec<AdminRequest>>, String> {
    let (access_token, org_uuid) = prepare_api_request().await.map_err(|e| e.to_string())?;

    let mut headers = get_oauth_headers(&access_token);
    headers.insert("x-organization-uuid".to_string(), org_uuid.clone());

    let config = get_oauth_config();
    let request_type_str = match request_type {
        AdminRequestType::LimitIncrease => "limit_increase",
        AdminRequestType::SeatUpgrade => "seat_upgrade",
    };

    let mut url = format!(
        "{}/api/oauth/organizations/{}/admin_requests/me?request_type={}",
        config.base_api_url, org_uuid, request_type_str
    );

    for status in &statuses {
        let status_str = match status {
            AdminRequestStatus::Pending => "pending",
            AdminRequestStatus::Approved => "approved",
            AdminRequestStatus::Dismissed => "dismissed",
        };
        url.push_str(&format!("&statuses={}", status_str));
    }

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .headers(
            headers
                .into_iter()
                .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
                .collect(),
        )
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(None);
    }

    response
        .json::<Vec<AdminRequest>>()
        .await
        .map(Some)
        .map_err(|e| e.to_string())
}

/// Check if a specific admin request type is allowed for this org.
pub async fn check_admin_request_eligibility(
    request_type: AdminRequestType,
) -> Result<AdminRequestEligibilityResponse, String> {
    let (access_token, org_uuid) = prepare_api_request().await.map_err(|e| e.to_string())?;

    let mut headers = get_oauth_headers(&access_token);
    headers.insert("x-organization-uuid".to_string(), org_uuid.clone());

    let config = get_oauth_config();
    let request_type_str = match request_type {
        AdminRequestType::LimitIncrease => "limit_increase",
        AdminRequestType::SeatUpgrade => "seat_upgrade",
    };

    let url = format!(
        "{}/api/oauth/organizations/{}/admin_requests/eligibility?request_type={}",
        config.base_api_url, org_uuid, request_type_str
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .headers(
            headers
                .into_iter()
                .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
                .collect(),
        )
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response
        .json::<AdminRequestEligibilityResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_request_create_params_limit_increase() {
        let params = AdminRequestCreateParams::LimitIncrease { details: None };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("limit_increase"));
    }

    #[test]
    fn test_admin_request_create_params_seat_upgrade() {
        let params = AdminRequestCreateParams::SeatUpgrade {
            details: AdminRequestSeatUpgradeDetails {
                message: Some("Please upgrade".to_string()),
                current_seat_tier: Some("team".to_string()),
            },
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("seat_upgrade"));
    }
}
