use std::fmt;

#[derive(Debug)]
pub enum MemorandaError {
    /// Configuration errors
    Config(String),
    /// Storage errors
    Storage(String),
    /// MCP server errors
    McpServer(String),
    /// IO errors
    Io(std::io::Error),
    /// JSON serialization errors
    Json(serde_json::Error),
    /// Validation errors
    Validation(String),
}

impl fmt::Display for MemorandaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemorandaError::Config(msg) => write!(f, "Configuration error: {msg}"),
            MemorandaError::Storage(msg) => write!(f, "Storage error: {msg}"),
            MemorandaError::McpServer(msg) => write!(f, "MCP server error: {msg}"),
            MemorandaError::Io(err) => write!(f, "IO error: {err}"),
            MemorandaError::Json(err) => write!(f, "JSON error: {err}"),
            MemorandaError::Validation(msg) => write!(f, "Validation error: {msg}"),
        }
    }
}

impl std::error::Error for MemorandaError {}

impl From<std::io::Error> for MemorandaError {
    fn from(err: std::io::Error) -> Self {
        MemorandaError::Io(err)
    }
}

impl From<serde_json::Error> for MemorandaError {
    fn from(err: serde_json::Error) -> Self {
        MemorandaError::Json(err)
    }
}

pub type Result<T> = std::result::Result<T, MemorandaError>;
