use anyhow::Result;
use std::io::{BufRead, BufReader, Write};
use tokio::signal;
use tracing::{error, info, warn};

use super::tools::McpTool;
use crate::memo::MemoStorage;

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
            McpTool::new("delete_memo".to_string(), "Delete a memo by ID".to_string()),
        ];

        Self {
            name,
            storage: MemoStorage::new(),
            tools,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting MCP server: {}", self.name);

        // Setup signal handling for graceful shutdown
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;

        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut stdout = std::io::stdout();

        info!("MCP server listening on stdio: {}", self.name);

        let mut initialized = false;

        // Process incoming messages with signal handling
        loop {
            tokio::select! {
                // Handle SIGINT (Ctrl+C)
                _ = sigint.recv() => {
                    info!("Received SIGINT, shutting down gracefully");
                    break;
                }

                // Handle SIGTERM
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, shutting down gracefully");
                    break;
                }

                // Handle stdin messages
                result = tokio::task::spawn_blocking(move || {
                    let mut line = String::new();
                    reader.read_line(&mut line).map(|n| (n, line, reader))
                }) => {
                    match result {
                        Ok(Ok((0, _, _))) => {
                            // EOF
                            break;
                        }
                        Ok(Ok((_, line, new_reader))) => {
                            reader = new_reader;
                            let line = line.trim();

                            if line.is_empty() {
                                continue;
                            }

                            info!("Received message: {}", line);

                            // Parse JSON-RPC message
                            match serde_json::from_str::<serde_json::Value>(line) {
                                Ok(message) => {
                                    let response = self.handle_message(message, &mut initialized).await;
                                    if let Some(response) = response {
                                        if let Err(e) = writeln!(stdout, "{response}") {
                                            error!("Failed to write response: {}", e);
                                            break;
                                        }
                                        if let Err(e) = stdout.flush() {
                                            error!("Failed to flush stdout: {}", e);
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Invalid JSON received: {}", e);
                                    let error_response = serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "error": {
                                            "code": -32700,
                                            "message": "Parse error"
                                        }
                                    });
                                    if let Err(e) = writeln!(stdout, "{error_response}") {
                                        error!("Failed to write error response: {}", e);
                                        break;
                                    }
                                    if let Err(e) = stdout.flush() {
                                        error!("Failed to flush stdout: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            error!("Error reading from stdin: {}", e);
                            break;
                        }
                        Err(e) => {
                            error!("Task error: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        info!("MCP server shutting down");
        Ok(())
    }

    async fn handle_message(
        &self,
        message: serde_json::Value,
        initialized: &mut bool,
    ) -> Option<serde_json::Value> {
        let method = message.get("method")?.as_str()?;
        let id = message.get("id");

        match method {
            "initialize" => {
                *initialized = true;
                info!("Handling initialize request");

                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
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

                Some(response)
            }

            "tools/list" => {
                if !*initialized {
                    return Some(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32002,
                            "message": "Server not initialized"
                        }
                    }));
                }

                info!("Handling tools/list request");

                let tools: Vec<serde_json::Value> = self
                    .tools
                    .iter()
                    .map(|tool| {
                        serde_json::json!({
                            "name": tool.name,
                            "description": tool.description,
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        })
                    })
                    .collect();

                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": tools
                    }
                }))
            }

            _ => {
                info!("Unhandled method: {}", method);
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    }
                }))
            }
        }
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
