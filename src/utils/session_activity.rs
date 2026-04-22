// Source: /data/home/swei/claudecode/openclaudecode/src/utils/sessionActivity.ts
//! Session activity tracking - track and analyze session activity events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Types of session activity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    Message,
    ToolUse,
    Command,
    FileAccess,
    SessionStart,
    SessionEnd,
    Error,
    UserInput,
    Compact,
    MemoryUpdate,
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityType::Message => write!(f, "message"),
            ActivityType::ToolUse => write!(f, "tool_use"),
            ActivityType::Command => write!(f, "command"),
            ActivityType::FileAccess => write!(f, "file_access"),
            ActivityType::SessionStart => write!(f, "session_start"),
            ActivityType::SessionEnd => write!(f, "session_end"),
            ActivityType::Error => write!(f, "error"),
            ActivityType::UserInput => write!(f, "user_input"),
            ActivityType::Compact => write!(f, "compact"),
            ActivityType::MemoryUpdate => write!(f, "memory_update"),
        }
    }
}

/// Session activity event with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivity {
    pub activity_type: ActivityType,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

impl SessionActivity {
    pub fn new(activity_type: ActivityType) -> Self {
        Self {
            activity_type,
            details: None,
            timestamp: Utc::now(),
            metadata: None,
        }
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Get the age of this activity
    pub fn age(&self) -> Duration {
        let now = Utc::now();
        (now - self.timestamp).to_std().unwrap_or(Duration::ZERO)
    }
}

/// Track session activities with time-based filtering and statistics
pub struct SessionActivityTracker {
    activities: Vec<SessionActivity>,
    max_capacity: usize,
}

impl SessionActivityTracker {
    pub fn new() -> Self {
        Self {
            activities: Vec::new(),
            max_capacity: 10_000,
        }
    }

    /// Create a tracker with a custom max capacity
    pub fn with_capacity(max_capacity: usize) -> Self {
        Self {
            activities: Vec::new(),
            max_capacity,
        }
    }

    /// Record an activity
    pub fn record(&mut self, activity: SessionActivity) {
        // Evict oldest entries if at capacity
        if self.activities.len() >= self.max_capacity {
            let remove_count = self.max_capacity / 4; // Remove 25% when full
            self.activities.drain(..remove_count);
        }
        self.activities.push(activity);
    }

    /// Record an activity by type
    pub fn record_type(&mut self, activity_type: ActivityType) {
        self.record(SessionActivity::new(activity_type));
    }

    /// Record an activity by type with details
    pub fn record_type_with_details(&mut self, activity_type: ActivityType, details: String) {
        self.record(SessionActivity::new(activity_type).with_details(details));
    }

    /// Get all activities
    pub fn get_activities(&self) -> &[SessionActivity] {
        &self.activities
    }

    /// Get activities within a duration from now
    pub fn get_recent(&self, duration: Duration) -> Vec<&SessionActivity> {
        let now = Utc::now();
        let cutoff = now - chrono::Duration::from_std(duration).unwrap_or(chrono::Duration::zero());

        self.activities
            .iter()
            .filter(|a| a.timestamp >= cutoff)
            .collect()
    }

    /// Get the count of recent activities within a duration
    pub fn get_recent_count(&self, duration: Duration) -> usize {
        self.get_recent(duration).len()
    }

    /// Get activities of a specific type
    pub fn get_activities_by_type(&self, activity_type: ActivityType) -> Vec<&SessionActivity> {
        self.activities
            .iter()
            .filter(|a| a.activity_type == activity_type)
            .collect()
    }

    /// Get the count of activities by type
    pub fn count_by_type(&self, activity_type: ActivityType) -> usize {
        self.get_activities_by_type(activity_type).len()
    }

    /// Get the last activity
    pub fn get_last_activity(&self) -> Option<&SessionActivity> {
        self.activities.last()
    }

    /// Get the last activity of a specific type
    pub fn get_last_activity_of_type(
        &self,
        activity_type: ActivityType,
    ) -> Option<&SessionActivity> {
        self.activities
            .iter()
            .rev()
            .find(|a| a.activity_type == activity_type)
    }

    /// Get the time since the last activity
    pub fn time_since_last_activity(&self) -> Option<Duration> {
        self.activities.last().map(|a| a.age())
    }

    /// Get the time since the last activity of a specific type
    pub fn time_since_last_activity_of_type(
        &self,
        activity_type: ActivityType,
    ) -> Option<Duration> {
        self.get_last_activity_of_type(activity_type)
            .map(|a| a.age())
    }

    /// Get activity rate (activities per second) over a duration
    pub fn get_activity_rate(&self, duration: Duration) -> f64 {
        let count = self.get_recent_count(duration);
        let secs = duration.as_secs_f64();
        if secs > 0.0 { count as f64 / secs } else { 0.0 }
    }

    /// Check if there has been any activity within a duration
    pub fn has_recent_activity(&self, duration: Duration) -> bool {
        self.get_recent_count(duration) > 0
    }

    /// Get total activity count
    pub fn total_count(&self) -> usize {
        self.activities.len()
    }

    /// Clear all activities
    pub fn clear(&mut self) {
        self.activities.clear();
    }

    /// Export activities for serialization
    pub fn export_activities(&self) -> Vec<SessionActivity> {
        self.activities.clone()
    }

    /// Import activities from serialized data
    pub fn import_activities(&mut self, activities: Vec<SessionActivity>) {
        self.activities = activities;
    }

    /// Get a summary of activity counts by type
    pub fn get_activity_summary(&self) -> serde_json::Value {
        let mut summary = serde_json::Map::new();
        for activity_type in &[
            ActivityType::Message,
            ActivityType::ToolUse,
            ActivityType::Command,
            ActivityType::FileAccess,
            ActivityType::SessionStart,
            ActivityType::SessionEnd,
            ActivityType::Error,
            ActivityType::UserInput,
            ActivityType::Compact,
            ActivityType::MemoryUpdate,
        ] {
            let count = self.count_by_type(*activity_type);
            summary.insert(
                activity_type.to_string(),
                serde_json::Value::Number(count.into()),
            );
        }
        serde_json::Value::Object(summary)
    }
}

impl Default for SessionActivityTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tracker() {
        let tracker = SessionActivityTracker::new();
        assert_eq!(tracker.total_count(), 0);
    }

    #[test]
    fn test_record_activity() {
        let mut tracker = SessionActivityTracker::new();
        tracker.record_type(ActivityType::Message);
        assert_eq!(tracker.total_count(), 1);
    }

    #[test]
    fn test_record_with_details() {
        let mut tracker = SessionActivityTracker::new();
        tracker.record_type_with_details(ActivityType::ToolUse, "ReadFile".to_string());
        assert_eq!(tracker.count_by_type(ActivityType::ToolUse), 1);
    }

    #[test]
    fn test_get_recent_count() {
        let mut tracker = SessionActivityTracker::new();
        tracker.record_type(ActivityType::Message);
        // Activity just recorded, so it's recent
        assert!(tracker.get_recent_count(Duration::from_secs(60)) > 0);
    }

    #[test]
    fn test_clear() {
        let mut tracker = SessionActivityTracker::new();
        tracker.record_type(ActivityType::Message);
        tracker.record_type(ActivityType::ToolUse);
        tracker.clear();
        assert_eq!(tracker.total_count(), 0);
    }

    #[test]
    fn test_activity_type_display() {
        assert_eq!(ActivityType::Message.to_string(), "message");
        assert_eq!(ActivityType::ToolUse.to_string(), "tool_use");
        assert_eq!(ActivityType::Command.to_string(), "command");
    }

    #[test]
    fn test_last_activity() {
        let mut tracker = SessionActivityTracker::new();
        assert!(tracker.get_last_activity().is_none());

        tracker.record_type(ActivityType::Message);
        assert!(tracker.get_last_activity().is_some());
    }

    #[test]
    fn test_time_since_last_activity() {
        let mut tracker = SessionActivityTracker::new();
        assert!(tracker.time_since_last_activity().is_none());

        tracker.record_type(ActivityType::Message);
        assert!(tracker.time_since_last_activity().is_some());
    }

    #[test]
    fn test_activity_summary() {
        let mut tracker = SessionActivityTracker::new();
        tracker.record_type(ActivityType::Message);
        tracker.record_type(ActivityType::Message);
        tracker.record_type(ActivityType::ToolUse);

        let summary = tracker.get_activity_summary();
        assert!(summary.is_object());
    }

    #[test]
    fn test_activity_rate() {
        let mut tracker = SessionActivityTracker::new();
        for _ in 0..5 {
            tracker.record_type(ActivityType::Message);
        }
        let rate = tracker.get_activity_rate(Duration::from_secs(60));
        assert!(rate > 0.0);
    }

    #[test]
    fn test_has_recent_activity() {
        let mut tracker = SessionActivityTracker::new();
        assert!(!tracker.has_recent_activity(Duration::from_secs(60)));

        tracker.record_type(ActivityType::Message);
        assert!(tracker.has_recent_activity(Duration::from_secs(60)));
    }

    #[test]
    fn test_export_import() {
        let mut tracker = SessionActivityTracker::new();
        tracker.record_type(ActivityType::Message);
        tracker.record_type(ActivityType::ToolUse);

        let exported = tracker.export_activities();
        let mut tracker2 = SessionActivityTracker::new();
        tracker2.import_activities(exported);

        assert_eq!(tracker2.total_count(), 2);
    }
}
