use serde_json::Value;
use tracing::info;

const MEMO_TITLE_MAX_LENGTH: u32 = 255;
const MEMO_CONTENT_MAX_LENGTH: u32 = 1_048_576;
const SEARCH_QUERY_MAX_LENGTH: u32 = 1000;

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
                            "maxLength": MEMO_TITLE_MAX_LENGTH
                        },
                        "content": {
                            "type": "string",
                            "description": "The content of the memo",
                            "maxLength": MEMO_CONTENT_MAX_LENGTH
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
                            "maxLength": MEMO_CONTENT_MAX_LENGTH
                        }
                    },
                    "required": ["id", "content"]
                })
            }
            "list_memos" | "get_all_context" => {
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                })
            }
            "get_memo" | "delete_memo" => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "The ID of the memo",
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
                            "maxLength": SEARCH_QUERY_MAX_LENGTH
                        }
                    },
                    "required": ["query"]
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
}
