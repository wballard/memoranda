#[cfg(test)]
mod unit_tests {
    use super::super::*;
    use crate::memo::{Memo, MemoStore};
    use anyhow::Result;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    // Helper function to create a test MCP server with a temporary directory
    fn create_test_server() -> Result<(McpServer, TempDir)> {
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

    #[tokio::test]
    async fn test_create_memo_tool() -> Result<()> {
        let (server, _temp_dir) = create_test_server()?;

        let args = json!({
            "title": "Test Memo",
            "content": "This is a test memo content"
        });

        let result = server.execute_tool("create_memo", args).await?;

        // Verify the result contains the memo data
        assert!(result.contains("Test Memo"));
        assert!(result.contains("This is a test memo content"));

        Ok(())
    }

    #[tokio::test]
    async fn test_list_memos_tool() -> Result<()> {
        let (server, _temp_dir) = create_test_server()?;

        // Create a memo first
        let create_args = json!({
            "title": "Test Memo 1",
            "content": "Content 1"
        });
        server.execute_tool("create_memo", create_args).await?;

        // Create another memo
        let create_args2 = json!({
            "title": "Test Memo 2",
            "content": "Content 2"
        });
        server.execute_tool("create_memo", create_args2).await?;

        // List all memos
        let result = server.execute_tool("list_memos", json!({})).await?;

        // Verify both memos are listed
        assert!(result.contains("Test Memo 1"));
        assert!(result.contains("Test Memo 2"));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_memo_tool() -> Result<()> {
        let (server, _temp_dir) = create_test_server()?;

        // Create a memo first
        let create_args = json!({
            "title": "Retrievable Memo",
            "content": "This memo can be retrieved"
        });
        let create_result = server.execute_tool("create_memo", create_args).await?;

        // Extract the memo ID from the create result
        let memo: Memo = serde_json::from_str(&create_result)?;

        // Get the memo by ID
        let get_args = json!({
            "id": memo.id.to_string()
        });
        let result = server.execute_tool("get_memo", get_args).await?;

        // Verify the memo was retrieved
        assert!(result.contains("Retrievable Memo"));
        assert!(result.contains("This memo can be retrieved"));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_memo_tool() -> Result<()> {
        let (server, _temp_dir) = create_test_server()?;

        // Create a memo first
        let create_args = json!({
            "title": "Updatable Memo",
            "content": "Original content"
        });
        let create_result = server.execute_tool("create_memo", create_args).await?;

        // Extract the memo ID from the create result
        let memo: Memo = serde_json::from_str(&create_result)?;

        // Update the memo
        let update_args = json!({
            "id": memo.id.to_string(),
            "content": "Updated content"
        });
        let result = server.execute_tool("update_memo", update_args).await?;

        // Verify the memo was updated
        assert!(result.contains("Updated content"));
        assert!(!result.contains("Original content"));

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_memo_tool() -> Result<()> {
        let (server, _temp_dir) = create_test_server()?;

        // Create a memo first
        let create_args = json!({
            "title": "Deletable Memo",
            "content": "This memo will be deleted"
        });
        let create_result = server.execute_tool("create_memo", create_args).await?;

        // Extract the memo ID from the create result
        let memo: Memo = serde_json::from_str(&create_result)?;

        // Delete the memo
        let delete_args = json!({
            "id": memo.id.to_string()
        });
        let result = server.execute_tool("delete_memo", delete_args).await?;

        // Verify the memo was deleted
        assert!(result.contains("deleted successfully"));

        // Verify the memo is no longer retrievable
        let get_args = json!({
            "id": memo.id.to_string()
        });
        let get_result = server.execute_tool("get_memo", get_args).await;
        assert!(get_result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_search_memos_tool() -> Result<()> {
        let (server, _temp_dir) = create_test_server()?;

        // Create test memos
        let create_args1 = json!({
            "title": "Rust Programming",
            "content": "This is about Rust programming language"
        });
        server.execute_tool("create_memo", create_args1).await?;

        let create_args2 = json!({
            "title": "Python Notes",
            "content": "This is about Python programming"
        });
        server.execute_tool("create_memo", create_args2).await?;

        // Search for memos containing "Rust"
        let search_args = json!({
            "query": "Rust"
        });
        let result = server.execute_tool("search_memos", search_args).await?;

        // Verify only the Rust memo is returned
        assert!(result.contains("Rust Programming"));
        assert!(!result.contains("Python Notes"));

        // Search for memos containing "programming"
        let search_args2 = json!({
            "query": "programming"
        });
        let result2 = server.execute_tool("search_memos", search_args2).await?;

        // Verify both memos are returned since both contain "programming"
        assert!(result2.contains("Rust Programming"));
        assert!(result2.contains("Python Notes"));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_all_context_tool() -> Result<()> {
        let (server, _temp_dir) = create_test_server()?;

        // Create test memos
        let create_args1 = json!({
            "title": "First Memo",
            "content": "Content of the first memo"
        });
        server.execute_tool("create_memo", create_args1).await?;

        let create_args2 = json!({
            "title": "Second Memo",
            "content": "Content of the second memo"
        });
        server.execute_tool("create_memo", create_args2).await?;

        // Get all context
        let result = server.execute_tool("get_all_context", json!({})).await?;

        // Verify both memos are included in the context
        assert!(result.contains("# First Memo"));
        assert!(result.contains("Content of the first memo"));
        assert!(result.contains("# Second Memo"));
        assert!(result.contains("Content of the second memo"));
        assert!(result.contains("---"));

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_tool_name() -> Result<()> {
        let (server, _temp_dir) = create_test_server()?;

        let result = server.execute_tool("nonexistent_tool", json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown tool"));

        Ok(())
    }

    #[tokio::test]
    async fn test_missing_required_parameters() -> Result<()> {
        let (server, _temp_dir) = create_test_server()?;

        // Test create_memo without title
        let result = server
            .execute_tool("create_memo", json!({"content": "test"}))
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing required parameter: title")
        );

        // Test create_memo without content
        let result = server
            .execute_tool("create_memo", json!({"title": "test"}))
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing required parameter: content")
        );

        // Test get_memo without id
        let result = server.execute_tool("get_memo", json!({})).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing required parameter: id")
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_memo_id() -> Result<()> {
        let (server, _temp_dir) = create_test_server()?;

        // Test with invalid ULID format
        let result = server
            .execute_tool("get_memo", json!({"id": "invalid-id"}))
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid memo ID format")
        );

        Ok(())
    }

    #[test]
    fn test_tool_schemas() {
        let tools = vec![
            McpTool::new("create_memo".to_string(), "Create memo".to_string()),
            McpTool::new("update_memo".to_string(), "Update memo".to_string()),
            McpTool::new("list_memos".to_string(), "List memos".to_string()),
            McpTool::new("get_memo".to_string(), "Get memo".to_string()),
            McpTool::new("delete_memo".to_string(), "Delete memo".to_string()),
            McpTool::new("search_memos".to_string(), "Search memos".to_string()),
            McpTool::new("get_all_context".to_string(), "Get all context".to_string()),
        ];

        for tool in tools {
            let def = tool.to_tool_definition();
            assert!(!def.name.is_empty());
            assert!(def.description.is_some());
            assert!(def.input_schema.is_object());

            // Verify the schema has proper structure
            let schema = def.input_schema.as_object().unwrap();
            assert_eq!(schema.get("type").unwrap().as_str().unwrap(), "object");
            assert!(schema.contains_key("properties"));
            assert!(schema.contains_key("required"));
        }
    }
}
