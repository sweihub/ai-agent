// Source: /data/home/swei/claudecode/openclaudecode/src/utils/bash/specs/timeout.ts
pub struct Timeout {
    duration_ms: u64,
    started_at: Option<u64>,
}

impl Timeout {
    pub fn new(duration_ms: u64) -> Self {
        Self {
            duration_ms,
            started_at: None,
        }
    }

    pub fn start(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.started_at = Some(now);
    }

    pub fn is_expired(&self) -> bool {
        if let Some(started) = self.started_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            now.saturating_sub(started) >= self.duration_ms
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.started_at = None;
    }

    pub fn remaining(&self) -> u64 {
        if let Some(started) = self.started_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let elapsed = now.saturating_sub(started);
            self.duration_ms.saturating_sub(elapsed)
        } else {
            self.duration_ms
        }
    }
}
