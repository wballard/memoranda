use thiserror::Error;

/// Main error type for the Memoranda application
#[derive(Error, Debug)]
pub enum MemorandaError {
    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Storage errors
    #[error("Storage error: {message}")]
    Storage {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// MCP server errors
    #[error("MCP server error: {message}")]
    McpServer {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// CLI errors
    #[error("CLI error: {message}")]
    Cli {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation { message: String },
}

/// Specific error type for memo operations
#[derive(Error, Debug)]
pub enum MemoError {
    #[error("Memo not found: {id}")]
    NotFound { id: String },

    #[error("Invalid memo format: {reason}")]
    InvalidFormat { reason: String },

    #[error("Memo validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Memo operation failed: {operation}")]
    OperationFailed {
        operation: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Specific error type for storage operations
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: String },

    #[error("File system error: {message}")]
    FileSystemError {
        message: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Serialization error: {message}")]
    SerializationError {
        message: String,
        #[source]
        source: serde_json::Error,
    },
}

/// Specific error type for MCP protocol operations
#[derive(Error, Debug)]
pub enum McpError {
    #[error("Protocol error: {message}")]
    Protocol { message: String },

    #[error("Invalid request: {reason}")]
    InvalidRequest { reason: String },

    #[error("Tool not found: {tool_name}")]
    ToolNotFound { tool_name: String },

    #[error("Tool execution failed: {tool_name}")]
    ToolExecutionFailed {
        tool_name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Server initialization failed: {reason}")]
    ServerInitializationFailed { reason: String },
}

/// Specific error type for CLI operations
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid command: {command}")]
    InvalidCommand { command: String },

    #[error("Missing argument: {argument}")]
    MissingArgument { argument: String },

    #[error("Invalid argument: {argument} - {reason}")]
    InvalidArgument { argument: String, reason: String },

    #[error("Command execution failed: {command}")]
    ExecutionFailed {
        command: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

// Error conversion implementations
impl From<MemoError> for MemorandaError {
    fn from(err: MemoError) -> Self {
        MemorandaError::Storage {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<StorageError> for MemorandaError {
    fn from(err: StorageError) -> Self {
        MemorandaError::Storage {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<McpError> for MemorandaError {
    fn from(err: McpError) -> Self {
        MemorandaError::McpServer {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<CliError> for MemorandaError {
    fn from(err: CliError) -> Self {
        MemorandaError::Cli {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

// Convert MemoStoreError to StorageError
impl From<crate::memo::storage::MemoStoreError> for StorageError {
    fn from(err: crate::memo::storage::MemoStoreError) -> Self {
        match err {
            crate::memo::storage::MemoStoreError::MemoNotFound { id } => {
                StorageError::FileNotFound { path: id }
            }
            crate::memo::storage::MemoStoreError::FileOperation { source } => {
                StorageError::FileSystemError {
                    message: "File operation failed".to_string(),
                    source,
                }
            }
            crate::memo::storage::MemoStoreError::Serialization { source } => {
                StorageError::SerializationError {
                    message: "Serialization failed".to_string(),
                    source,
                }
            }
            _ => StorageError::FileSystemError {
                message: err.to_string(),
                source: std::io::Error::new(std::io::ErrorKind::Other, err.to_string()),
            },
        }
    }
}

// Convert MemoStoreError to MemorandaError (optimized direct conversion)
impl From<crate::memo::storage::MemoStoreError> for MemorandaError {
    fn from(err: crate::memo::storage::MemoStoreError) -> Self {
        match &err {
            crate::memo::storage::MemoStoreError::MemoNotFound { id } => MemorandaError::Storage {
                message: format!("Memo not found: {id}"),
                source: Some(Box::new(err)),
            },
            crate::memo::storage::MemoStoreError::FileOperation { .. } => MemorandaError::Storage {
                message: "File operation failed".to_string(),
                source: Some(Box::new(err)),
            },
            crate::memo::storage::MemoStoreError::Serialization { .. } => MemorandaError::Storage {
                message: "Serialization failed".to_string(),
                source: Some(Box::new(err)),
            },
            crate::memo::storage::MemoStoreError::InvalidFrontmatter { file, .. } => {
                MemorandaError::Storage {
                    message: format!("Invalid frontmatter in file: {file}"),
                    source: Some(Box::new(err)),
                }
            }
            crate::memo::storage::MemoStoreError::MissingFrontmatter { file } => {
                MemorandaError::Storage {
                    message: format!("Missing frontmatter in file: {file}"),
                    source: Some(Box::new(err)),
                }
            }
            crate::memo::storage::MemoStoreError::Validation { .. } => MemorandaError::Validation {
                message: err.to_string(),
            },
            _ => MemorandaError::Storage {
                message: err.to_string(),
                source: Some(Box::new(err)),
            },
        }
    }
}

// Helper functions for creating errors with context
impl MemorandaError {
    /// Create a configuration error with context
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
        }
    }

    /// Create a configuration error with source
    pub fn config_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Config {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a storage error with context
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage {
            message: message.into(),
            source: None,
        }
    }

    /// Create a storage error with source
    pub fn storage_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Storage {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an MCP server error with context
    pub fn mcp_server(message: impl Into<String>) -> Self {
        Self::McpServer {
            message: message.into(),
            source: None,
        }
    }

    /// Create an MCP server error with source
    pub fn mcp_server_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::McpServer {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a CLI error with context
    pub fn cli(message: impl Into<String>) -> Self {
        Self::Cli {
            message: message.into(),
            source: None,
        }
    }

    /// Create a CLI error with source
    pub fn cli_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Cli {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }
}

// Helper functions for specific error types
impl MemoError {
    pub fn not_found(id: impl Into<String>) -> Self {
        Self::NotFound { id: id.into() }
    }

    pub fn invalid_format(reason: impl Into<String>) -> Self {
        Self::InvalidFormat {
            reason: reason.into(),
        }
    }

    pub fn validation_failed(reason: impl Into<String>) -> Self {
        Self::ValidationFailed {
            reason: reason.into(),
        }
    }

    pub fn operation_failed(
        operation: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::OperationFailed {
            operation: operation.into(),
            source: Box::new(source),
        }
    }
}

impl StorageError {
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    pub fn permission_denied(path: impl Into<String>) -> Self {
        Self::PermissionDenied { path: path.into() }
    }

    pub fn directory_not_found(path: impl Into<String>) -> Self {
        Self::DirectoryNotFound { path: path.into() }
    }
}

impl McpError {
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol {
            message: message.into(),
        }
    }

    pub fn invalid_request(reason: impl Into<String>) -> Self {
        Self::InvalidRequest {
            reason: reason.into(),
        }
    }

    pub fn tool_not_found(tool_name: impl Into<String>) -> Self {
        Self::ToolNotFound {
            tool_name: tool_name.into(),
        }
    }

    pub fn tool_execution_failed(
        tool_name: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::ToolExecutionFailed {
            tool_name: tool_name.into(),
            source: Box::new(source),
        }
    }

    pub fn server_initialization_failed(reason: impl Into<String>) -> Self {
        Self::ServerInitializationFailed {
            reason: reason.into(),
        }
    }
}

impl CliError {
    pub fn invalid_command(command: impl Into<String>) -> Self {
        Self::InvalidCommand {
            command: command.into(),
        }
    }

    pub fn missing_argument(argument: impl Into<String>) -> Self {
        Self::MissingArgument {
            argument: argument.into(),
        }
    }

    pub fn invalid_argument(argument: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidArgument {
            argument: argument.into(),
            reason: reason.into(),
        }
    }

    pub fn execution_failed(
        command: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::ExecutionFailed {
            command: command.into(),
            source: Box::new(source),
        }
    }
}

pub type Result<T> = std::result::Result<T, MemorandaError>;
pub type MemoResult<T> = std::result::Result<T, MemoError>;
pub type StorageResult<T> = std::result::Result<T, StorageError>;
pub type McpResult<T> = std::result::Result<T, McpError>;
pub type CliResult<T> = std::result::Result<T, CliError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_memo_error_creation() {
        let error = MemoError::not_found("test-id");
        assert_eq!(error.to_string(), "Memo not found: test-id");

        let error = MemoError::invalid_format("missing required field");
        assert_eq!(
            error.to_string(),
            "Invalid memo format: missing required field"
        );

        let error = MemoError::validation_failed("title too long");
        assert_eq!(error.to_string(), "Memo validation failed: title too long");
    }

    #[test]
    fn test_storage_error_creation() {
        let error = StorageError::file_not_found("/path/to/file");
        assert_eq!(error.to_string(), "File not found: /path/to/file");

        let error = StorageError::permission_denied("/path/to/file");
        assert_eq!(error.to_string(), "Permission denied: /path/to/file");

        let error = StorageError::directory_not_found("/path/to/dir");
        assert_eq!(error.to_string(), "Directory not found: /path/to/dir");
    }

    #[test]
    fn test_mcp_error_creation() {
        let error = McpError::protocol("invalid message format");
        assert_eq!(error.to_string(), "Protocol error: invalid message format");

        let error = McpError::invalid_request("missing required field");
        assert_eq!(error.to_string(), "Invalid request: missing required field");

        let error = McpError::tool_not_found("create_memo");
        assert_eq!(error.to_string(), "Tool not found: create_memo");

        let error = McpError::server_initialization_failed("memo store not found");
        assert_eq!(
            error.to_string(),
            "Server initialization failed: memo store not found"
        );
    }

    #[test]
    fn test_cli_error_creation() {
        let error = CliError::invalid_command("invalid");
        assert_eq!(error.to_string(), "Invalid command: invalid");

        let error = CliError::missing_argument("--file");
        assert_eq!(error.to_string(), "Missing argument: --file");

        let error = CliError::invalid_argument("--verbose", "expected boolean");
        assert_eq!(
            error.to_string(),
            "Invalid argument: --verbose - expected boolean"
        );
    }

    #[test]
    fn test_memoranda_error_creation() {
        let error = MemorandaError::config("missing configuration file");
        assert_eq!(
            error.to_string(),
            "Configuration error: missing configuration file"
        );

        let error = MemorandaError::storage("file not found");
        assert_eq!(error.to_string(), "Storage error: file not found");

        let error = MemorandaError::mcp_server("server startup failed");
        assert_eq!(error.to_string(), "MCP server error: server startup failed");

        let error = MemorandaError::cli("invalid command");
        assert_eq!(error.to_string(), "CLI error: invalid command");

        let error = MemorandaError::validation("invalid input");
        assert_eq!(error.to_string(), "Validation error: invalid input");
    }

    #[test]
    fn test_error_conversion() {
        let memo_error = MemoError::not_found("test-id");
        let memoranda_error: MemorandaError = memo_error.into();
        assert!(memoranda_error.to_string().contains("Storage error"));

        let storage_error = StorageError::file_not_found("/path/to/file");
        let memoranda_error: MemorandaError = storage_error.into();
        assert!(memoranda_error.to_string().contains("Storage error"));

        let mcp_error = McpError::protocol("invalid message");
        let memoranda_error: MemorandaError = mcp_error.into();
        assert!(memoranda_error.to_string().contains("MCP server error"));

        let cli_error = CliError::invalid_command("invalid");
        let memoranda_error: MemorandaError = cli_error.into();
        assert!(memoranda_error.to_string().contains("CLI error"));
    }

    #[test]
    fn test_error_with_source() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let memoranda_error = MemorandaError::storage_with_source("failed to read file", io_error);

        assert_eq!(
            memoranda_error.to_string(),
            "Storage error: failed to read file"
        );
        assert!(memoranda_error.source().is_some());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let memoranda_error: MemorandaError = io_error.into();
        assert!(memoranda_error.to_string().contains("IO error"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let memoranda_error: MemorandaError = json_error.into();
        assert!(memoranda_error.to_string().contains("JSON error"));
    }

    #[test]
    fn test_error_chain() {
        let root_cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let memo_error = MemoError::operation_failed("create_memo", root_cause);
        let memoranda_error: MemorandaError = memo_error.into();

        // Check the error chain
        assert!(memoranda_error.to_string().contains("Storage error"));
        assert!(memoranda_error.source().is_some());

        let source = memoranda_error.source().unwrap();
        assert!(source.to_string().contains("Memo operation failed"));
    }

    #[test]
    fn test_specific_error_types() {
        // Test that specific error types can be used independently
        let memo_error: MemoResult<()> = Err(MemoError::not_found("test-id"));
        assert!(memo_error.is_err());

        let storage_error: StorageResult<()> = Err(StorageError::file_not_found("/path"));
        assert!(storage_error.is_err());

        let mcp_error: McpResult<()> = Err(McpError::protocol("invalid"));
        assert!(mcp_error.is_err());

        let cli_error: CliResult<()> = Err(CliError::invalid_command("invalid"));
        assert!(cli_error.is_err());
    }
}
