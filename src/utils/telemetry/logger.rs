// Source: ~/claudecode/openclaudecode/src/utils/telemetry/logger.rs

use tracing::{debug, error, info, warn};

/// DiagLogger implementation for OpenTelemetry diagnostic logging.
pub struct DiagLogger;

impl DiagLogger {
    /// Log an error message.
    pub fn error(message: &str) {
        error!("[3P telemetry] OTEL diag error: {}", message);
    }

    /// Log a warning message.
    pub fn warn(message: &str) {
        warn!("[3P telemetry] OTEL diag warn: {}", message);
    }

    /// Log an info message (no-op in diag logger).
    #[allow(dead_code)]
    pub fn info(_message: &str) {
        // No-op for diag logger
    }

    /// Log a debug message (no-op in diag logger).
    #[allow(dead_code)]
    pub fn debug(_message: &str) {
        // No-op for diag logger
    }

    /// Log a verbose message (no-op in diag logger).
    #[allow(dead_code)]
    pub fn verbose(_message: &str) {
        // No-op for diag logger
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diag_logger_does_not_panic() {
        DiagLogger::error("test error");
        DiagLogger::warn("test warn");
        DiagLogger::info("test info");
        DiagLogger::debug("test debug");
        DiagLogger::verbose("test verbose");
    }
}
