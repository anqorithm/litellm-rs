//! Tool types for function calling

use serde::{Deserialize, Serialize};

/// Tool type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    /// Function definition
    pub function: FunctionDefinition,
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// Function description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Parameter JSON Schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Tool selection strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// String selection ("auto", "none")
    String(String),
    /// Specific tool selection
    Specific {
        #[serde(rename = "type")]
        choice_type: String,
        function: Option<FunctionChoice>,
    },
}

/// Specific function selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChoice {
    pub name: String,
}

/// Tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Call ID
    pub id: String,
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function call details
    pub function: FunctionCall,
}

/// Function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    /// Function parameters (JSON string)
    pub arguments: String,
}

/// Function call selection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCallChoice {
    /// Do not call function
    None,
    /// Auto decide
    Auto,
    /// Specify function
    Function {
        /// Function name
        name: String,
    },
}

/// Response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// Format type ("text", "json_object", "json_schema")
    #[serde(rename = "type")]
    pub format_type: String,
    /// JSON Schema (when type is json_schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<serde_json::Value>,
    /// Response type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_type: Option<String>,
}

/// Anthropic server-side built-in tool type identifiers.
///
/// These represent tools hosted by Anthropic that do not require
/// a client-side function definition (no `input_schema` needed).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnthropicBuiltinToolType {
    /// Web search tool (released 2025-03-05)
    #[serde(rename = "web_search_20250305")]
    WebSearch,
    /// Computer use tool (released 2025-01-24)
    #[serde(rename = "computer_20250124")]
    ComputerUse,
}

/// An Anthropic server-side (built-in) tool that is executed by Anthropic
/// without a client-provided function implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicBuiltinTool {
    /// Tool type identifier
    #[serde(rename = "type")]
    pub tool_type: AnthropicBuiltinToolType,
    /// Tool name used when referencing the tool in messages
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_type_serialization() {
        let tool_type = ToolType::Function;
        let json = serde_json::to_string(&tool_type).unwrap();
        assert_eq!(json, "\"function\"");
    }

    #[test]
    fn test_tool_definition() {
        let tool = Tool {
            tool_type: ToolType::Function,
            function: FunctionDefinition {
                name: "get_weather".to_string(),
                description: Some("Get current weather".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    }
                })),
            },
        };

        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["type"], "function");
        assert_eq!(json["function"]["name"], "get_weather");
    }

    #[test]
    fn test_tool_choice_string() {
        let choice = ToolChoice::String("auto".to_string());
        let json = serde_json::to_string(&choice).unwrap();
        assert_eq!(json, "\"auto\"");
    }

    #[test]
    fn test_tool_choice_specific() {
        let choice = ToolChoice::Specific {
            choice_type: "function".to_string(),
            function: Some(FunctionChoice {
                name: "my_function".to_string(),
            }),
        };

        let json = serde_json::to_value(&choice).unwrap();
        assert_eq!(json["type"], "function");
        assert_eq!(json["function"]["name"], "my_function");
    }

    #[test]
    fn test_tool_call() {
        let call = ToolCall {
            id: "call_123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "get_weather".to_string(),
                arguments: "{\"location\": \"NYC\"}".to_string(),
            },
        };

        let json = serde_json::to_value(&call).unwrap();
        assert_eq!(json["id"], "call_123");
        assert_eq!(json["type"], "function");
        assert_eq!(json["function"]["name"], "get_weather");
    }

    #[test]
    fn test_function_call() {
        let call = FunctionCall {
            name: "calculate".to_string(),
            arguments: "{\"x\": 1, \"y\": 2}".to_string(),
        };

        assert_eq!(call.name, "calculate");
        assert!(call.arguments.contains("x"));
    }

    #[test]
    fn test_response_format() {
        let format = ResponseFormat {
            format_type: "json_object".to_string(),
            json_schema: None,
            response_type: None,
        };

        let json = serde_json::to_value(&format).unwrap();
        assert_eq!(json["type"], "json_object");
    }
}
