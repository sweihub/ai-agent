// Source: /data/home/swei/claudecode/openclaudecode/src/utils/errors.ts
pub fn error_message(error: &dyn std::error::Error) -> String {
    error.to_string()
}

pub fn to_error<T: std::fmt::Display + Sized>(err: T) -> String {
    err.to_string()
}

pub fn is_enoent(error: &str) -> bool {
    error.contains("No such file")
}