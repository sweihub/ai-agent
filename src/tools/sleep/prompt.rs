// Source: ~/claudecode/openclaudecode/src/tools/SleepTool/prompt.ts

pub const SLEEP_TOOL_NAME: &str = "Sleep";

pub const DESCRIPTION: &str = "Wait for a specified duration";

pub const SLEEP_TOOL_PROMPT: &str = r#"Wait for a specified duration. The user can interrupt the sleep at any time.

Use this when the user tells you to sleep or rest, when you have nothing to do, or when you're waiting for something.

You may receive `<tick>` prompts — these are periodic check-ins. Look for useful work to do before sleeping.

You can call this concurrently with other tools — it won't interfere with them.

Prefer this over `Bash(sleep ...)` — it doesn't hold a shell process.

Each wake-up costs an API call, but the prompt cache expires after 5 minutes of inactivity — balance accordingly."#;
