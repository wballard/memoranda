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

// Validation constants
const MIN_VALID_PORT: u16 = 1024;
const MIN_MEMO_FILE_SIZE: u64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub data_dir: PathBuf,
    pub log_level: String,
    pub mcp_server_port: u16,
    pub minimum_rust_version: String,
    pub max_memo_file_size: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: DEFAULT_MCP_SERVER_PORT,
            minimum_rust_version: DEFAULT_MINIMUM_RUST_VERSION.to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
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

    pub fn validate(&self) -> Result<()> {
        if self.mcp_server_port < MIN_VALID_PORT {
            return Err(MemorandaError::Validation(format!(
                "Invalid port number: {}. Port must be {} or higher",
                self.mcp_server_port, MIN_VALID_PORT
            )));
        }

        if self.log_level.is_empty() {
            return Err(MemorandaError::Validation(
                "Log level cannot be empty".to_string(),
            ));
        }

        if self.minimum_rust_version.is_empty() {
            return Err(MemorandaError::Validation(
                "Minimum Rust version cannot be empty".to_string(),
            ));
        }

        // Validate that the minimum Rust version is parseable and is a stable version
        match semver::Version::parse(&self.minimum_rust_version) {
            Ok(version) => {
                if !version.pre.is_empty() || !version.build.is_empty() {
                    return Err(MemorandaError::Validation(format!(
                        "Invalid minimum Rust version format: {}. Must be a stable version (e.g., 1.70.0), pre-release and build metadata are not allowed",
                        self.minimum_rust_version
                    )));
                }
            }
            Err(_) => {
                return Err(MemorandaError::Validation(format!(
                    "Invalid minimum Rust version format: {}. Must be in semver format (e.g., 1.70.0)",
                    self.minimum_rust_version
                )));
            }
        }

        if self.max_memo_file_size < MIN_MEMO_FILE_SIZE {
            return Err(MemorandaError::Validation(format!(
                "Maximum memo file size must be at least {} bytes",
                MIN_MEMO_FILE_SIZE
            )));
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
        assert_eq!(settings.minimum_rust_version, loaded_settings.minimum_rust_version);
        assert_eq!(settings.max_memo_file_size, loaded_settings.max_memo_file_size);
    }

    #[test]
    fn test_settings_validation_invalid_rust_version() {
        let settings = Settings {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: DEFAULT_MCP_SERVER_PORT,
            minimum_rust_version: "invalid.version".to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
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
        };
        assert!(settings.validate().is_ok());
        
        // Test maximum valid port
        let settings = Settings {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            mcp_server_port: u16::MAX,
            minimum_rust_version: DEFAULT_MINIMUM_RUST_VERSION.to_string(),
            max_memo_file_size: DEFAULT_MAX_MEMO_FILE_SIZE,
        };
        assert!(settings.validate().is_ok());
    }
}
