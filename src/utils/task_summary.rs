use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TaskSummary {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub in_progress: usize,
    pub pending: usize,
    pub duration_ms: u64,
}

impl TaskSummary {
    pub fn new() -> Self {
        Self {
            total: 0,
            completed: 0,
            failed: 0,
            in_progress: 0,
            pending: 0,
            duration_ms: 0,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.completed as f64 / self.total as f64) * 100.0
    }

    pub fn is_complete(&self) -> bool {
        self.total > 0 && (self.completed + self.failed) == self.total
    }

    pub fn add_completed(&mut self) {
        self.completed += 1;
    }

    pub fn add_failed(&mut self) {
        self.failed += 1;
    }

    pub fn add_in_progress(&mut self) {
        self.in_progress += 1;
    }

    pub fn add_pending(&mut self) {
        self.pending += 1;
    }
}

impl Default for TaskSummary {
    fn default() -> Self {
        Self::new()
    }
}

pub fn format_summary(summary: &TaskSummary) -> String {
    format!(
        "Tasks: {} completed, {} failed, {} in progress, {} pending ({:.1}% success rate)",
        summary.completed,
        summary.failed,
        summary.in_progress,
        summary.pending,
        summary.success_rate()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_summary() {
        let mut summary = TaskSummary::new();
        summary.total = 10;
        summary.completed = 8;
        summary.failed = 2;

        assert_eq!(summary.success_rate(), 80.0);
        assert!(summary.is_complete());
    }
}
