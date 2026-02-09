//! Converse API Implementation
//!
//! Modern unified API for chat completions in Bedrock

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::chat::ChatRequest;
use crate::core::types::{message::MessageContent, message::MessageRole};
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
                                    if let crate::core::types::content::ContentPart::Text { text } =
                                        part
                                    {
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
                                        crate::core::types::content::ContentPart::Text { text } => {
                                            Some(ContentBlock::Text { text: text.clone() })
                                        }
                                        crate::core::types::content::ContentPart::Image {
                                            ..
                                        } => {
                                            // TODO: Handle image content
                                            None
                                        }
                                        crate::core::types::content::ContentPart::ImageUrl {
                                            ..
                                        } => {
                                            // TODO: Handle image URL content
                                            None
                                        }
                                        crate::core::types::content::ContentPart::Audio {
                                            ..
                                        } => {
                                            // TODO: Handle audio content
                                            None
                                        }
                                        crate::core::types::content::ContentPart::Document {
                                            ..
                                        } => {
                                            // TODO: Handle document content
                                            None
                                        }
                                        crate::core::types::content::ContentPart::ToolResult {
                                            ..
                                        } => {
                                            // TODO: Handle tool result content
                                            None
                                        }
                                        crate::core::types::content::ContentPart::ToolUse {
                                            ..
                                        } => {
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
#[path = "converse_tests.rs"]
mod tests;
