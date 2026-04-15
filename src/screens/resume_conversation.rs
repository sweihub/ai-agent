use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeConversationScreen {
    pub sessions: Vec<ResumeSession>,
    pub selected_index: usize,
    pub search_query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeSession {
    pub session_id: String,
    pub title: String,
    pub preview: String,
    pub created_at: i64,
    pub last_active: i64,
}

impl ResumeConversationScreen {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            selected_index: 0,
            search_query: None,
        }
    }

    pub fn add_session(&mut self, session: ResumeSession) {
        self.sessions.push(session);
    }

    pub fn select_next(&mut self) {
        if !self.sessions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.sessions.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.sessions.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.sessions.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn set_search(&mut self, query: Option<String>) {
        self.search_query = query;
    }

    pub fn selected_session(&self) -> Option<&ResumeSession> {
        self.sessions.get(self.selected_index)
    }
}

impl Default for ResumeConversationScreen {
    fn default() -> Self {
        Self::new()
    }
}
