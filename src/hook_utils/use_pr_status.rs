use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrStatus {
    Open,
    Merged,
    Closed,
    Draft,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PrInfo {
    pub number: u32,
    pub title: String,
    pub status: PrStatus,
    pub author: String,
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct PrStatusManager {
    prs: Arc<RwLock<HashMap<u32, PrInfo>>>,
}

impl PrStatusManager {
    pub fn new() -> Self {
        Self {
            prs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add(&self, pr: PrInfo) {
        let mut prs = self.prs.write().await;
        prs.insert(pr.number, pr);
    }

    pub async fn get(&self, number: u32) -> Option<PrInfo> {
        let prs = self.prs.read().await;
        prs.get(&number).cloned()
    }

    pub async fn remove(&self, number: u32) -> Option<PrInfo> {
        let mut prs = self.prs.write().await;
        prs.remove(&number)
    }

    pub async fn update_status(&self, number: u32, status: PrStatus) -> bool {
        let mut prs = self.prs.write().await;
        if let Some(pr) = prs.get_mut(&number) {
            pr.status = status;
            pr.updated_at = now_timestamp();
            true
        } else {
            false
        }
    }

    pub async fn list(&self) -> Vec<PrInfo> {
        let prs = self.prs.read().await;
        prs.values().cloned().collect()
    }

    pub async fn list_by_status(&self, status: PrStatus) -> Vec<PrInfo> {
        let prs = self.prs.read().await;
        prs.values()
            .filter(|pr| pr.status == status)
            .cloned()
            .collect()
    }

    pub async fn clear(&self) {
        let mut prs = self.prs.write().await;
        prs.clear();
    }
}

impl Default for PrStatusManager {
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
    async fn test_pr_status_manager() {
        let manager = PrStatusManager::new();

        let pr = PrInfo {
            number: 1,
            title: "Test PR".to_string(),
            status: PrStatus::Open,
            author: "testuser".to_string(),
            created_at: 1000,
            updated_at: 1000,
        };

        manager.add(pr).await;

        let retrieved = manager.get(1).await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_list_by_status() {
        let manager = PrStatusManager::new();

        manager
            .add(PrInfo {
                number: 1,
                title: "Open PR".to_string(),
                status: PrStatus::Open,
                author: "user".to_string(),
                created_at: 0,
                updated_at: 0,
            })
            .await;

        manager
            .add(PrInfo {
                number: 2,
                title: "Merged PR".to_string(),
                status: PrStatus::Merged,
                author: "user".to_string(),
                created_at: 0,
                updated_at: 0,
            })
            .await;

        let open_prs = manager.list_by_status(PrStatus::Open).await;
        assert_eq!(open_prs.len(), 1);
    }
}
