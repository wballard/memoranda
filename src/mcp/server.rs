use anyhow::Result;
use std::io::{BufRead, BufReader, Write};
use tokio::signal;
use tracing::{error, info, warn};

use super::tools::McpTool;
use crate::memo::{MemoStorage, MemoStore};

pub struct McpServer {
    pub name: String,
    storage: MemoStorage,
    memo_store: MemoStore,
    tools: Vec<McpTool>,
}

impl McpServer {
    pub fn new(name: String) -> Result<Self> {
        info!("Creating MCP server: {}", name);
        let tools = vec![
            McpTool::new(
                "create_memo".to_string(),
                "Create a new memo with title and content".to_string(),
            ),
            McpTool::new(
                "update_memo".to_string(),
                "Update an existing memo by ID".to_string(),
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
            McpTool::new(
                "search_memos".to_string(),
                "Search memo content by text pattern".to_string(),
            ),
            McpTool::new(
                "get_all_context".to_string(),
                "Combine all memos for LLM context".to_string(),
            ),
        ];

        let memo_store = MemoStore::from_git_root()?;
        Ok(Self {
            name,
            storage: MemoStorage::new(),
            memo_store,
            tools,
        })
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
                        let tool_def = tool.to_tool_definition();
                        serde_json::json!({
                            "name": tool_def.name,
                            "description": tool_def.description,
                            "inputSchema": tool_def.input_schema
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

            "tools/call" => {
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

                info!("Handling tools/call request");

                let params = message.get("params")?;
                let tool_name = params.get("name")?.as_str()?;
                let arguments = params.get("arguments").unwrap_or(&serde_json::Value::Null);

                match self.execute_tool(tool_name, arguments.clone()).await {
                    Ok(result) => Some(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": result
                                }
                            ]
                        }
                    })),
                    Err(e) => {
                        error!("Tool execution failed: {}", e);
                        Some(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32000,
                                "message": format!("Tool execution failed: {}", e)
                            }
                        }))
                    }
                }
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

    #[cfg(test)]
    pub fn new_with_memo_store(name: String, memo_store: MemoStore) -> Self {
        info!("Creating test MCP server: {}", name);
        let tools = vec![
            McpTool::new(
                "create_memo".to_string(),
                "Create a new memo with title and content".to_string(),
            ),
            McpTool::new(
                "update_memo".to_string(),
                "Update an existing memo by ID".to_string(),
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
            McpTool::new(
                "search_memos".to_string(),
                "Search memo content by text pattern".to_string(),
            ),
            McpTool::new(
                "get_all_context".to_string(),
                "Combine all memos for LLM context".to_string(),
            ),
        ];

        Self {
            name,
            storage: MemoStorage::new(),
            memo_store,
            tools,
        }
    }

    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<String> {
        info!("Executing tool: {} with args: {}", tool_name, arguments);

        match tool_name {
            "create_memo" => {
                let title = arguments
                    .get("title")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: title"))?;
                let content = arguments
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;

                let memo = self
                    .memo_store
                    .create_memo(title.to_string(), content.to_string())?;
                Ok(serde_json::to_string_pretty(&memo)?)
            }

            "update_memo" => {
                let id_str = arguments
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: id"))?;
                let content = arguments
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;

                let memo_id = id_str
                    .parse::<ulid::Ulid>()
                    .map_err(|_| anyhow::anyhow!("Invalid memo ID format"))?;
                let memo_id = crate::memo::MemoId::from_ulid(memo_id);

                let memo = self.memo_store.update_memo(&memo_id, content.to_string())?;
                Ok(serde_json::to_string_pretty(&memo)?)
            }

            "list_memos" => {
                let memos = self.memo_store.list_memos()?;
                Ok(serde_json::to_string_pretty(&memos)?)
            }

            "get_memo" => {
                let id_str = arguments
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: id"))?;

                let memo_id = id_str
                    .parse::<ulid::Ulid>()
                    .map_err(|_| anyhow::anyhow!("Invalid memo ID format"))?;
                let memo_id = crate::memo::MemoId::from_ulid(memo_id);

                match self.memo_store.get_memo(&memo_id)? {
                    Some(memo) => Ok(serde_json::to_string_pretty(&memo)?),
                    None => Err(anyhow::anyhow!("Memo not found: {}", id_str)),
                }
            }

            "delete_memo" => {
                let id_str = arguments
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: id"))?;

                let memo_id = id_str
                    .parse::<ulid::Ulid>()
                    .map_err(|_| anyhow::anyhow!("Invalid memo ID format"))?;
                let memo_id = crate::memo::MemoId::from_ulid(memo_id);

                self.memo_store.delete_memo(&memo_id)?;
                Ok(format!("Memo {} deleted successfully", id_str))
            }

            "search_memos" => {
                let query = arguments
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;

                let memos = self.memo_store.list_memos()?;
                let matching_memos: Vec<_> = memos
                    .into_iter()
                    .filter(|memo| {
                        memo.title.to_lowercase().contains(&query.to_lowercase())
                            || memo.content.to_lowercase().contains(&query.to_lowercase())
                    })
                    .collect();

                Ok(serde_json::to_string_pretty(&matching_memos)?)
            }

            "get_all_context" => {
                let memos = self.memo_store.list_memos()?;
                let mut context = String::new();

                for memo in memos {
                    context.push_str(&format!("# {}\n\n{}", memo.title, memo.content));
                    context.push_str("\n\n---\n\n");
                }

                Ok(context)
            }

            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        }
    }
}
