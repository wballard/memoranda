use anyhow::{Context, Result};
use std::io::Write;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::signal;
use tracing::{debug, error, info, span, warn, Level};
use ulid::Ulid;

use super::tools::McpTool;
use crate::error::McpError;
use crate::memo::MemoStore;
use crate::utils::{retry_with_backoff_sync, RetryConfig};

const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

pub struct McpServer {
    pub name: String,
    memo_store: Option<MemoStore>,
    tools: Vec<McpTool>,
}

impl McpServer {
    pub fn new(name: String) -> Result<Self> {
        let _span = span!(Level::INFO, "mcp_server_new", server_name = %name).entered();
        info!(server_name = %name, "Creating MCP server");

        // Try to initialize memo store with retry mechanism
        let memo_store = Self::try_initialize_memo_store();

        let tools = if memo_store.is_some() {
            // Full functionality when memo store is available
            vec![
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
            ]
        } else {
            // Limited functionality when memo store is unavailable
            warn!("MCP server starting with limited functionality - memo store unavailable");
            vec![
                McpTool::new(
                    "server_status".to_string(),
                    "Get server status and available functionality".to_string(),
                ),
                McpTool::new(
                    "retry_memo_store".to_string(),
                    "Attempt to reinitialize the memo store".to_string(),
                ),
            ]
        };

        info!(
            tool_count = tools.len(),
            memo_store_available = memo_store.is_some(),
            "MCP server initialized"
        );

        Ok(Self {
            name,
            memo_store,
            tools,
        })
    }

    /// Try to initialize memo store with retry logic
    fn try_initialize_memo_store() -> Option<MemoStore> {
        let result = retry_with_backoff_sync(
            || MemoStore::from_git_root().map_err(anyhow::Error::from),
            RetryConfig::for_network(), // Use network config for more retries
            "memo_store_initialization",
        );

        match result {
            Ok(store) => {
                info!("Memo store initialized successfully");
                Some(store)
            }
            Err(e) => {
                warn!(error = %e, "Failed to initialize memo store - server will run with limited functionality");
                None
            }
        }
    }

    /// Attempt to reinitialize the memo store
    pub fn retry_memo_store_initialization(&mut self) -> Result<bool> {
        if self.memo_store.is_some() {
            info!("Memo store is already initialized");
            return Ok(true);
        }

        info!("Attempting to reinitialize memo store");

        if let Some(store) = Self::try_initialize_memo_store() {
            self.memo_store = Some(store);

            // Update tools to full functionality
            self.tools = vec![
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

            info!("Memo store successfully reinitialized - full functionality restored");
            Ok(true)
        } else {
            warn!("Failed to reinitialize memo store - continuing with limited functionality");
            Ok(false)
        }
    }

    /// Get server status and available functionality
    pub fn get_server_status(&self) -> serde_json::Value {
        serde_json::json!({
            "server_name": self.name,
            "memo_store_available": self.memo_store.is_some(),
            "available_tools": self.tools.iter().map(|t| t.to_tool_definition().name).collect::<Vec<_>>(),
            "functionality": if self.memo_store.is_some() {
                "full"
            } else {
                "limited"
            },
            "status": "running"
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let _span = span!(Level::INFO, "mcp_server_start", server_name = %self.name).entered();
        info!(server_name = %self.name, "Starting MCP server");

        // Setup signal handling for graceful shutdown
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
            .context("Failed to setup SIGINT handler")
            .map_err(|e| {
                McpError::server_initialization_failed(format!("Signal handling setup failed: {e}"))
            })?;
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .context("Failed to setup SIGTERM handler")
            .map_err(|e| {
                McpError::server_initialization_failed(format!("Signal handling setup failed: {e}"))
            })?;

        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut stdout = std::io::stdout();

        info!(server_name = %self.name, "MCP server listening on stdio");

        let mut initialized = false;
        let mut message_count = 0u64;

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
                result = async {
                    let mut line = String::new();
                    reader.read_line(&mut line).await.map(|n| (n, line))
                } => {
                    match result {
                        Ok((0, _)) => {
                            // EOF
                            break;
                        }
                        Ok((_, line)) => {
                            let line = line.trim();

                            if line.is_empty() {
                                continue;
                            }

                            message_count += 1;
                            let message_id = Ulid::new();
                            let _msg_span = span!(Level::DEBUG, "mcp_message",
                                message_id = %message_id,
                                message_count = message_count
                            ).entered();

                            debug!(message_id = %message_id, raw_message = %line, "Received MCP message");

                            // Parse JSON-RPC message with better error handling
                            match serde_json::from_str::<serde_json::Value>(line) {
                                Ok(message) => {
                                    let start_time = std::time::Instant::now();
                                    let response = self.handle_message_internal(message, &mut initialized).await;
                                    let duration = start_time.elapsed();

                                    debug!(message_id = %message_id, duration_ms = duration.as_millis(), "Message processing completed");

                                    if let Some(response) = response {
                                        if let Err(e) = writeln!(stdout, "{response}") {
                                            error!(message_id = %message_id, error = %e, "Failed to write response to stdout");
                                            break;
                                        }
                                        if let Err(e) = stdout.flush() {
                                            error!(message_id = %message_id, error = %e, "Failed to flush stdout");
                                            break;
                                        }
                                        debug!(message_id = %message_id, "Response sent successfully");
                                    }
                                }
                                Err(e) => {
                                    warn!(message_id = %message_id, error = %e, raw_message = %line, "Failed to parse JSON-RPC message");
                                    let error_response = serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "error": {
                                            "code": -32700,
                                            "message": "Parse error",
                                            "data": {
                                                "details": e.to_string()
                                            }
                                        }
                                    });
                                    if let Err(e) = writeln!(stdout, "{error_response}") {
                                        error!(message_id = %message_id, error = %e, "Failed to write error response");
                                        break;
                                    }
                                    if let Err(e) = stdout.flush() {
                                        error!(message_id = %message_id, error = %e, "Failed to flush stdout after error");
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Error reading from stdin, shutting down server");
                            break;
                        }
                    }
                }
            }
        }

        info!("MCP server shutting down");
        Ok(())
    }

    pub async fn handle_message(
        &mut self,
        message: serde_json::Value,
        initialized: &mut bool,
    ) -> Option<serde_json::Value> {
        self.handle_message_internal(message, initialized).await
    }

    async fn handle_message_internal(
        &mut self,
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
                        "protocolVersion": MCP_PROTOCOL_VERSION,
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
            memo_store: Some(memo_store),
            tools,
        }
    }

    /// Extracts a string parameter from the arguments JSON.
    fn extract_string_param<'a>(
        arguments: &'a serde_json::Value,
        param_name: &str,
    ) -> Result<&'a str> {
        arguments
            .get(param_name)
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: {}", param_name))
    }

    /// Parses a string ID into a MemoId.
    fn parse_memo_id(id_str: &str) -> Result<crate::memo::MemoId> {
        let ulid = id_str
            .parse::<ulid::Ulid>()
            .map_err(|_| anyhow::anyhow!("Invalid memo ID format"))?;
        Ok(crate::memo::MemoId::from_ulid(ulid))
    }

    /// Handles server status tool execution.
    async fn execute_server_status(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.get_server_status())?)
    }

    /// Handles retry memo store tool execution.
    async fn execute_retry_memo_store(&mut self) -> Result<String> {
        let success = self.retry_memo_store_initialization()?;
        Ok(serde_json::to_string_pretty(&serde_json::json!({
            "success": success,
            "message": if success {
                "Memo store successfully reinitialized - full functionality restored"
            } else {
                "Failed to reinitialize memo store - continuing with limited functionality"
            }
        }))?)
    }

    /// Handles create memo tool execution.
    async fn execute_create_memo(
        memo_store: &crate::memo::MemoStore,
        arguments: &serde_json::Value,
    ) -> Result<String> {
        let title = Self::extract_string_param(arguments, "title")?;
        let content = Self::extract_string_param(arguments, "content")?;

        let memo = memo_store.create_memo(title.to_string(), content.to_string())?;
        Ok(serde_json::to_string_pretty(&memo)?)
    }

    /// Handles update memo tool execution.
    async fn execute_update_memo(
        memo_store: &crate::memo::MemoStore,
        arguments: &serde_json::Value,
    ) -> Result<String> {
        let id_str = Self::extract_string_param(arguments, "id")?;
        let content = Self::extract_string_param(arguments, "content")?;

        let memo_id = Self::parse_memo_id(id_str)?;
        let memo = memo_store.update_memo(&memo_id, content.to_string())?;
        Ok(serde_json::to_string_pretty(&memo)?)
    }

    /// Handles list memos tool execution.
    async fn execute_list_memos(memo_store: &crate::memo::MemoStore) -> Result<String> {
        let memos = memo_store.list_memos()?;
        Ok(serde_json::to_string_pretty(&memos)?)
    }

    /// Handles get memo tool execution.
    async fn execute_get_memo(
        memo_store: &crate::memo::MemoStore,
        arguments: &serde_json::Value,
    ) -> Result<String> {
        let id_str = Self::extract_string_param(arguments, "id")?;
        let memo_id = Self::parse_memo_id(id_str)?;

        let memo = memo_store
            .get_memo(&memo_id)?
            .ok_or_else(|| anyhow::anyhow!("Memo not found with ID: {}", memo_id))?;
        Ok(serde_json::to_string_pretty(&memo)?)
    }

    /// Handles delete memo tool execution.
    async fn execute_delete_memo(
        memo_store: &crate::memo::MemoStore,
        arguments: &serde_json::Value,
    ) -> Result<String> {
        let id_str = Self::extract_string_param(arguments, "id")?;
        let memo_id = Self::parse_memo_id(id_str)?;

        memo_store.delete_memo(&memo_id)?;
        Ok(serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "message": "Memo deleted successfully"
        }))?)
    }

    /// Handles search memos tool execution.
    async fn execute_search_memos(
        memo_store: &crate::memo::MemoStore,
        arguments: &serde_json::Value,
    ) -> Result<String> {
        let query = Self::extract_string_param(arguments, "query")?;

        // Simple search implementation like in the original function
        let memos = memo_store.list_memos()?;
        let matching_memos: Vec<_> = memos
            .into_iter()
            .filter(|memo| {
                memo.title.to_lowercase().contains(&query.to_lowercase())
                    || memo.content.to_lowercase().contains(&query.to_lowercase())
            })
            .collect();

        Ok(serde_json::to_string_pretty(&matching_memos)?)
    }

    /// Handles get all context tool execution.
    async fn execute_get_all_context(memo_store: &crate::memo::MemoStore) -> Result<String> {
        let all_memos = memo_store.list_memos()?;

        let context = all_memos
            .into_iter()
            .map(|memo| format!("# {}\n{}", memo.title, memo.content))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        Ok(context)
    }

    pub async fn execute_tool(
        &mut self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<String> {
        info!("Executing tool: {} with args: {}", tool_name, arguments);

        // Handle limited functionality tools first
        match tool_name {
            "server_status" => return self.execute_server_status().await,
            "retry_memo_store" => return self.execute_retry_memo_store().await,
            _ => {}
        }

        // Check if memo store is available for memo operations
        let Some(memo_store) = &self.memo_store else {
            return Err(anyhow::anyhow!(
                "Memo store is not available. Use 'retry_memo_store' to attempt reinitialization or 'server_status' to check server status."
            ));
        };

        // Route to appropriate tool handler
        match tool_name {
            "create_memo" => Self::execute_create_memo(memo_store, &arguments).await,
            "update_memo" => Self::execute_update_memo(memo_store, &arguments).await,
            "list_memos" => Self::execute_list_memos(memo_store).await,
            "get_memo" => Self::execute_get_memo(memo_store, &arguments).await,
            "delete_memo" => Self::execute_delete_memo(memo_store, &arguments).await,
            "search_memos" => Self::execute_search_memos(memo_store, &arguments).await,
            "get_all_context" => Self::execute_get_all_context(memo_store).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        }
    }
}
