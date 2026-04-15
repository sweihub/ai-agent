use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct TaskListWatcher {
    tasks: Arc<RwLock<HashMap<String, TaskInfo>>>,
    status_listeners: Arc<RwLock<HashMap<String, Vec<TaskStatus>>>>,
}

impl TaskListWatcher {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            status_listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_task(&self, id: String, name: String) {
        let now = now_timestamp();
        let task = TaskInfo {
            id: id.clone(),
            name,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
        };

        let mut tasks = self.tasks.write().await;
        tasks.insert(id, task);
    }

    pub async fn update_status(&self, id: &str, status: TaskStatus) -> bool {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(id) {
            let old_status = task.status.clone();
            task.status = status.clone();
            task.updated_at = now_timestamp();

            let mut listeners = self.status_listeners.write().await;
            if let Some(listener_list) = listeners.get_mut(id) {
                listener_list.push(status);
            }

            true
        } else {
            false
        }
    }

    pub async fn get_task(&self, id: &str) -> Option<TaskInfo> {
        let tasks = self.tasks.read().await;
        tasks.get(id).cloned()
    }

    pub async fn list_tasks(&self) -> Vec<TaskInfo> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    pub async fn list_by_status(&self, status: TaskStatus) -> Vec<TaskInfo> {
        let tasks = self.tasks.read().await;
        tasks
            .values()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }

    pub async fn get_running_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks
            .values()
            .filter(|t| t.status == TaskStatus::Running)
            .count()
    }

    pub async fn remove_task(&self, id: &str) -> Option<TaskInfo> {
        let mut tasks = self.tasks.write().await;
        tasks.remove(id)
    }

    pub async fn clear_completed(&self) {
        let mut tasks = self.tasks.write().await;
        tasks.retain(|_, t| t.status != TaskStatus::Completed && t.status != TaskStatus::Failed);
    }
}

impl Default for TaskListWatcher {
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
    async fn test_task_list_watcher() {
        let watcher = TaskListWatcher::new();

        watcher
            .add_task("task-1".to_string(), "Test Task".to_string())
            .await;

        let task = watcher.get_task("task-1").await;
        assert!(task.is_some());
    }

    #[tokio::test]
    async fn test_update_status() {
        let watcher = TaskListWatcher::new();

        watcher
            .add_task("task-1".to_string(), "Test".to_string())
            .await;
        watcher.update_status("task-1", TaskStatus::Running).await;

        let task = watcher.get_task("task-1").await;
        assert_eq!(task.unwrap().status, TaskStatus::Running);
    }

    #[tokio::test]
    async fn test_list_by_status() {
        let watcher = TaskListWatcher::new();

        watcher
            .add_task("task-1".to_string(), "Task 1".to_string())
            .await;
        watcher
            .add_task("task-2".to_string(), "Task 2".to_string())
            .await;

        watcher.update_status("task-1", TaskStatus::Running).await;

        let running = watcher.list_by_status(TaskStatus::Running).await;
        assert_eq!(running.len(), 1);
    }
}
