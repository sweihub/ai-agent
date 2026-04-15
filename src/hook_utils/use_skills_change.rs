use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct SkillChange {
    pub skill_id: String,
    pub change_type: SkillChangeType,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillChangeType {
    Added,
    Removed,
    Updated,
    Enabled,
    Disabled,
}

pub struct SkillsChangeTracker {
    changes: Arc<RwLock<Vec<SkillChange>>>,
    latest_by_skill: Arc<RwLock<HashMap<String, SkillChange>>>,
}

impl SkillsChangeTracker {
    pub fn new() -> Self {
        Self {
            changes: Arc::new(RwLock::new(Vec::new())),
            latest_by_skill: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_change(&self, skill_id: String, change_type: SkillChangeType) {
        let change = SkillChange {
            skill_id: skill_id.clone(),
            change_type,
            timestamp: now_timestamp(),
        };

        let mut changes = self.changes.write().await;
        changes.push(change.clone());

        let mut latest = self.latest_by_skill.write().await;
        latest.insert(skill_id, change);
    }

    pub async fn get_changes(&self) -> Vec<SkillChange> {
        let changes = self.changes.read().await;
        changes.clone()
    }

    pub async fn get_latest_change(&self, skill_id: &str) -> Option<SkillChange> {
        let latest = self.latest_by_skill.read().await;
        latest.get(skill_id).cloned()
    }

    pub async fn has_changes_since(&self, since: u64) -> bool {
        let changes = self.changes.read().await;
        changes.iter().any(|c| c.timestamp > since)
    }

    pub async fn get_changes_since(&self, since: u64) -> Vec<SkillChange> {
        let changes = self.changes.read().await;
        changes
            .iter()
            .filter(|c| c.timestamp > since)
            .cloned()
            .collect()
    }

    pub async fn clear(&self) {
        let mut changes = self.changes.write().await;
        changes.clear();

        let mut latest = self.latest_by_skill.write().await;
        latest.clear();
    }
}

impl Default for SkillsChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}

fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_skills_change_tracker() {
        let tracker = SkillsChangeTracker::new();

        tracker
            .record_change("skill-1".to_string(), SkillChangeType::Added)
            .await;

        let changes = tracker.get_changes().await;
        assert_eq!(changes.len(), 1);

        let latest = tracker.get_latest_change("skill-1").await;
        assert!(latest.is_some());
    }

    #[tokio::test]
    async fn test_changes_since() {
        let tracker = SkillsChangeTracker::new();

        let before = now_timestamp();
        tracker
            .record_change("skill-1".to_string(), SkillChangeType::Added)
            .await;

        let has_changes = tracker.has_changes_since(before).await;
        assert!(has_changes);
    }
}
