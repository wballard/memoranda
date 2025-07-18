use anyhow::Result;
use std::io::{BufRead, BufReader};
use tracing::info;

use crate::memo::MemoStorage;
use super::tools::McpTool;

pub struct McpServer {
    pub name: String,
    storage: MemoStorage,
    tools: Vec<McpTool>,
}

impl McpServer {
    pub fn new(name: String) -> Self {
        info!("Creating MCP server: {}", name);
        let tools = vec![
            McpTool::new(
                "create_memo".to_string(),
                "Create a new memo with title and content".to_string(),
            ),
            McpTool::new(
                "list_memos".to_string(),
                "List all stored memos".to_string(),
            ),
            McpTool::new(
                "get_memo".to_string(),
                "Get a specific memo by ID".to_string(),
            ),
            McpTool::new(
                "delete_memo".to_string(),
                "Delete a memo by ID".to_string(),
            ),
        ];
        
        Self {
            name,
            storage: MemoStorage::new(),
            tools,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting MCP server: {}", self.name);
        
        // Basic MCP server implementation using stdio
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin);
        
        info!("MCP server listening on stdio: {}", self.name);
        
        // Send server info
        let server_info = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": self.name,
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    }
                }
            }
        });
        
        println!("{server_info}");
        
        // Process incoming messages
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    info!("Received message: {}", line.trim());
                    // Process MCP message here
                    if line.trim().is_empty() {
                        continue;
                    }
                    // Echo back for now
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "result": "ok"
                    });
                    println!("{response}");
                }
                Err(e) => {
                    info!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }

    pub fn get_tools(&self) -> &[McpTool] {
        &self.tools
    }

    pub fn get_storage(&self) -> &MemoStorage {
        &self.storage
    }

    pub fn get_storage_mut(&mut self) -> &mut MemoStorage {
        &mut self.storage
    }
}
