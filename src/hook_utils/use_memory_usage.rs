use std::collections::HashMap;

const HIGH_MEMORY_THRESHOLD: u64 = 1_610_612_736;
const CRITICAL_MEMORY_THRESHOLD: u64 = 2_684_354_560;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryStatus {
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct MemoryUsageInfo {
    pub heap_used: u64,
    pub status: MemoryStatus,
}

pub struct MemoryMonitor {
    last_status: MemoryStatus,
    poll_interval_ms: u64,
}

impl MemoryMonitor {
    pub fn new(poll_interval_ms: u64) -> Self {
        Self {
            last_status: MemoryStatus::Normal,
            poll_interval_ms,
        }
    }

    pub fn check_memory(&mut self) -> Option<MemoryUsageInfo> {
        let heap_used = get_heap_usage();

        let status = if heap_used >= CRITICAL_MEMORY_THRESHOLD {
            MemoryStatus::Critical
        } else if heap_used >= HIGH_MEMORY_THRESHOLD {
            MemoryStatus::High
        } else {
            MemoryStatus::Normal
        };

        self.last_status = status;

        if status == MemoryStatus::Normal {
            None
        } else {
            Some(MemoryUsageInfo { heap_used, status })
        }
    }

    pub fn get_status(&self) -> MemoryStatus {
        self.last_status
    }

    pub fn get_poll_interval_ms(&self) -> u64 {
        self.poll_interval_ms
    }
}

pub fn get_heap_usage() -> u64 {
    #[cfg(target_os = "windows")]
    {
        return 0;
    }

    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return kb * 1024;
                    }
                }
            }
        }
    }

    0
}

pub fn get_memory_status(heap_used: u64) -> MemoryStatus {
    if heap_used >= CRITICAL_MEMORY_THRESHOLD {
        MemoryStatus::Critical
    } else if heap_used >= HIGH_MEMORY_THRESHOLD {
        MemoryStatus::High
    } else {
        MemoryStatus::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_status_normal() {
        assert_eq!(get_memory_status(0), MemoryStatus::Normal);
        assert_eq!(get_memory_status(1000000), MemoryStatus::Normal);
    }

    #[test]
    fn test_memory_status_high() {
        assert_eq!(
            get_memory_status(HIGH_MEMORY_THRESHOLD + 1),
            MemoryStatus::High
        );
    }

    #[test]
    fn test_memory_status_critical() {
        assert_eq!(
            get_memory_status(CRITICAL_MEMORY_THRESHOLD + 1),
            MemoryStatus::Critical
        );
    }
}
