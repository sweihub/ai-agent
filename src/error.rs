use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Tool not implemented: {0}")]
    ToolNotImplemented(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("MCP error: {0}")]
    Mcp(String),

    #[error("Skill error: {0}")]
    Skill(String),

    #[error("Command error: {0}")]
    Command(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Max turns reached: {0}")]
    MaxTurns(String),

    #[error("User aborted the request")]
    UserAborted,

    #[error("API connection timeout: {0}")]
    ApiConnectionTimeout(String),

    #[error("Stream ended without events")]
    StreamEndedWithoutEvents,

    #[error("404 stream creation error - fallback needed")]
    Stream404CreationError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
