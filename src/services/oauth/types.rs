// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OAuthTokens {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(rename = "refreshToken", default)]
    pub refresh_token: Option<String>,
    #[serde(rename = "expiresAt", default)]
    pub expires_at: Option<i64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl OAuthTokens {
    pub fn new() -> Self {
        Self::default()
    }
}

pub type SubscriptionType = String;
pub type BillingType = String;
pub type OAuthProfileResponse = HashMap<String, serde_json::Value>;
pub type ReferralEligibilityResponse = HashMap<String, serde_json::Value>;
pub type ReferralRedemptionsResponse = HashMap<String, serde_json::Value>;
pub type ReferrerRewardInfo = HashMap<String, serde_json::Value>;
