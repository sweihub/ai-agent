//! Session history functionality for assistant mode

pub const HISTORY_PAGE_SIZE: usize = 100;

#[derive(Debug, Clone)]
pub struct HistoryPage {
    pub events: Vec<serde_json::Value>,
    pub first_id: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone)]
pub struct HistoryAuthCtx {
    pub base_url: String,
    pub headers: std::collections::HashMap<String, String>,
}
