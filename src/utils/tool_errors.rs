use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellErrorData {
    pub code: Option<i32>,
    pub interrupted: bool,
    pub stderr: String,
    pub stdout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZodIssue {
    pub code: String,
    pub message: String,
    pub path: Vec<serde_json::Value>,
    #[serde(default)]
    pub keys: Option<Vec<String>>,
    #[serde(default)]
    pub expected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZodErrorData {
    pub issues: Vec<ZodIssue>,
}

const INTERRUPT_MESSAGE: &str = "Interrupted";

pub fn format_error(error: &str) -> String {
    if error.is_empty() {
        return INTERRUPT_MESSAGE.to_string();
    }

    if error.len() > 10000 {
        let half = 5000;
        let start = &error[..half];
        let end = &error[error.len() - half..];
        return format!(
            "{}\n\n... [{} characters truncated] ...\n\n{}",
            start,
            error.len() - 10000,
            end
        );
    }

    error.to_string()
}

pub fn get_error_parts(error: &ShellErrorData) -> Vec<String> {
    let mut parts = Vec::new();

    if let Some(code) = error.code {
        parts.push(format!("Exit code {}", code));
    }

    if error.interrupted {
        parts.push(INTERRUPT_MESSAGE.to_string());
    }

    if !error.stderr.is_empty() {
        parts.push(error.stderr.clone());
    }

    if !error.stdout.is_empty() {
        parts.push(error.stdout.clone());
    }

    parts
}

fn format_validation_path(path: &[serde_json::Value]) -> String {
    if path.is_empty() {
        return String::new();
    }

    path.iter()
        .enumerate()
        .map(|(i, segment)| match segment {
            serde_json::Value::Number(n) => format!("[{}]", n),
            serde_json::Value::String(s) => {
                if i == 0 {
                    s.clone()
                } else {
                    format!(".{}", s)
                }
            }
            other => other.to_string(),
        })
        .collect()
}

pub fn format_zod_validation_error(tool_name: &str, error: &ZodErrorData) -> String {
    let missing_params: Vec<String> = error
        .issues
        .iter()
        .filter(|err| err.code == "invalid_type" && err.message.contains("received undefined"))
        .map(|err| format_validation_path(&err.path))
        .collect();

    let unexpected_params: Vec<String> = error
        .issues
        .iter()
        .filter(|err| err.code == "unrecognized_keys")
        .flat_map(|err| err.keys.clone().unwrap_or_default())
        .collect();

    let type_mismatch_params: Vec<(String, String, String)> = error
        .issues
        .iter()
        .filter(|err| err.code == "invalid_type" && !err.message.contains("received undefined"))
        .map(|err| {
            let param = format_validation_path(&err.path);
            let expected = err
                .expected
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let received = err
                .message
                .split("received ")
                .nth(1)
                .map(|s| s.split_whitespace().next().unwrap_or("unknown"))
                .unwrap_or("unknown")
                .to_string();
            (param, expected, received)
        })
        .collect();

    let mut error_parts = Vec::new();

    for param in &missing_params {
        error_parts.push(format!("The required parameter `{}` is missing", param));
    }

    for param in &unexpected_params {
        error_parts.push(format!("An unexpected parameter `{}` was provided", param));
    }

    for (param, expected, received) in &type_mismatch_params {
        error_parts.push(format!(
            "The parameter `{}` type is expected as `{}` but provided as `{}`",
            param, expected, received
        ));
    }

    if error_parts.is_empty() {
        error
            .issues
            .first()
            .map(|i| i.message.clone())
            .unwrap_or_default()
    } else {
        let issue_word = if error_parts.len() > 1 {
            "issues"
        } else {
            "issue"
        };
        format!(
            "{} failed due to the following {}:\n{}",
            tool_name,
            issue_word,
            error_parts.join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_error_short() {
        let result = format_error("short error");
        assert_eq!(result, "short error");
    }

    #[test]
    fn test_format_validation_path() {
        let path = vec![
            serde_json::Value::String("todos".to_string()),
            serde_json::Value::Number(serde_json::Number::from(0)),
            serde_json::Value::String("activeForm".to_string()),
        ];
        let result = format_validation_path(&path);
        assert_eq!(result, "todos[0].activeForm");
    }
}
