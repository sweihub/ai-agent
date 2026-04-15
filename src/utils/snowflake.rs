pub struct SnowflakeIdGenerator {
    timestamp: u64,
    worker_id: u64,
    sequence: u64,
}

impl SnowflakeIdGenerator {
    pub fn new(worker_id: u64) -> Self {
        Self {
            timestamp: 0,
            worker_id,
            sequence: 0,
        }
    }

    pub fn next(&mut self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let id = (now << 22) | (self.worker_id << 12) | self.sequence;
        self.sequence = (self.sequence + 1) & 0xFFF;
        id
    }
}
