use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisorMessage {
    pub id: String,
    pub advisor_type: String,
    pub message: String,
    pub suggestions: Vec<String>,
    pub severity: AdvisorSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AdvisorSeverity {
    Info,
    Warning,
    Error,
}

impl AdvisorMessage {
    pub fn new(advisor_type: &str, message: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            advisor_type: advisor_type.to_string(),
            message: message.to_string(),
            suggestions: Vec::new(),
            severity: AdvisorSeverity::Info,
        }
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn with_severity(mut self, severity: AdvisorSeverity) -> Self {
        self.severity = severity;
        self
    }
}
