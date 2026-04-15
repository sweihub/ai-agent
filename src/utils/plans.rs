// Source: /data/home/swei/claudecode/openclaudecode/src/utils/plans.ts
//! Plans utilities for planning mode.

use serde::{Deserialize, Serialize};

/// A plan item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub steps: Vec<PlanStep>,
    pub status: PlanStatus,
}

/// Plan step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub status: StepStatus,
}

/// Plan status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Step status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
    Failed,
}

impl Plan {
    pub fn new(id: String) -> Self {
        Self {
            id,
            steps: Vec::new(),
            status: PlanStatus::Pending,
        }
    }

    pub fn add_step(&mut self, description: &str) -> &PlanStep {
        let step = PlanStep {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.to_string(),
            status: StepStatus::Pending,
        };

        self.steps.push(step);
        self.steps.last().unwrap()
    }
}
