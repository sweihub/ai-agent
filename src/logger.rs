// Source: /data/home/swei/claudecode/openclaudecode/src/utils/telemetry/logger.ts
#![allow(dead_code)]

pub fn create_logger(name: &str) -> Logger {
    Logger {
        name: name.to_string(),
    }
}

pub struct Logger {
    name: String,
}

impl Logger {
    pub fn info(&self, message: &str) {
        println!("[INFO] {}: {}", self.name, message);
    }

    pub fn error(&self, message: &str) {
        eprintln!("[ERROR] {}: {}", self.name, message);
    }

    pub fn debug(&self, message: &str) {
        println!("[DEBUG] {}: {}", self.name, message);
    }
}
