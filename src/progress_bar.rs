#![allow(dead_code)]

pub fn create_progress_bar(total: u64) -> ProgressBar {
    ProgressBar { total, current: 0 }
}

pub struct ProgressBar {
    pub total: u64,
    pub current: u64,
}

impl ProgressBar {
    pub fn set_progress(&mut self, value: u64) {
        self.current = value;
    }

    pub fn finish(self) {}
}
