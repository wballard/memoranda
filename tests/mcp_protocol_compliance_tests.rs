//! MCP Protocol Compliance Tests
//!
//! These tests verify that the MCP server implementation properly adheres to the
//! Model Context Protocol (MCP) specification, including JSON-RPC 2.0 compliance,
//! proper initialization handshake, tool schema validation, and error handling.

use memoranda::mcp::server::McpServer;
use memoranda::memo::MemoStore;
use serde_json::{json, Value};
use std::fs;
use tempfile::TempDir;

/// Helper function to create a test MCP server with a temporary directory
fn create_test_server() -> anyhow::Result<(McpServer, TempDir)> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create a .memoranda directory
    let memoranda_dir = temp_path.join(".memoranda");
    fs::create_dir(&memoranda_dir)?;

    // Create a git directory to satisfy the git_root requirement
    let git_dir = temp_path.join(".git");
    fs::create_dir(&git_dir)?;

    // Create a server with a custom MemoStore that points to our temp directory
    let server = McpServer::new_with_memo_store(
        "test-server".to_string(),
        MemoStore::new(temp_path.to_path_buf()),
    );

    Ok((server, temp_dir))
}

/// Test JSON-RPC 2.0 Protocol Compliance
#[tokio::test]
async fn test_jsonrpc_protocol_compliance() -> anyhow::Result<()> {
    let (mut server, _temp_dir) = create_test_server()?;

    // Test initialize response format
    let initialize_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let mut initialized = false;
    let response = server
        .handle_message(initialize_msg, &mut initialized)
        .await;

    assert!(response.is_some());
    let response = response.unwrap();

    // Verify JSON-RPC 2.0 compliance
    assert_eq!(response.get("jsonrpc").unwrap().as_str().unwrap(), "2.0");
    assert_eq!(response.get("id").unwrap().as_i64().unwrap(), 1);
    assert!(response.get("result").is_some());
    assert!(response.get("error").is_none());

    // Verify result structure
    let result = response.get("result").unwrap();
    assert!(result.get("protocolVersion").is_some());
    assert!(result.get("serverInfo").is_some());
    assert!(result.get("capabilities").is_some());

    Ok(())
}

/// Test MCP Initialization Protocol
#[tokio::test]
async fn test_mcp_initialization_protocol() -> anyhow::Result<()> {
    let (mut server, _temp_dir) = create_test_server()?;

    // Test initialization without being initialized
    let tools_list_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    let mut initialized = false;
    let response = server
        .handle_message(tools_list_msg, &mut initialized)
        .await;

    assert!(response.is_some());
    let response = response.unwrap();

    // Should return an error since not initialized
    assert!(response.get("error").is_some());
    let error = response.get("error").unwrap();
    assert_eq!(error.get("code").unwrap().as_i64().unwrap(), -32002);
    assert_eq!(
        error.get("message").unwrap().as_str().unwrap(),
        "Server not initialized"
    );

    // Test proper initialization
    let initialize_msg = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let response = server
        .handle_message(initialize_msg, &mut initialized)
        .await;
    assert!(response.is_some());
    assert!(initialized);

    // Now tools/list should work
    let tools_list_msg = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/list",
        "params": {}
    });

    let response = server
        .handle_message(tools_list_msg, &mut initialized)
        .await;
    assert!(response.is_some());
    let response = response.unwrap();

    // Should return success
    assert!(response.get("result").is_some());
    assert!(response.get("error").is_none());

    Ok(())
}

/// Test Tool Discovery and Schema Validation
#[tokio::test]
async fn test_tool_discovery_and_schema_validation() -> anyhow::Result<()> {
    let (mut server, _temp_dir) = create_test_server()?;

    // Initialize the server
    let initialize_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let mut initialized = false;
    let _response = server
        .handle_message(initialize_msg, &mut initialized)
        .await;

    // Test tools/list
    let tools_list_msg = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let response = server
        .handle_message(tools_list_msg, &mut initialized)
        .await;
    assert!(response.is_some());
    let response = response.unwrap();

    // Verify response structure
    assert_eq!(response.get("jsonrpc").unwrap().as_str().unwrap(), "2.0");
    assert_eq!(response.get("id").unwrap().as_i64().unwrap(), 2);

    let result = response.get("result").unwrap();
    let tools = result.get("tools").unwrap().as_array().unwrap();

    // Should have all expected tools
    assert_eq!(tools.len(), 7);

    // Verify each tool has proper schema
    let expected_tools = [
        "create_memo",
        "update_memo",
        "list_memos",
        "get_memo",
        "delete_memo",
        "search_memos",
        "get_all_context",
    ];

    for tool in tools {
        let tool_obj = tool.as_object().unwrap();

        // Verify required fields
        assert!(tool_obj.contains_key("name"));
        assert!(tool_obj.contains_key("description"));
        assert!(tool_obj.contains_key("inputSchema"));

        let name = tool_obj.get("name").unwrap().as_str().unwrap();
        assert!(expected_tools.contains(&name));

        // Verify inputSchema is a proper JSON schema
        let input_schema = tool_obj.get("inputSchema").unwrap().as_object().unwrap();
        assert_eq!(
            input_schema.get("type").unwrap().as_str().unwrap(),
            "object"
        );
        assert!(input_schema.contains_key("properties"));
        assert!(input_schema.contains_key("required"));

        // Verify specific tool schemas
        match name {
            "create_memo" => {
                let required = input_schema.get("required").unwrap().as_array().unwrap();
                assert!(required.contains(&json!("title")));
                assert!(required.contains(&json!("content")));
            }
            "get_memo" | "update_memo" | "delete_memo" => {
                let required = input_schema.get("required").unwrap().as_array().unwrap();
                assert!(required.contains(&json!("id")));
            }
            "search_memos" => {
                let required = input_schema.get("required").unwrap().as_array().unwrap();
                assert!(required.contains(&json!("query")));
            }
            "list_memos" | "get_all_context" => {
                // These tools don't require parameters
                let required = input_schema.get("required").unwrap().as_array().unwrap();
                assert!(required.is_empty());
            }
            _ => {}
        }
    }

    Ok(())
}

/// Test Error Handling According to MCP Spec
#[tokio::test]
async fn test_mcp_error_handling() -> anyhow::Result<()> {
    let (mut server, _temp_dir) = create_test_server()?;

    // Test parse error (invalid JSON)
    let invalid_json = r#"{"jsonrpc": "2.0", "id": 1, "method": "invalid"#;
    let parse_result = serde_json::from_str::<Value>(invalid_json);
    assert!(parse_result.is_err());

    // Test method not found error
    let invalid_method_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "invalid_method",
        "params": {}
    });

    let mut initialized = true;
    let response = server
        .handle_message(invalid_method_msg, &mut initialized)
        .await;
    assert!(response.is_some());
    let response = response.unwrap();

    // Should return method not found error
    assert!(response.get("error").is_some());
    let error = response.get("error").unwrap();
    assert_eq!(error.get("code").unwrap().as_i64().unwrap(), -32601);
    assert_eq!(
        error.get("message").unwrap().as_str().unwrap(),
        "Method not found"
    );

    // Test initialization required error
    let mut initialized = false;
    let tools_call_msg = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "create_memo",
            "arguments": {
                "title": "Test",
                "content": "Test content"
            }
        }
    });

    let response = server
        .handle_message(tools_call_msg, &mut initialized)
        .await;
    assert!(response.is_some());
    let response = response.unwrap();

    // Should return not initialized error
    assert!(response.get("error").is_some());
    let error = response.get("error").unwrap();
    assert_eq!(error.get("code").unwrap().as_i64().unwrap(), -32002);
    assert_eq!(
        error.get("message").unwrap().as_str().unwrap(),
        "Server not initialized"
    );

    Ok(())
}

/// Test Tool Execution Protocol Compliance
#[tokio::test]
async fn test_tool_execution_protocol_compliance() -> anyhow::Result<()> {
    let (mut server, _temp_dir) = create_test_server()?;

    // Initialize the server
    let initialize_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let mut initialized = false;
    let _response = server
        .handle_message(initialize_msg, &mut initialized)
        .await;

    // Test successful tool execution
    let tools_call_msg = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "create_memo",
            "arguments": {
                "title": "Test Memo",
                "content": "Test content"
            }
        }
    });

    let response = server
        .handle_message(tools_call_msg, &mut initialized)
        .await;
    assert!(response.is_some());
    let response = response.unwrap();

    // Verify response format
    assert_eq!(response.get("jsonrpc").unwrap().as_str().unwrap(), "2.0");
    assert_eq!(response.get("id").unwrap().as_i64().unwrap(), 2);
    assert!(response.get("result").is_some());

    let result = response.get("result").unwrap();
    assert!(result.get("content").is_some());
    let content = result.get("content").unwrap().as_array().unwrap();
    assert_eq!(content.len(), 1);

    let content_item = content[0].as_object().unwrap();
    assert_eq!(content_item.get("type").unwrap().as_str().unwrap(), "text");
    assert!(content_item.get("text").is_some());

    // Test tool execution with invalid parameters
    let invalid_params_msg = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "create_memo",
            "arguments": {
                "title": "Test"
                // Missing required "content" parameter
            }
        }
    });

    let response = server
        .handle_message(invalid_params_msg, &mut initialized)
        .await;
    assert!(response.is_some());
    let response = response.unwrap();

    // Should return error
    assert!(response.get("error").is_some());
    let error = response.get("error").unwrap();
    assert_eq!(error.get("code").unwrap().as_i64().unwrap(), -32000);
    assert!(error
        .get("message")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("Tool execution failed"));

    Ok(())
}

/// Test Protocol Version Compliance
#[tokio::test]
async fn test_protocol_version_compliance() -> anyhow::Result<()> {
    let (mut server, _temp_dir) = create_test_server()?;

    let initialize_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let mut initialized = false;
    let response = server
        .handle_message(initialize_msg, &mut initialized)
        .await;

    assert!(response.is_some());
    let response = response.unwrap();

    let result = response.get("result").unwrap();
    assert_eq!(
        result.get("protocolVersion").unwrap().as_str().unwrap(),
        "2024-11-05"
    );

    // Verify server info
    let server_info = result.get("serverInfo").unwrap();
    assert_eq!(
        server_info.get("name").unwrap().as_str().unwrap(),
        "test-server"
    );
    assert!(server_info.get("version").is_some());

    // Verify capabilities
    let capabilities = result.get("capabilities").unwrap();
    assert!(capabilities.get("tools").is_some());

    let tools_capability = capabilities.get("tools").unwrap();
    assert!(tools_capability
        .get("listChanged")
        .unwrap()
        .as_bool()
        .unwrap());

    Ok(())
}

/// Test Message Format Validation
#[tokio::test]
async fn test_message_format_validation() -> anyhow::Result<()> {
    let (mut server, _temp_dir) = create_test_server()?;

    // Test message without method
    let invalid_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "params": {}
    });

    let mut initialized = false;
    let response = server.handle_message(invalid_msg, &mut initialized).await;

    // Should return None for messages without method
    assert!(response.is_none());

    // Test message without ID (notification)
    let notification_msg = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {}
    });

    let response = server
        .handle_message(notification_msg, &mut initialized)
        .await;

    // The server still processes the notification and returns a response
    // but in a real server, notification responses would not be sent
    assert!(response.is_some());

    Ok(())
}

/// Test Concurrent Message Handling
#[tokio::test]
async fn test_concurrent_message_handling() -> anyhow::Result<()> {
    let (mut server, _temp_dir) = create_test_server()?;

    // Initialize the server
    let initialize_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let mut initialized = false;
    let _response = server
        .handle_message(initialize_msg, &mut initialized)
        .await;

    // Test sequential tool calls to simulate concurrent behavior
    // (We can't test true concurrency due to borrowing restrictions)
    for i in 0..10 {
        let tools_call_msg = json!({
            "jsonrpc": "2.0",
            "id": i + 2,
            "method": "tools/call",
            "params": {
                "name": "create_memo",
                "arguments": {
                    "title": format!("Test Memo {}", i),
                    "content": format!("Test content {}", i)
                }
            }
        });

        let mut initialized = true;
        let response = server
            .handle_message(tools_call_msg, &mut initialized)
            .await;

        assert!(response.is_some());
        let response = response.unwrap();
        assert!(response.get("result").is_some());
        assert!(response.get("error").is_none());
    }

    Ok(())
}
