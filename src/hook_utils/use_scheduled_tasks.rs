use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub scheduled_at: u64,
    pub interval_ms: Option<u64>,
    pub enabled: bool,
}

pub struct ScheduledTasksManager {
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,
}

impl ScheduledTasksManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn schedule(&self, task: ScheduledTask) {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task);
    }

    pub async fn unschedule(&self, id: &str) -> Option<ScheduledTask> {
        let mut tasks = self.tasks.write().await;
        tasks.remove(id)
    }

    pub async fn get(&self, id: &str) -> Option<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.get(id).cloned()
    }

    pub async fn enable(&self, id: &str) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.enabled = true;
            true
        } else {
            false
        }
    }

    pub async fn disable(&self, id: &str) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.enabled = false;
            true
        } else {
            false
        }
    }

    pub async fn list_enabled(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.values().filter(|t| t.enabled).cloned().collect()
    }

    pub async fn list_all(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    pub async fn get_due_tasks(&self) -> Vec<ScheduledTask> {
        let now = now_timestamp();
        let tasks = self.tasks.read().await;

        tasks
            .values()
            .filter(|t| t.enabled && t.scheduled_at <= now)
            .cloned()
            .collect()
    }

    pub async fn clear(&self) {
        let mut tasks = self.tasks.write().await;
        tasks.clear();
    }
}

impl Default for ScheduledTasksManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_scheduled_task(
    id: &str,
    name: &str,
    scheduled_at_ms: u64,
    interval_ms: Option<u64>,
) -> ScheduledTask {
    ScheduledTask {
        id: id.to_string(),
        name: name.to_string(),
        scheduled_at: scheduled_at_ms,
        interval_ms,
        enabled: true,
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
    async fn test_scheduled_tasks_manager() {
        let manager = ScheduledTasksManager::new();

        let task = create_scheduled_task("task-1", "Test Task", 1000, None);
        manager.schedule(task).await;

        let retrieved = manager.get("task-1").await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_enable_disable() {
        let manager = ScheduledTasksManager::new();

        let task = create_scheduled_task("task-1", "Test", 1000, None);
        manager.schedule(task).await;

        manager.disable("task-1").await;
        let disabled = manager.get("task-1").await;
        assert!(disabled.map(|t| !t.enabled).unwrap_or(false));

        manager.enable("task-1").await;
        let enabled = manager.get("task-1").await;
        assert!(enabled.map(|t| t.enabled).unwrap_or(false));
    }
}
