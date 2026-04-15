// Source: ~/claudecode/openclaudecode/src/tools/ScheduleCronTool/prompt.ts
use std::env;

pub fn cron_list_tool_name() -> &'static str {
    "CronList"
}

pub fn cron_list_description() -> String {
    "List all active cron jobs".to_string()
}

pub fn cron_delete_tool_name() -> &'static str {
    "CronDelete"
}

pub fn cron_delete_description() -> String {
    "Cancel a scheduled cron job".to_string()
}

pub fn is_kairos_cron_enabled() -> bool {
    // Check if KAIROS_CRON is enabled (localized from CLAUDE_CODE_CRON)
    env::var("AI_CODE_CRON")
        .map(|v| v != "0" && v != "false" && v != "no")
        .unwrap_or(false)
}

pub fn is_durable_cron_enabled() -> bool {
    // Check if durable cron is enabled (localized from ANTHROPIC_)
    env::var("AI_DURABLE_CRON")
        .map(|v| v != "0" && v != "false" && v != "no")
        .unwrap_or(false)
}

pub fn build_cron_list_prompt(durable_enabled: bool) -> String {
    let durable_note = if durable_enabled {
        "\n\n**Durable crons:** Jobs persist across sessions by default. Jobs with `durable: false` are session-only."
    } else {
        ""
    };

    format!(
        r#"# CronList

List all active cron jobs.

Returns a list of scheduled cron jobs with their ID, schedule, human-readable schedule, and prompt.

| Field | Description |
|-------|-------------|
| `id` | Unique job identifier |
| `cron` | Cron expression (e.g., "0 9 * * 1") |
| `humanSchedule` | Human-readable schedule (e.g., "Every Monday at 9:00 AM") |
| `prompt` | The job's prompt (truncated) |
| `recurring` | Whether the job repeats |
| `durable` | Whether the job persists across sessions (only shown if false){}

No parameters required.
"#,
        durable_note
    )
    .trim()
    .to_string()
}

pub fn build_cron_delete_prompt(durable_enabled: bool) -> String {
    let durable_note = if durable_enabled {
        "\n\n**Note:** If the job was session-only (`durable: false`), it will only be removed from the current session's cron list."
    } else {
        ""
    };

    format!(
        r#"# CronDelete

Cancel a scheduled cron job by its ID.

| Parameter | Description |
|-----------|-------------|
| `id` | Job ID returned by CronCreate |{}

Returns confirmation of the cancelled job.
"#,
        durable_note
    )
    .trim()
    .to_string()
}
