use std::io::{self, Write};

pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    fn to_string(&self) -> &str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

pub struct Logger {
    level: LogLevel,
    prefix: String,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            level: LogLevel::Info,
            prefix: String::new(),
        }
    }

    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    fn should_log(&self, level: &LogLevel) -> bool {
        let self_level = match self.level {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warning => 2,
            LogLevel::Error => 3,
        };
        let target_level = match level {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warning => 2,
            LogLevel::Error => 3,
        };
        target_level >= self_level
    }

    pub fn debug(&self, message: &str) {
        if self.should_log(&LogLevel::Debug) {
            self.log(&LogLevel::Debug, message);
        }
    }

    pub fn info(&self, message: &str) {
        if self.should_log(&LogLevel::Info) {
            self.log(&LogLevel::Info, message);
        }
    }

    pub fn warning(&self, message: &str) {
        if self.should_log(&LogLevel::Warning) {
            self.log(&LogLevel::Warning, message);
        }
    }

    pub fn error(&self, message: &str) {
        if self.should_log(&LogLevel::Error) {
            self.log(&LogLevel::Error, message);
        }
    }

    fn log(&self, level: &LogLevel, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let prefix = if self.prefix.is_empty() {
            String::new()
        } else {
            format!("[{}] ", self.prefix)
        };
        eprintln!("{} {} {}{}", timestamp, level.to_string(), prefix, message);
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
