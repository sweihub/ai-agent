use std::process::{Command, Output, Stdio};

pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

pub async fn exec_file_no_throw(command: &str, args: Vec<String>) -> ExecResult {
    exec_file_no_throw_inner(command, args)
}

fn exec_file_no_throw_inner(command: &str, args: Vec<String>) -> ExecResult {
    let result = Command::new(command)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match result {
        Ok(output) => ExecResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            code: output.status.code().unwrap_or(-1),
        },
        Err(e) => ExecResult {
            stdout: String::new(),
            stderr: e.to_string(),
            code: -1,
        },
    }
}

pub fn exec_file_no_throw_sync(command: &str, args: Vec<String>) -> ExecResult {
    exec_file_no_throw_inner(command, args)
}

pub async fn exec_file_no_throw_with_cwd(
    command: &str,
    args: Vec<String>,
    cwd: &std::path::Path,
) -> ExecResult {
    let result = Command::new(command)
        .args(&args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match result {
        Ok(output) => ExecResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            code: output.status.code().unwrap_or(-1),
        },
        Err(e) => ExecResult {
            stdout: String::new(),
            stderr: e.to_string(),
            code: -1,
        },
    }
}
