use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub struct ConcurrentSessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    max_concurrent: usize,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub started_at: Instant,
    pub last_activity: Instant,
}

impl ConcurrentSessionManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent,
        }
    }

    pub fn create_session(&self, id: String) -> Result<Session, String> {
        let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;

        if sessions.len() >= self.max_concurrent {
            return Err("Max concurrent sessions reached".to_string());
        }

        let now = Instant::now();
        let session = Session {
            id: id.clone(),
            started_at: now,
            last_activity: now,
        };

        sessions.insert(id, session.clone());
        Ok(session)
    }

    pub fn update_activity(&self, id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;

        if let Some(session) = sessions.get_mut(id) {
            session.last_activity = Instant::now();
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    pub fn close_session(&self, id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;
        sessions.remove(id);
        Ok(())
    }

    pub fn get_active_count(&self) -> Result<usize, String> {
        let sessions = self.sessions.read().map_err(|e| e.to_string())?;
        Ok(sessions.len())
    }

    pub fn cleanup_idle(&self, max_idle: Duration) -> Result<usize, String> {
        let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;
        let now = Instant::now();

        let idle: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| now.duration_since(s.last_activity) > max_idle)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &idle {
            sessions.remove(id);
        }

        Ok(idle.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_management() {
        let manager = ConcurrentSessionManager::new(2);

        let session = manager.create_session("s1".to_string()).unwrap();
        assert_eq!(session.id, "s1");

        assert_eq!(manager.get_active_count().unwrap(), 1);

        manager.close_session("s1").unwrap();
        assert_eq!(manager.get_active_count().unwrap(), 0);
    }
}
