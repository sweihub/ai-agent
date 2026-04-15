// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyLimitsResponse {
    pub restrictions: HashMap<String, PolicyRestriction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRestriction {
    pub allowed: bool,
}

#[derive(Debug, Clone)]
pub struct PolicyLimitsFetchResult {
    pub success: bool,
    pub restrictions: Option<Option<HashMap<String, PolicyRestriction>>>,
    pub etag: Option<String>,
    pub error: Option<String>,
    pub skip_retry: Option<bool>,
}
