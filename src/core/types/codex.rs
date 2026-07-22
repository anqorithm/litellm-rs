//! Codex Responses wire types and fail-closed decoding.

use serde::de::{DeserializeOwned, Error as DeError};
use serde::ser::Error as SerError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use thiserror::Error;

use crate::core::models::openai::responses_api::{
    ResponseFunctionDefinition, ResponseFunctionTool, ResponseInputItem, ResponseTool,
};

/// Codex protocol revision used by the compatibility fixtures for GH-1107.
pub const CODEX_PROTOCOL_BASELINE: &str = "6e5a2d6b8d148a5554fdceb6f399ca45bd1c78d9";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexFunctionCall {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub call_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    pub arguments: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexFunctionCallOutput {
    pub call_id: String,
    pub output: CodexToolOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexCustomToolCall {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub call_id: String,
    pub name: String,
    pub input: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexCustomToolCallOutput {
    pub call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub output: CodexToolOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CodexToolOutput {
    Text(String),
    ContentItems(Vec<CodexToolOutputContent>),
}

impl CodexToolOutput {
    pub fn to_chat_text(&self) -> Result<String, CodexCompatibilityError> {
        match self {
            Self::Text(text) => Ok(text.clone()),
            Self::ContentItems(items) => {
                let mut text = Vec::new();
                for item in items {
                    match item {
                        CodexToolOutputContent::InputText { text: item } => {
                            text.push(item.as_str())
                        }
                        CodexToolOutputContent::InputImage { .. }
                        | CodexToolOutputContent::EncryptedContent { .. } => {
                            return Err(CodexCompatibilityError::UnsupportedFeature {
                                feature: "structured_tool_output".to_string(),
                            });
                        }
                    }
                }
                Ok(text.join("\n"))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CodexToolOutputContent {
    InputText {
        text: String,
    },
    InputImage {
        image_url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    EncryptedContent {
        encrypted_content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexCustomTool {
    pub name: String,
    pub description: String,
    pub format: Value,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CodexCompatibilityError {
    #[error("unsupported Codex feature: {feature}")]
    UnsupportedFeature { feature: String },
}

impl ResponseInputItem {
    pub fn feature_name(&self) -> &str {
        match self {
            Self::Message(_) => "message",
            Self::FunctionCall(_) => "function_call",
            Self::FunctionCallOutput(_) => "function_call_output",
            Self::CustomToolCall(_) => "custom_tool_call",
            Self::CustomToolCallOutput(_) => "custom_tool_call_output",
            Self::Unsupported { item_type, .. } => item_type,
        }
    }
}

impl Serialize for ResponseInputItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Self::Message(value) => tagged_value("message", value),
            Self::FunctionCall(value) => tagged_value("function_call", value),
            Self::FunctionCallOutput(value) => tagged_value("function_call_output", value),
            Self::CustomToolCall(value) => tagged_value("custom_tool_call", value),
            Self::CustomToolCallOutput(value) => tagged_value("custom_tool_call_output", value),
            Self::Unsupported { item_type, payload } => {
                Ok(tagged_payload(item_type, payload.clone()))
            }
        }
        .map_err(S::Error::custom)?;
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ResponseInputItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (item_type, payload) = deserialize_tagged(deserializer)?;
        match item_type.as_str() {
            "message" => decode(payload).map(Self::Message),
            "function_call" => decode(payload).map(Self::FunctionCall),
            "function_call_output" => decode(payload).map(Self::FunctionCallOutput),
            "custom_tool_call" => decode(payload).map(Self::CustomToolCall),
            "custom_tool_call_output" => decode(payload).map(Self::CustomToolCallOutput),
            _ => Ok(Self::Unsupported { item_type, payload }),
        }
        .map_err(D::Error::custom)
    }
}

impl Serialize for ResponseTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Self::WebSearch(value) => tagged_value("web_search", value),
            Self::WebSearchPreview(value) => tagged_value("web_search_preview", value),
            Self::FileSearch(value) => tagged_value("file_search", value),
            Self::CodeInterpreter(value) => tagged_value("code_interpreter", value),
            Self::ComputerUsePreview(value) => tagged_value("computer_use_preview", value),
            Self::Mcp(value) => tagged_value("mcp", value),
            Self::Function(value) => tagged_value("function", value),
            Self::Custom(value) => tagged_value("custom", value),
            Self::Unsupported { tool_type, payload } => {
                Ok(tagged_payload(tool_type, payload.clone()))
            }
        }
        .map_err(S::Error::custom)?;
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ResponseTool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (tool_type, payload) = deserialize_tagged(deserializer)?;
        let decoded = match tool_type.as_str() {
            "web_search" => decode(payload).map(Self::WebSearch),
            "web_search_preview" => decode(payload).map(Self::WebSearchPreview),
            "file_search" => decode(payload).map(Self::FileSearch),
            "code_interpreter" => decode(payload).map(Self::CodeInterpreter),
            "computer_use_preview" => decode(payload).map(Self::ComputerUsePreview),
            "mcp" => decode(payload).map(Self::Mcp),
            "function" => decode_function_tool(payload).map(Self::Function),
            "custom" => decode(payload).map(Self::Custom),
            _ => Ok(Self::Unsupported { tool_type, payload }),
        };
        decoded.map_err(D::Error::custom)
    }
}

fn tagged_value<T: Serialize>(item_type: &str, value: &T) -> Result<Value, serde_json::Error> {
    let Value::Object(payload) = serde_json::to_value(value)? else {
        unreachable!("Codex wire payload structs serialize as objects");
    };
    Ok(tagged_payload(item_type, payload))
}

fn tagged_payload(item_type: &str, mut payload: serde_json::Map<String, Value>) -> Value {
    payload.insert("type".to_string(), Value::String(item_type.to_string()));
    Value::Object(payload)
}

fn deserialize_tagged<'de, D>(
    deserializer: D,
) -> Result<(String, serde_json::Map<String, Value>), D::Error>
where
    D: Deserializer<'de>,
{
    let Value::Object(mut payload) = Value::deserialize(deserializer)? else {
        return Err(D::Error::custom("Codex item must be an object"));
    };
    let item_type = payload
        .remove("type")
        .and_then(|value| value.as_str().map(str::to_string))
        .ok_or_else(|| D::Error::custom("Codex item type must be a non-empty string"))?;
    if item_type.trim().is_empty() {
        return Err(D::Error::custom(
            "Codex item type must be a non-empty string",
        ));
    }
    Ok((item_type, payload))
}

fn decode<T: DeserializeOwned>(
    payload: serde_json::Map<String, Value>,
) -> Result<T, serde_json::Error> {
    serde_json::from_value(Value::Object(payload))
}

fn decode_function_tool(
    payload: serde_json::Map<String, Value>,
) -> Result<ResponseFunctionTool, serde_json::Error> {
    if payload.contains_key("function") {
        decode(payload)
    } else {
        decode::<ResponseFunctionDefinition>(payload)
            .map(|function| ResponseFunctionTool { function })
    }
}
