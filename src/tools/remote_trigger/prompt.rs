// Source: ~/claudecode/openclaudecode/src/tools/RemoteTriggerTool/prompt.ts
pub const REMOTE_TRIGGER_TOOL_NAME: &str = "RemoteTrigger";

pub const DESCRIPTION: &str =
    "Manage scheduled remote Claude Code agents (triggers) via the claude.ai CCR API. Auth is handled in-process — the token never reaches the shell.";

pub const PROMPT: &str = r#"Call the claude.ai remote-trigger API. Use this instead of curl — the OAuth token is added automatically in-process and never exposed.

Actions:
- list: GET /v1/code/triggers
- get: GET /v1/code/triggers/{trigger_id}
- create: POST /v1/code/triggers (requires body)
- update: POST /v1/code/triggers/{trigger_id} (requires body, partial update)
- run: POST /v1/code/triggers/{trigger_id}/run

The response is the raw JSON from the API."#;
