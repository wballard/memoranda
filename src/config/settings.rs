use crate::error::{MemorandaError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

// Default configuration constants
const DEFAULT_DATA_DIR: &str = "./data";
const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_MCP_SERVER_PORT: u16 = 8080;
const DEFAULT_MINIMUM_RUST_VERSION: &str = "1.70.0";
const DEFAULT_MAX_MEMO_FILE_SIZE: u64 = 1_000_000; // 1MB

// Search configuration constants
const DEFAULT_RECENCY_BOOST_DAYS: f64 = 365.0;
const DEFAULT_SNIPPET_LENGTH: usize = 100;
const DEFAULT_SNIPPET_CONTEXT_PADDING: usize = 2;

// MCP tool configuration
const DEFAULT_EXPECTED_TOOLS: &[&str] = &[
    "create_memo",
    "update_memo",
    "list_memos",
    "get_memo",
    "delete_memo",
    "search_memos",
    "get_all_context",
];

// Validation constants
/// Minimum valid port number for MCP server.
/// Ports below 1024 are privileged ports reserved for system services on Unix-like systems.
/// User applications should use ports 1024 and above to avoid permission conflicts.
const MIN_VALID_PORT: u16 = 1024;
/// Minimum memo file size in bytes.
/// Files must be at least 1 byte to be considered valid memo files.
const MIN_MEMO_FILE_SIZE: u64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub data_dir: PathBuf,
    pub log_level: String,
    pub mcp_server_port: u16,
    pub minimum_rust_version: String,
    pub max_memo_file_size: u64,

    // Search configuration
    pub search_recency_boost_days: f64,
    pub search_snippet_length: usize,
    pub search_snippet_context_padding: usize,

    // MCP configuration
    pub expected_mcp_tools: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: DEFAULT_MCP_SERVER_PORT,
            minimum_rust_version: DEFAULT_MINIMUM_RUST_VERSION.to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
            search_recency_boost_days: DEFAULT_RECENCY_BOOST_DAYS,
            search_snippet_length: DEFAULT_SNIPPET_LENGTH,
            search_snippet_context_padding: DEFAULT_SNIPPET_CONTEXT_PADDING,
            expected_mcp_tools: DEFAULT_EXPECTED_TOOLS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

impl Settings {
    pub fn new() -> Result<Self> {
        info!("Creating default settings");
        let settings = Self::default();
        settings.validate()?;
        Ok(settings)
    }

    /// Creates new validated settings, falling back to defaults if validation fails.
    /// This is a convenience method that encapsulates the common pattern of
    /// Settings::new().unwrap_or_default().
    pub fn new_or_default() -> Self {
        Self::new().unwrap_or_default()
    }

    pub fn validate(&self) -> Result<()> {
        if self.mcp_server_port < MIN_VALID_PORT {
            return Err(MemorandaError::validation(format!(
                "Invalid port number: {}. Port must be {} or higher",
                self.mcp_server_port, MIN_VALID_PORT
            )));
        }

        if self.log_level.is_empty() {
            return Err(MemorandaError::validation("Log level cannot be empty"));
        }

        if self.minimum_rust_version.is_empty() {
            return Err(MemorandaError::validation(
                "Minimum Rust version cannot be empty",
            ));
        }

        // Validate that the minimum Rust version is parseable and is a stable version
        match semver::Version::parse(&self.minimum_rust_version) {
            Ok(version) => {
                if !version.pre.is_empty() || !version.build.is_empty() {
                    return Err(MemorandaError::validation(format!(
                        "Invalid minimum Rust version format: {}. Must be a stable version (e.g., 1.70.0), pre-release and build metadata are not allowed",
                        self.minimum_rust_version
                    )));
                }
            }
            Err(_) => {
                return Err(MemorandaError::validation(format!(
                    "Invalid minimum Rust version format: {}. Must be in semver format (e.g., 1.70.0)",
                    self.minimum_rust_version
                )));
            }
        }

        if self.max_memo_file_size < MIN_MEMO_FILE_SIZE {
            return Err(MemorandaError::validation(format!(
                "Maximum memo file size must be at least {MIN_MEMO_FILE_SIZE} bytes"
            )));
        }

        if self.search_recency_boost_days <= 0.0 {
            return Err(MemorandaError::validation(
                "Search recency boost days must be positive",
            ));
        }

        if self.search_snippet_length == 0 {
            return Err(MemorandaError::validation(
                "Search snippet length must be greater than 0",
            ));
        }

        if self.expected_mcp_tools.is_empty() {
            return Err(MemorandaError::validation(
                "Expected MCP tools list cannot be empty",
            ));
        }

        Ok(())
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        info!("Loading settings from file: {:?}", path);
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            if content.trim().is_empty() {
                info!("Settings file is empty, using defaults");
                Ok(Self::default())
            } else {
                let settings: Settings = serde_json::from_str(&content)?;
                Ok(settings)
            }
        } else {
            info!("Settings file not found, using defaults");
            Ok(Self::default())
        }
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        info!("Saving settings to file: {:?}", path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_settings_creation() {
        let settings = Settings::new().unwrap();
        assert_eq!(settings.data_dir, PathBuf::from(DEFAULT_DATA_DIR));
        assert_eq!(settings.log_level, DEFAULT_LOG_LEVEL);
        assert_eq!(settings.mcp_server_port, DEFAULT_MCP_SERVER_PORT);
        assert_eq!(settings.minimum_rust_version, DEFAULT_MINIMUM_RUST_VERSION);
        assert_eq!(settings.max_memo_file_size, DEFAULT_MAX_MEMO_FILE_SIZE);
    }

    #[test]
    fn test_settings_validation_valid_port() {
        let settings = Settings {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: DEFAULT_MCP_SERVER_PORT,
            minimum_rust_version: DEFAULT_MINIMUM_RUST_VERSION.to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
            search_recency_boost_days: DEFAULT_RECENCY_BOOST_DAYS,
            search_snippet_length: DEFAULT_SNIPPET_LENGTH,
            search_snippet_context_padding: DEFAULT_SNIPPET_CONTEXT_PADDING,
            expected_mcp_tools: DEFAULT_EXPECTED_TOOLS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_settings_validation_invalid_port() {
        let settings = Settings {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: MIN_VALID_PORT - 1, // Invalid port
            minimum_rust_version: DEFAULT_MINIMUM_RUST_VERSION.to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
            search_recency_boost_days: DEFAULT_RECENCY_BOOST_DAYS,
            search_snippet_length: DEFAULT_SNIPPET_LENGTH,
            search_snippet_context_padding: DEFAULT_SNIPPET_CONTEXT_PADDING,
            expected_mcp_tools: DEFAULT_EXPECTED_TOOLS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_settings_validation_empty_log_level() {
        let settings = Settings {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: "".to_string(),
            mcp_server_port: DEFAULT_MCP_SERVER_PORT,
            minimum_rust_version: DEFAULT_MINIMUM_RUST_VERSION.to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
            search_recency_boost_days: DEFAULT_RECENCY_BOOST_DAYS,
            search_snippet_length: DEFAULT_SNIPPET_LENGTH,
            search_snippet_context_padding: DEFAULT_SNIPPET_CONTEXT_PADDING,
            expected_mcp_tools: DEFAULT_EXPECTED_TOOLS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_settings_save_and_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let settings = Settings::new().unwrap();

        // Save settings
        let path = temp_file.path().to_path_buf();
        settings.save_to_file(&path).unwrap();

        // Load settings
        let loaded_settings = Settings::load_from_file(&path).unwrap();

        assert_eq!(settings.data_dir, loaded_settings.data_dir);
        assert_eq!(settings.log_level, loaded_settings.log_level);
        assert_eq!(settings.mcp_server_port, loaded_settings.mcp_server_port);
        assert_eq!(
            settings.minimum_rust_version,
            loaded_settings.minimum_rust_version
        );
        assert_eq!(
            settings.max_memo_file_size,
            loaded_settings.max_memo_file_size
        );
    }

    #[test]
    fn test_settings_validation_invalid_rust_version() {
        let settings = Settings {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: DEFAULT_MCP_SERVER_PORT,
            minimum_rust_version: "invalid.version".to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
            search_recency_boost_days: DEFAULT_RECENCY_BOOST_DAYS,
            search_snippet_length: DEFAULT_SNIPPET_LENGTH,
            search_snippet_context_padding: DEFAULT_SNIPPET_CONTEXT_PADDING,
            expected_mcp_tools: DEFAULT_EXPECTED_TOOLS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_settings_validation_empty_rust_version() {
        let settings = Settings {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: DEFAULT_MCP_SERVER_PORT,
            minimum_rust_version: "".to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
            search_recency_boost_days: DEFAULT_RECENCY_BOOST_DAYS,
            search_snippet_length: DEFAULT_SNIPPET_LENGTH,
            search_snippet_context_padding: DEFAULT_SNIPPET_CONTEXT_PADDING,
            expected_mcp_tools: DEFAULT_EXPECTED_TOOLS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_settings_validation_zero_file_size() {
        let settings = Settings {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: DEFAULT_MCP_SERVER_PORT,
            minimum_rust_version: DEFAULT_MINIMUM_RUST_VERSION.to_string(),
            max_memo_file_size: MIN_MEMO_FILE_SIZE - 1, // Invalid size
            search_recency_boost_days: DEFAULT_RECENCY_BOOST_DAYS,
            search_snippet_length: DEFAULT_SNIPPET_LENGTH,
            search_snippet_context_padding: DEFAULT_SNIPPET_CONTEXT_PADDING,
            expected_mcp_tools: DEFAULT_EXPECTED_TOOLS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_settings_validation_high_port() {
        // Test with a high valid port value
        let settings = Settings {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: u16::MAX, // Maximum possible port
            minimum_rust_version: DEFAULT_MINIMUM_RUST_VERSION.to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
            search_recency_boost_days: DEFAULT_RECENCY_BOOST_DAYS,
            search_snippet_length: DEFAULT_SNIPPET_LENGTH,
            search_snippet_context_padding: DEFAULT_SNIPPET_CONTEXT_PADDING,
            expected_mcp_tools: DEFAULT_EXPECTED_TOOLS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_settings_validation_edge_case_ports() {
        // Test minimum valid port
        let settings = Settings {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: MIN_VALID_PORT,
            minimum_rust_version: DEFAULT_MINIMUM_RUST_VERSION.to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
            search_recency_boost_days: DEFAULT_RECENCY_BOOST_DAYS,
            search_snippet_length: DEFAULT_SNIPPET_LENGTH,
            search_snippet_context_padding: DEFAULT_SNIPPET_CONTEXT_PADDING,
            expected_mcp_tools: DEFAULT_EXPECTED_TOOLS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        assert!(settings.validate().is_ok());

        // Test maximum valid port
        let settings = Settings {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: u16::MAX,
            minimum_rust_version: DEFAULT_MINIMUM_RUST_VERSION.to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
            search_recency_boost_days: DEFAULT_RECENCY_BOOST_DAYS,
            search_snippet_length: DEFAULT_SNIPPET_LENGTH,
            search_snippet_context_padding: DEFAULT_SNIPPET_CONTEXT_PADDING,
            expected_mcp_tools: DEFAULT_EXPECTED_TOOLS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        assert!(settings.validate().is_ok());
    }
}
