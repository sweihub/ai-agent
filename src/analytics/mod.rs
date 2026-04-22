use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone)]
pub enum AnalyticsEvent {
    PageView {
        path: String,
    },
    Action {
        name: String,
        metadata: Vec<(String, String)>,
    },
    Error {
        message: String,
        stack: Option<String>,
    },
    Performance {
        metric: String,
        value: f64,
    },
}

#[derive(Debug, Default)]
pub struct GleanTracker {
    event_count: AtomicU64,
    enabled: Arc<AtomicU64>,
}

impl GleanTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track(&self, event: AnalyticsEvent) {
        if self.is_enabled() {
            self.event_count.fetch_add(1, Ordering::Relaxed);
            self.send_event(event);
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) == 1
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled
            .store(if enabled { 1 } else { 0 }, Ordering::Relaxed);
    }

    pub fn event_count(&self) -> u64 {
        self.event_count.load(Ordering::Relaxed)
    }

    fn send_event(&self, _event: AnalyticsEvent) {}
}

pub fn track_page_view(tracker: &GleanTracker, path: &str) {
    tracker.track(AnalyticsEvent::PageView {
        path: path.to_string(),
    });
}

pub fn track_action(tracker: &GleanTracker, name: &str, metadata: Vec<(String, String)>) {
    tracker.track(AnalyticsEvent::Action {
        name: name.to_string(),
        metadata,
    });
}

pub fn track_error(tracker: &GleanTracker, message: &str, stack: Option<String>) {
    tracker.track(AnalyticsEvent::Error {
        message: message.to_string(),
        stack,
    });
}

pub fn track_performance(tracker: &GleanTracker, metric: &str, value: f64) {
    tracker.track(AnalyticsEvent::Performance {
        metric: metric.to_string(),
        value,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_disabled() {
        let tracker = GleanTracker::new();
        tracker.set_enabled(false);
        tracker.track(AnalyticsEvent::PageView {
            path: "/test".to_string(),
        });
        assert_eq!(tracker.event_count(), 0);
    }

    #[test]
    fn test_tracker_enabled() {
        let tracker = GleanTracker::new();
        tracker.set_enabled(true);
        track_page_view(&tracker, "/test");
        assert_eq!(tracker.event_count(), 1);
    }
}
