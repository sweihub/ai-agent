// Source: ~/claudecode/openclaudecode/src/tasks/pillLabel.ts

#![allow(dead_code)]

use crate::constants::figures::{DIAMOND_FILLED, DIAMOND_OPEN};
use crate::tasks::types::BackgroundTaskState;

/// Produces the compact footer-pill label for a set of background tasks.
/// Used by both the footer pill and the turn-duration transcript line so the
/// two surfaces agree on terminology.
pub fn get_pill_label(tasks: &[BackgroundTaskState]) -> String {
    let n = tasks.len();

    if n == 0 {
        return format!("{} background tasks", n);
    }

    let all_same_type = tasks.iter().all(|t| t.task_type() == tasks[0].task_type());

    if all_same_type {
        match tasks[0].task_type() {
            "local_bash" => {
                let monitors = tasks
                    .iter()
                    .filter(|t| {
                        t.task_type() == "local_bash"
                            && matches!(t, BackgroundTaskState::LocalShell(s) if s.kind == Some(crate::tasks::guards::BashTaskKind::Monitor))
                    })
                    .count();
                let shells = n - monitors;
                let mut parts: Vec<String> = Vec::new();
                if shells > 0 {
                    if shells == 1 {
                        parts.push("1 shell".to_string());
                    } else {
                        parts.push(format!("{} shells", shells));
                    }
                }
                if monitors > 0 {
                    if monitors == 1 {
                        parts.push("1 monitor".to_string());
                    } else {
                        parts.push(format!("{} monitors", monitors));
                    }
                }
                parts.join(", ")
            }
            "in_process_teammate" => {
                let team_names: std::collections::HashSet<String> = tasks
                    .iter()
                    .filter_map(|t| {
                        if let BackgroundTaskState::InProcessTeammate(m) = t {
                            m.get("identity")
                                .and_then(|i| i.get("teamName"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                let team_count = team_names.len();
                if team_count == 1 {
                    "1 team".to_string()
                } else {
                    format!("{} teams", team_count)
                }
            }
            "local_agent" => {
                if n == 1 {
                    "1 local agent".to_string()
                } else {
                    format!("{} local agents", n)
                }
            }
            "remote_agent" => {
                if n == 1 {
                    // Per design mockup: diamond open while running/needs-input,
                    // diamond filled once ExitPlanMode is awaiting approval.
                    if let BackgroundTaskState::RemoteAgent(first) = &tasks[0] {
                        let is_ultraplan = first
                            .get("isUltraplan")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        if is_ultraplan {
                            let ultraplan_phase = first
                                .get("ultraplanPhase")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            match ultraplan_phase {
                                "plan_ready" => {
                                    return format!("{} ultraplan ready", DIAMOND_FILLED);
                                }
                                "needs_input" => {
                                    return format!("{} ultraplan needs your input", DIAMOND_OPEN);
                                }
                                _ => {
                                    return format!("{} ultraplan", DIAMOND_OPEN);
                                }
                            }
                        }
                    }
                    format!("{} 1 cloud session", DIAMOND_OPEN)
                } else {
                    format!("{} {} cloud sessions", DIAMOND_OPEN, n)
                }
            }
            "local_workflow" => {
                if n == 1 {
                    "1 background workflow".to_string()
                } else {
                    format!("{} background workflows", n)
                }
            }
            "monitor_mcp" => {
                if n == 1 {
                    "1 monitor".to_string()
                } else {
                    format!("{} monitors", n)
                }
            }
            "dream" => "dreaming".to_string(),
            other => format!("{} background {}", n, if n == 1 { "task" } else { "tasks" }),
        }
    } else {
        format!("{} background {}", n, if n == 1 { "task" } else { "tasks" })
    }
}

/// True when the pill should show the dimmed " arrow down to view" call-to-action.
/// Per the state diagram: only the two attention states (needs_input,
/// plan_ready) surface the CTA; plain running shows just the diamond + label.
pub fn pill_needs_cta(tasks: &[BackgroundTaskState]) -> bool {
    if tasks.len() != 1 {
        return false;
    }
    let t = &tasks[0];
    if t.task_type() != "remote_agent" {
        return false;
    }

    if let BackgroundTaskState::RemoteAgent(m) = t {
        let is_ultraplan = m
            .get("isUltraplan")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let has_ultraplan_phase = m.get("ultraplanPhase").is_some();
        is_ultraplan && has_ultraplan_phase
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_shell_task(command: &str, kind: Option<crate::tasks::guards::BashTaskKind>) -> BackgroundTaskState {
        use crate::tasks::guards::LocalShellTaskState;
        BackgroundTaskState::LocalShell(LocalShellTaskState {
            id: "test".to_string(),
            task_type: "local_bash".to_string(),
            r#type: "local_bash".to_string(),
            status: crate::task::TaskStatus::running,
            description: command.to_string(),
            tool_use_id: None,
            start_time: 0,
            end_time: None,
            total_paused_ms: None,
            output_file: "".to_string(),
            output_offset: 0,
            notified: false,
            command: command.to_string(),
            result: None,
            completion_status_sent_in_attachment: false,
            shell_command: None,
            unregister_cleanup: None,
            cleanup_timeout_id: None,
            last_reported_total_lines: 0,
            is_backgrounded: Some(true),
            agent_id: None,
            kind,
        })
    }

    #[test]
    fn test_get_pill_label_single_shell() {
        let tasks = vec![create_shell_task("echo hello", None)];
        let label = get_pill_label(&tasks);
        assert_eq!(label, "1 shell");
    }

    #[test]
    fn test_get_pill_label_multiple_shells() {
        let tasks = vec![
            create_shell_task("echo 1", None),
            create_shell_task("echo 2", None),
        ];
        let label = get_pill_label(&tasks);
        assert_eq!(label, "2 shells");
    }

    #[test]
    fn test_get_pill_label_mixed_shells_and_monitors() {
        use crate::tasks::guards::BashTaskKind;
        let tasks = vec![
            create_shell_task("echo 1", None),
            create_shell_task("monitor", Some(BashTaskKind::Monitor)),
        ];
        let label = get_pill_label(&tasks);
        assert_eq!(label, "1 shell, 1 monitor");
    }

    #[test]
    fn test_pill_needs_cta_single_remote_agent() {
        let mut map = std::collections::HashMap::new();
        map.insert("isUltraplan".to_string(), serde_json::json!(true));
        map.insert("ultraplanPhase".to_string(), serde_json::json!("plan_ready"));
        map.insert("status".to_string(), serde_json::json!("running"));
        let tasks = vec![BackgroundTaskState::RemoteAgent(map)];
        assert!(pill_needs_cta(&tasks));
    }

    #[test]
    fn test_pill_needs_cta_multiple_tasks() {
        let mut map = std::collections::HashMap::new();
        map.insert("isUltraplan".to_string(), serde_json::json!(true));
        map.insert("status".to_string(), serde_json::json!("running"));
        let tasks = vec![
            BackgroundTaskState::RemoteAgent(map.clone()),
            BackgroundTaskState::RemoteAgent(map),
        ];
        assert!(!pill_needs_cta(&tasks));
    }

    #[test]
    fn test_pill_needs_cta_non_ultraplan() {
        let mut map = std::collections::HashMap::new();
        map.insert("status".to_string(), serde_json::json!("running"));
        let tasks = vec![BackgroundTaskState::RemoteAgent(map)];
        assert!(!pill_needs_cta(&tasks));
    }
}
