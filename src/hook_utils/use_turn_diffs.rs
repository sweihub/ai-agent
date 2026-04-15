use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct TurnDiff {
    pub turn_index: usize,
    pub additions: usize,
    pub deletions: usize,
    pub timestamp: u64,
}

pub struct TurnDiffsTracker {
    diffs: VecDeque<TurnDiff>,
    max_history: usize,
}

impl TurnDiffsTracker {
    pub fn new(max_history: usize) -> Self {
        Self {
            diffs: VecDeque::new(),
            max_history,
        }
    }

    pub fn add_diff(&mut self, turn_index: usize, additions: usize, deletions: usize) {
        let diff = TurnDiff {
            turn_index,
            additions,
            deletions,
            timestamp: now_timestamp(),
        };

        self.diffs.push_back(diff);

        while self.diffs.len() > self.max_history {
            self.diffs.pop_front();
        }
    }

    pub fn get_diffs(&self) -> Vec<TurnDiff> {
        self.diffs.iter().cloned().collect()
    }

    pub fn get_latest(&self) -> Option<TurnDiff> {
        self.diffs.back().cloned()
    }

    pub fn get_total_additions(&self) -> usize {
        self.diffs.iter().map(|d| d.additions).sum()
    }

    pub fn get_total_deletions(&self) -> usize {
        self.diffs.iter().map(|d| d.deletions).sum()
    }

    pub fn get_net_change(&self) -> isize {
        let additions = self.get_total_additions() as isize;
        let deletions = self.get_total_deletions() as isize;
        additions - deletions
    }

    pub fn get_turn_count(&self) -> usize {
        self.diffs.len()
    }

    pub fn clear(&mut self) {
        self.diffs.clear();
    }

    pub fn get_turn_diff(&self, turn_index: usize) -> Option<TurnDiff> {
        self.diffs
            .iter()
            .find(|d| d.turn_index == turn_index)
            .cloned()
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

    #[test]
    fn test_turn_diffs_tracker() {
        let mut tracker = TurnDiffsTracker::new(10);

        tracker.add_diff(0, 10, 5);
        tracker.add_diff(1, 20, 3);

        assert_eq!(tracker.get_turn_count(), 2);
        assert_eq!(tracker.get_total_additions(), 30);
        assert_eq!(tracker.get_total_deletions(), 8);
    }

    #[test]
    fn test_net_change() {
        let mut tracker = TurnDiffsTracker::new(10);

        tracker.add_diff(0, 10, 5);
        assert_eq!(tracker.get_net_change(), 5);

        tracker.add_diff(1, 3, 10);
        assert_eq!(tracker.get_net_change(), -2);
    }

    #[test]
    fn test_max_history() {
        let mut tracker = TurnDiffsTracker::new(2);

        tracker.add_diff(0, 10, 0);
        tracker.add_diff(1, 10, 0);
        tracker.add_diff(2, 10, 0);

        assert_eq!(tracker.get_turn_count(), 2);
    }
}
