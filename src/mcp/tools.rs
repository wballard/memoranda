use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
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
                let mut properties = HashMap::new();
                properties.insert(
                    "title".to_string(),
                    Value::Object({
                        let mut obj = serde_json::Map::new();
                        obj.insert("type".to_string(), Value::String("string".to_string()));
                        obj.insert(
                            "description".to_string(),
                            Value::String("The title of the memo".to_string()),
                        );
                        obj
                    }),
                );
                properties.insert(
                    "content".to_string(),
                    Value::Object({
                        let mut obj = serde_json::Map::new();
                        obj.insert("type".to_string(), Value::String("string".to_string()));
                        obj.insert(
                            "description".to_string(),
                            Value::String("The content of the memo".to_string()),
                        );
                        obj
                    }),
                );

                Value::Object({
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), Value::String("object".to_string()));
                    obj.insert(
                        "properties".to_string(),
                        Value::Object({
                            let mut props = serde_json::Map::new();
                            for (k, v) in properties {
                                props.insert(k, v);
                            }
                            props
                        }),
                    );
                    obj.insert(
                        "required".to_string(),
                        Value::Array(vec![
                            Value::String("title".to_string()),
                            Value::String("content".to_string()),
                        ]),
                    );
                    obj
                })
            }
            "get_memo" => Value::Object({
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), Value::String("object".to_string()));
                obj.insert(
                    "properties".to_string(),
                    Value::Object({
                        let mut props = serde_json::Map::new();
                        props.insert(
                            "id".to_string(),
                            Value::Object({
                                let mut id_obj = serde_json::Map::new();
                                id_obj.insert(
                                    "type".to_string(),
                                    Value::String("string".to_string()),
                                );
                                id_obj.insert(
                                    "description".to_string(),
                                    Value::String("The ID of the memo to retrieve".to_string()),
                                );
                                id_obj
                            }),
                        );
                        props
                    }),
                );
                obj.insert(
                    "required".to_string(),
                    Value::Array(vec![Value::String("id".to_string())]),
                );
                obj
            }),
            "delete_memo" => Value::Object({
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), Value::String("object".to_string()));
                obj.insert(
                    "properties".to_string(),
                    Value::Object({
                        let mut props = serde_json::Map::new();
                        props.insert(
                            "id".to_string(),
                            Value::Object({
                                let mut id_obj = serde_json::Map::new();
                                id_obj.insert(
                                    "type".to_string(),
                                    Value::String("string".to_string()),
                                );
                                id_obj.insert(
                                    "description".to_string(),
                                    Value::String("The ID of the memo to delete".to_string()),
                                );
                                id_obj
                            }),
                        );
                        props
                    }),
                );
                obj.insert(
                    "required".to_string(),
                    Value::Array(vec![Value::String("id".to_string())]),
                );
                obj
            }),
            _ => Value::Object({
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), Value::String("object".to_string()));
                obj
            }),
        };

        ToolDefinition {
            name: self.name.clone(),
            description: Some(self.description.clone()),
            input_schema: schema,
        }
    }

    pub async fn execute(&self, _args: Value) -> Result<String> {
        info!("Executing MCP tool: {}", self.name);
        match self.name.as_str() {
            "create_memo" => Ok("Memo creation tool executed".to_string()),
            "list_memos" => Ok("Memo listing tool executed".to_string()),
            "get_memo" => Ok("Memo retrieval tool executed".to_string()),
            "delete_memo" => Ok("Memo deletion tool executed".to_string()),
            _ => Ok(format!("Tool {} executed successfully", self.name)),
        }
    }
}
