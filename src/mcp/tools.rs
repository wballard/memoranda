use anyhow::Result;
use serde_json::Value;
use tracing::info;

#[derive(Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

impl McpTool {
    pub fn new(name: String, description: String) -> Self {
        info!("Creating MCP tool: {}", name);
        Self { name, description }
    }

    pub fn to_tool_definition(&self) -> ToolDefinition {
        let schema = match self.name.as_str() {
            "create_memo" => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "The title of the memo",
                            "minLength": 1,
                            "maxLength": 255
                        },
                        "content": {
                            "type": "string",
                            "description": "The content of the memo",
                            "maxLength": 1048576
                        }
                    },
                    "required": ["title", "content"]
                })
            }
            "update_memo" => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "The ID of the memo to update",
                            "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$"
                        },
                        "content": {
                            "type": "string",
                            "description": "The new content of the memo",
                            "maxLength": 1048576
                        }
                    },
                    "required": ["id", "content"]
                })
            }
            "list_memos" => {
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                })
            }
            "get_memo" => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "The ID of the memo to retrieve",
                            "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$"
                        }
                    },
                    "required": ["id"]
                })
            }
            "delete_memo" => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "The ID of the memo to delete",
                            "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$"
                        }
                    },
                    "required": ["id"]
                })
            }
            "search_memos" => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query to match against memo titles and content",
                            "minLength": 1,
                            "maxLength": 1000
                        }
                    },
                    "required": ["query"]
                })
            }
            "get_all_context" => {
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                })
            }
            _ => {
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                })
            }
        };

        ToolDefinition {
            name: self.name.clone(),
            description: Some(self.description.clone()),
            input_schema: schema,
        }
    }

    pub async fn execute(&self, _args: Value) -> Result<String> {
        info!("Executing MCP tool: {}", self.name);
        // Tool execution is now handled by the server's execute_tool method
        Ok(format!("Tool {} executed successfully", self.name))
    }
}
