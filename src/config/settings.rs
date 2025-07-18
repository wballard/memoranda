use crate::error::{MemorandaError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub data_dir: PathBuf,
    pub log_level: String,
    pub mcp_server_port: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            log_level: "info".to_string(),
            mcp_server_port: 8080,
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
        if self.mcp_server_port < 1024 {
            return Err(MemorandaError::Validation(format!(
                "Invalid port number: {}. Port must be 1024 or higher",
                self.mcp_server_port
            )));
        }

        if self.log_level.is_empty() {
            return Err(MemorandaError::Validation(
                "Log level cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        info!("Loading settings from file: {:?}", path);
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let settings: Settings = serde_json::from_str(&content)?;
            Ok(settings)
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
        assert_eq!(settings.data_dir, PathBuf::from("./data"));
        assert_eq!(settings.log_level, "info");
        assert_eq!(settings.mcp_server_port, 8080);
    }

    #[test]
    fn test_settings_validation_valid_port() {
        let settings = Settings {
            data_dir: PathBuf::from("./data"),
            log_level: "info".to_string(),
            mcp_server_port: 8080,
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_settings_validation_invalid_port() {
        let settings = Settings {
            data_dir: PathBuf::from("./data"),
            log_level: "info".to_string(),
            mcp_server_port: 80, // Invalid port
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_settings_validation_empty_log_level() {
        let settings = Settings {
            data_dir: PathBuf::from("./data"),
            log_level: "".to_string(),
            mcp_server_port: 8080,
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
    }
}
