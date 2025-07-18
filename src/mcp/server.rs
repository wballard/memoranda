use anyhow::Result;
use tracing::info;

pub struct McpServer {
    pub name: String,
}

impl McpServer {
    pub fn new(name: String) -> Self {
        info!("Creating MCP server: {}", name);
        Self { name }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting MCP server: {}", self.name);
        Ok(())
    }
}