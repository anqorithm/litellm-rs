//! Converse API Implementation
//!
//! Modern unified API for chat completions in Bedrock

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::ChatRequest;
use crate::core::types::{MessageContent, MessageRole};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Converse API request format
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConverseRequest {
    pub messages: Vec<ConverseMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Vec<SystemMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inference_config: Option<InferenceConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guardrail_config: Option<GuardrailConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_model_request_fields: Option<Value>,
}

/// Converse message format
#[derive(Debug, Serialize, Deserialize)]
pub struct ConverseMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

/// System message format
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guardrail_content: Option<GuardrailContent>,
}

/// Content block for messages
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentBlock {
    Text { text: String },
    Image { image: ImageBlock },
    Document { document: DocumentBlock },
    ToolUse { tool_use: ToolUseBlock },
    ToolResult { tool_result: ToolResultBlock },
    GuardrailContent { guardrail_content: GuardrailContent },
}

/// Image block for multimodal input
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageBlock {
    pub format: String,
    pub source: ImageSource,
}

/// Image source
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageSource {
    Bytes { bytes: String },
}

/// Document block for document input
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentBlock {
    pub format: String,
    pub name: String,
    pub source: DocumentSource,
}

/// Document source
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DocumentSource {
    Bytes { bytes: String },
}

/// Tool use block
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolUseBlock {
    pub tool_use_id: String,
    pub name: String,
    pub input: Value,
}

/// Tool result block
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResultBlock {
    pub tool_use_id: String,
    pub content: Vec<ToolResultContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Tool result content
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolResultContent {
    Text { text: String },
    Image { image: ImageBlock },
    Document { document: DocumentBlock },
}

/// Guardrail content
#[derive(Debug, Serialize, Deserialize)]
pub struct GuardrailContent {
    pub text: String,
}

/// Inference configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InferenceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

/// Tool configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolConfig {
    pub tools: Vec<ToolSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

/// Tool specification
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSpec {
    pub tool_spec: ToolSpecDefinition,
}

/// Tool specification definition
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSpecDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: InputSchema,
}

/// Input schema for tools
#[derive(Debug, Serialize, Deserialize)]
pub struct InputSchema {
    pub json: Value,
}

/// Tool choice
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolChoice {
    Auto,
    Any,
    Tool { name: String },
}

/// Guardrail configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardrailConfig {
    pub guardrail_identifier: String,
    pub guardrail_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<bool>,
}

/// Execute a converse API request
pub async fn execute_converse(
    client: &crate::core::providers::bedrock::client::BedrockClient,
    request: &ChatRequest,
) -> Result<Value, ProviderError> {
    // Transform ChatRequest to ConverseRequest
    let converse_request = transform_to_converse(request)?;

    // Send request using the client
    let response = client
        .send_request(
            &request.model,
            "converse",
            &serde_json::to_value(converse_request)?,
        )
        .await?;

    // Parse response and return as Value
    response
        .json::<Value>()
        .await
        .map_err(|e| ProviderError::response_parsing("bedrock", e.to_string()))
}

/// Transform OpenAI-style ChatRequest to Converse API format
fn transform_to_converse(request: &ChatRequest) -> Result<ConverseRequest, ProviderError> {
    let mut messages = Vec::new();
    let mut system_messages = Vec::new();

    for msg in &request.messages {
        match msg.role {
            MessageRole::System => {
                // Extract system message
                if let Some(content) = &msg.content {
                    let text = match content {
                        MessageContent::Text(text) => text.clone(),
                        MessageContent::Parts(parts) => {
                            // Extract text from parts
                            parts
                                .iter()
                                .filter_map(|part| {
                                    if let crate::core::types::ContentPart::Text { text } = part {
                                        Some(text.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(" ")
                        }
                    };
                    system_messages.push(SystemMessage {
                        text: Some(text),
                        guardrail_content: None,
                    });
                }
            }
            MessageRole::User | MessageRole::Assistant => {
                // Transform to converse message
                let role = match msg.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    _ => continue,
                }
                .to_string();

                let content = if let Some(msg_content) = &msg.content {
                    match msg_content {
                        MessageContent::Text(text) => {
                            vec![ContentBlock::Text { text: text.clone() }]
                        }
                        MessageContent::Parts(parts) => {
                            parts
                                .iter()
                                .filter_map(|part| {
                                    match part {
                                        crate::core::types::ContentPart::Text { text } => {
                                            Some(ContentBlock::Text { text: text.clone() })
                                        }
                                        crate::core::types::ContentPart::Image { .. } => {
                                            // TODO: Handle image content
                                            None
                                        }
                                        crate::core::types::ContentPart::ImageUrl { .. } => {
                                            // TODO: Handle image URL content
                                            None
                                        }
                                        crate::core::types::ContentPart::Audio { .. } => {
                                            // TODO: Handle audio content
                                            None
                                        }
                                        crate::core::types::ContentPart::Document { .. } => {
                                            // TODO: Handle document content
                                            None
                                        }
                                        crate::core::types::ContentPart::ToolResult { .. } => {
                                            // TODO: Handle tool result content
                                            None
                                        }
                                        crate::core::types::ContentPart::ToolUse { .. } => {
                                            // TODO: Handle tool use content
                                            None
                                        }
                                    }
                                })
                                .collect()
                        }
                    }
                } else {
                    vec![]
                };

                messages.push(ConverseMessage { role, content });
            }
            _ => {
                // Skip function/tool messages for now
                // TODO: Handle tool messages
            }
        }
    }

    // Build inference config
    let inference_config = Some(InferenceConfig {
        max_tokens: request.max_tokens,
        temperature: request.temperature.map(|t| t as f64),
        top_p: request.top_p.map(|t| t as f64),
        stop_sequences: request.stop.clone(),
    });

    // Build tool config if tools are present
    let tool_config = if let Some(tools) = &request.tools {
        let tool_specs: Vec<ToolSpec> = tools
            .iter()
            .map(|tool| ToolSpec {
                tool_spec: ToolSpecDefinition {
                    name: tool.function.name.clone(),
                    description: tool.function.description.clone().unwrap_or_default(),
                    input_schema: InputSchema {
                        json: tool
                            .function
                            .parameters
                            .clone()
                            .unwrap_or(Value::Object(Default::default())),
                    },
                },
            })
            .collect();

        Some(ToolConfig {
            tools: tool_specs,
            tool_choice: None, // TODO: Map tool_choice
        })
    } else {
        None
    };

    Ok(ConverseRequest {
        messages,
        system: if system_messages.is_empty() {
            None
        } else {
            Some(system_messages)
        },
        inference_config,
        tool_config,
        guardrail_config: None, // TODO: Add guardrail support
        additional_model_request_fields: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{ChatMessage, MessageContent, MessageRole};

    // ==================== Data Structure Tests ====================

    #[test]
    fn test_converse_message_serialization() {
        let message = ConverseMessage {
            role: "user".to_string(),
            content: vec![ContentBlock::Text {
                text: "Hello".to_string(),
            }],
        };

        let json = serde_json::to_value(&message).unwrap();
        assert_eq!(json["role"], "user");
        assert!(json["content"].is_array());
    }

    #[test]
    fn test_system_message_with_text() {
        let msg = SystemMessage {
            text: Some("You are a helpful assistant".to_string()),
            guardrail_content: None,
        };

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["text"], "You are a helpful assistant");
        assert!(json.get("guardrail_content").is_none());
    }

    #[test]
    fn test_system_message_with_guardrail() {
        let msg = SystemMessage {
            text: None,
            guardrail_content: Some(GuardrailContent {
                text: "Safety content".to_string(),
            }),
        };

        let json = serde_json::to_value(&msg).unwrap();
        assert!(json.get("text").is_none());
        assert_eq!(json["guardrail_content"]["text"], "Safety content");
    }

    #[test]
    fn test_content_block_text() {
        let block = ContentBlock::Text {
            text: "Hello world".to_string(),
        };

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["text"]["text"], "Hello world");
    }

    #[test]
    fn test_content_block_image() {
        let block = ContentBlock::Image {
            image: ImageBlock {
                format: "png".to_string(),
                source: ImageSource::Bytes {
                    bytes: "base64data".to_string(),
                },
            },
        };

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["image"]["image"]["format"], "png");
    }

    #[test]
    fn test_content_block_document() {
        let block = ContentBlock::Document {
            document: DocumentBlock {
                format: "pdf".to_string(),
                name: "test.pdf".to_string(),
                source: DocumentSource::Bytes {
                    bytes: "pdfdata".to_string(),
                },
            },
        };

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["document"]["document"]["format"], "pdf");
        assert_eq!(json["document"]["document"]["name"], "test.pdf");
    }

    #[test]
    fn test_tool_use_block() {
        let block = ContentBlock::ToolUse {
            tool_use: ToolUseBlock {
                tool_use_id: "tool-123".to_string(),
                name: "get_weather".to_string(),
                input: serde_json::json!({"location": "NYC"}),
            },
        };

        let json = serde_json::to_value(&block).unwrap();
        // ContentBlock::ToolUse serializes as:
        // { "toolUse": { "tool_use": { "toolUseId": "...", "name": "...", ... } } }
        // - outer key "toolUse" comes from enum variant with rename_all = "camelCase"
        // - inner key "tool_use" is the field name in the enum variant
        // - field names inside ToolUseBlock use camelCase (toolUseId)
        assert!(json.get("toolUse").is_some());
        let inner = &json["toolUse"]["tool_use"];
        assert_eq!(inner["toolUseId"], "tool-123");
        assert_eq!(inner["name"], "get_weather");
    }

    #[test]
    fn test_tool_result_block() {
        let block = ContentBlock::ToolResult {
            tool_result: ToolResultBlock {
                tool_use_id: "tool-123".to_string(),
                content: vec![ToolResultContent::Text {
                    text: "Weather is sunny".to_string(),
                }],
                status: Some("success".to_string()),
            },
        };

        let json = serde_json::to_value(&block).unwrap();
        // Similar to ToolUse, serializes as:
        // { "toolResult": { "tool_result": { ... } } }
        let inner = &json["toolResult"]["tool_result"];
        assert_eq!(inner["toolUseId"], "tool-123");
    }

    #[test]
    fn test_inference_config_full() {
        let config = InferenceConfig {
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stop_sequences: Some(vec!["STOP".to_string()]),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["maxTokens"], 1000);
        assert_eq!(json["temperature"], 0.7);
        assert_eq!(json["topP"], 0.9);
    }

    #[test]
    fn test_inference_config_minimal() {
        let config = InferenceConfig {
            max_tokens: None,
            temperature: None,
            top_p: None,
            stop_sequences: None,
        };

        let json = serde_json::to_value(&config).unwrap();
        // All fields should be omitted due to skip_serializing_if
        assert!(json.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_tool_spec() {
        let spec = ToolSpec {
            tool_spec: ToolSpecDefinition {
                name: "calculator".to_string(),
                description: "Performs calculations".to_string(),
                input_schema: InputSchema {
                    json: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "expression": {"type": "string"}
                        }
                    }),
                },
            },
        };

        let json = serde_json::to_value(&spec).unwrap();
        assert_eq!(json["toolSpec"]["name"], "calculator");
        assert_eq!(json["toolSpec"]["description"], "Performs calculations");
    }

    #[test]
    fn test_tool_choice_auto() {
        let choice = ToolChoice::Auto;
        let json = serde_json::to_value(&choice).unwrap();
        assert_eq!(json, "auto");
    }

    #[test]
    fn test_tool_choice_any() {
        let choice = ToolChoice::Any;
        let json = serde_json::to_value(&choice).unwrap();
        assert_eq!(json, "any");
    }

    #[test]
    fn test_tool_choice_specific_tool() {
        let choice = ToolChoice::Tool {
            name: "get_weather".to_string(),
        };
        let json = serde_json::to_value(&choice).unwrap();
        assert_eq!(json["tool"]["name"], "get_weather");
    }

    #[test]
    fn test_guardrail_config() {
        let config = GuardrailConfig {
            guardrail_identifier: "guardrail-123".to_string(),
            guardrail_version: "1.0".to_string(),
            trace: Some(true),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["guardrailIdentifier"], "guardrail-123");
        assert_eq!(json["guardrailVersion"], "1.0");
        assert_eq!(json["trace"], true);
    }

    #[test]
    fn test_image_source_bytes() {
        let source = ImageSource::Bytes {
            bytes: "base64imagedata".to_string(),
        };

        let json = serde_json::to_value(&source).unwrap();
        assert_eq!(json["bytes"]["bytes"], "base64imagedata");
    }

    #[test]
    fn test_document_source_bytes() {
        let source = DocumentSource::Bytes {
            bytes: "base64docdata".to_string(),
        };

        let json = serde_json::to_value(&source).unwrap();
        assert_eq!(json["bytes"]["bytes"], "base64docdata");
    }

    // ==================== Transform Tests ====================

    #[test]
    fn test_transform_simple_user_message() {
        let request = ChatRequest {
            model: "anthropic.claude-3-sonnet".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = transform_to_converse(&request);
        assert!(result.is_ok());

        let converse = result.unwrap();
        assert_eq!(converse.messages.len(), 1);
        assert_eq!(converse.messages[0].role, "user");
    }

    #[test]
    fn test_transform_with_system_message() {
        let request = ChatRequest {
            model: "anthropic.claude-3-sonnet".to_string(),
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: Some(MessageContent::Text("You are helpful".to_string())),
                    ..Default::default()
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: Some(MessageContent::Text("Hello".to_string())),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let result = transform_to_converse(&request);
        assert!(result.is_ok());

        let converse = result.unwrap();
        assert!(converse.system.is_some());
        let system = converse.system.unwrap();
        assert_eq!(system.len(), 1);
        assert_eq!(system[0].text, Some("You are helpful".to_string()));
    }

    #[test]
    fn test_transform_with_inference_config() {
        let request = ChatRequest {
            model: "anthropic.claude-3-sonnet".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            max_tokens: Some(500),
            temperature: Some(0.8),
            top_p: Some(0.95),
            stop: Some(vec!["END".to_string()]),
            ..Default::default()
        };

        let result = transform_to_converse(&request);
        assert!(result.is_ok());

        let converse = result.unwrap();
        assert!(converse.inference_config.is_some());

        let config = converse.inference_config.unwrap();
        assert_eq!(config.max_tokens, Some(500));
        assert!((config.temperature.unwrap() - 0.8).abs() < 0.001);
        assert!((config.top_p.unwrap() - 0.95).abs() < 0.001);
        assert_eq!(config.stop_sequences, Some(vec!["END".to_string()]));
    }

    #[test]
    fn test_transform_conversation() {
        let request = ChatRequest {
            model: "anthropic.claude-3-sonnet".to_string(),
            messages: vec![
                ChatMessage {
                    role: MessageRole::User,
                    content: Some(MessageContent::Text("Hi".to_string())),
                    ..Default::default()
                },
                ChatMessage {
                    role: MessageRole::Assistant,
                    content: Some(MessageContent::Text("Hello!".to_string())),
                    ..Default::default()
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: Some(MessageContent::Text("How are you?".to_string())),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let result = transform_to_converse(&request);
        assert!(result.is_ok());

        let converse = result.unwrap();
        assert_eq!(converse.messages.len(), 3);
        assert_eq!(converse.messages[0].role, "user");
        assert_eq!(converse.messages[1].role, "assistant");
        assert_eq!(converse.messages[2].role, "user");
    }

    #[test]
    fn test_transform_empty_messages() {
        let request = ChatRequest {
            model: "anthropic.claude-3-sonnet".to_string(),
            messages: vec![],
            ..Default::default()
        };

        let result = transform_to_converse(&request);
        assert!(result.is_ok());

        let converse = result.unwrap();
        assert!(converse.messages.is_empty());
        assert!(converse.system.is_none());
    }

    #[test]
    fn test_transform_message_without_content() {
        let request = ChatRequest {
            model: "anthropic.claude-3-sonnet".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: None,
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = transform_to_converse(&request);
        assert!(result.is_ok());

        let converse = result.unwrap();
        assert_eq!(converse.messages.len(), 1);
        assert!(converse.messages[0].content.is_empty());
    }

    // ==================== Converse Request Full Tests ====================

    #[test]
    fn test_converse_request_serialization() {
        let request = ConverseRequest {
            messages: vec![ConverseMessage {
                role: "user".to_string(),
                content: vec![ContentBlock::Text {
                    text: "Hello".to_string(),
                }],
            }],
            system: Some(vec![SystemMessage {
                text: Some("Be helpful".to_string()),
                guardrail_content: None,
            }]),
            inference_config: Some(InferenceConfig {
                max_tokens: Some(100),
                temperature: Some(0.5),
                top_p: None,
                stop_sequences: None,
            }),
            tool_config: None,
            guardrail_config: None,
            additional_model_request_fields: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(json["messages"].is_array());
        assert!(json["system"].is_array());
        assert_eq!(json["inferenceConfig"]["maxTokens"], 100);
    }

    #[test]
    fn test_converse_request_deserialization() {
        let json = serde_json::json!({
            "messages": [{
                "role": "user",
                "content": [{"text": {"text": "Hello"}}]
            }],
            "inferenceConfig": {
                "maxTokens": 200
            }
        });

        let request: ConverseRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
    }

    // ==================== Tool Result Content Tests ====================

    #[test]
    fn test_tool_result_content_text() {
        let content = ToolResultContent::Text {
            text: "Result text".to_string(),
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["text"]["text"], "Result text");
    }

    #[test]
    fn test_tool_result_content_image() {
        let content = ToolResultContent::Image {
            image: ImageBlock {
                format: "jpeg".to_string(),
                source: ImageSource::Bytes {
                    bytes: "imagedata".to_string(),
                },
            },
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["image"]["image"]["format"], "jpeg");
    }

    #[test]
    fn test_tool_result_content_document() {
        let content = ToolResultContent::Document {
            document: DocumentBlock {
                format: "txt".to_string(),
                name: "result.txt".to_string(),
                source: DocumentSource::Bytes {
                    bytes: "docdata".to_string(),
                },
            },
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["document"]["document"]["name"], "result.txt");
    }

    // ==================== Tool Config Tests ====================

    #[test]
    fn test_tool_config_with_tools() {
        let config = ToolConfig {
            tools: vec![
                ToolSpec {
                    tool_spec: ToolSpecDefinition {
                        name: "tool1".to_string(),
                        description: "First tool".to_string(),
                        input_schema: InputSchema {
                            json: serde_json::json!({}),
                        },
                    },
                },
                ToolSpec {
                    tool_spec: ToolSpecDefinition {
                        name: "tool2".to_string(),
                        description: "Second tool".to_string(),
                        input_schema: InputSchema {
                            json: serde_json::json!({}),
                        },
                    },
                },
            ],
            tool_choice: Some(ToolChoice::Auto),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["tools"].as_array().unwrap().len(), 2);
        // ToolConfig has no rename_all, so field stays as tool_choice
        assert_eq!(json["tool_choice"], "auto");
    }

    #[test]
    fn test_guardrail_content() {
        let content = GuardrailContent {
            text: "Safety message".to_string(),
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["text"], "Safety message");
    }

    #[test]
    fn test_content_block_guardrail() {
        let block = ContentBlock::GuardrailContent {
            guardrail_content: GuardrailContent {
                text: "Guardrail text".to_string(),
            },
        };

        let json = serde_json::to_value(&block).unwrap();
        assert!(json.get("guardrailContent").is_some());
    }
}
