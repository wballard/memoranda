use anyhow::Result;
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
    pub fn new() -> Self {
        info!("Creating default settings");
        Self::default()
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        info!("Loading settings from file: {:?}", path);
        // TODO: Implement file loading
        Ok(Self::default())
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        info!("Saving settings to file: {:?}", path);
        // TODO: Implement file saving
        Ok(())
    }
}