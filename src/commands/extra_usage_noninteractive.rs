pub async fn run_extra_usage() -> ExtraUsageResult {
    ExtraUsageResult::Message("Extra usage info".to_string())
}

pub enum ExtraUsageResult {
    Message(String),
    Url { url: String, opened: bool },
}

impl ExtraUsageResult {
    pub fn message(value: &str) -> Self {
        ExtraUsageResult::Message(value.to_string())
    }

    pub fn url(url: &str, opened: bool) -> Self {
        ExtraUsageResult::Url {
            url: url.to_string(),
            opened,
        }
    }
}
