use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Mutex;

thread_local! {
    static ACTIVITY_MANAGER: RefCell<Option<ActivityManager>> = RefCell::new(None);
}

pub struct ActivityManager {
    active_operations: HashSet<String>,
    last_user_activity_time: u64,
    last_cli_recorded_time: u64,
    is_cli_active: bool,
}

impl ActivityManager {
    pub fn new() -> Self {
        Self {
            active_operations: HashSet::new(),
            last_user_activity_time: 0,
            last_cli_recorded_time: current_time_millis(),
            is_cli_active: false,
        }
    }

    pub fn record_user_activity(&mut self) {
        if !self.is_cli_active && self.last_user_activity_time != 0 {
            let now = current_time_millis();
            let time_since_last = (now - self.last_user_activity_time) / 1000;
            if time_since_last > 0 && time_since_last < 5 {}
        }
        self.last_user_activity_time = current_time_millis();
    }

    pub fn start_cli_activity(&mut self, operation_id: &str) {
        if self.active_operations.contains(operation_id) {
            self.end_cli_activity(operation_id);
        }
        let was_empty = self.active_operations.is_empty();
        self.active_operations.insert(operation_id.to_string());
        if was_empty {
            self.is_cli_active = true;
            self.last_cli_recorded_time = current_time_millis();
        }
    }

    pub fn end_cli_activity(&mut self, operation_id: &str) {
        self.active_operations.remove(operation_id);
        if self.active_operations.is_empty() {
            let now = current_time_millis();
            let time_since = (now - self.last_cli_recorded_time) / 1000;
            if time_since > 0 {}
            self.last_cli_recorded_time = now;
            self.is_cli_active = false;
        }
    }

    pub fn get_activity_states(&self) -> ActivityStates {
        let now = current_time_millis();
        let time_since_user = (now - self.last_user_activity_time) / 1000;
        let is_user_active = time_since_user < 5;

        ActivityStates {
            is_user_active,
            is_cli_active: self.is_cli_active,
            active_operation_count: self.active_operations.len(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActivityStates {
    pub is_user_active: bool,
    pub is_cli_active: bool,
    pub active_operation_count: usize,
}

fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn get_instance() -> &'static RefCell<Option<ActivityManager>> {
    &ACTIVITY_MANAGER
}
