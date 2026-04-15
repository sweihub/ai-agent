// Source: /data/home/swei/claudecode/openclaudecode/src/tasks.ts
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

pub struct TaskQueue {
    tasks: Arc<RwLock<VecDeque<Task>>>,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub fn push(&self, task: Task) -> Result<(), String> {
        let mut queue = self.tasks.write().map_err(|e| e.to_string())?;
        queue.push_back(task);
        Ok(())
    }

    pub fn pop(&self) -> Result<Option<Task>, String> {
        let mut queue = self.tasks.write().map_err(|e| e.to_string())?;
        Ok(queue.pop_front())
    }

    pub fn peek(&self) -> Result<Option<Task>, String> {
        let queue = self.tasks.read().map_err(|e| e.to_string())?;
        Ok(queue.front().cloned())
    }

    pub fn len(&self) -> Result<usize, String> {
        let queue = self.tasks.read().map_err(|e| e.to_string())?;
        Ok(queue.len())
    }

    pub fn is_empty(&self) -> Result<bool, String> {
        let queue = self.tasks.read().map_err(|e| e.to_string())?;
        Ok(queue.is_empty())
    }

    pub fn clear(&self) -> Result<(), String> {
        let mut queue = self.tasks.write().map_err(|e| e.to_string())?;
        queue.clear();
        Ok(())
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_queue() {
        let queue = TaskQueue::new();

        queue
            .push(Task {
                id: "1".to_string(),
                description: "test".to_string(),
                priority: TaskPriority::Normal,
                status: TaskStatus::Pending,
            })
            .unwrap();

        assert_eq!(queue.len().unwrap(), 1);
    }
}
