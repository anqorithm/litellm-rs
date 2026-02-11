//! Cohere Chat Completions Handler
//!
//! Handles chat completion requests for Cohere Command models.
//! Supports both v1 (legacy) and v2 (OpenAI-compatible) APIs.

#[cfg(test)]
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use super::config::{CohereApiVersion, CohereConfig};
use super::error::CohereError;
use crate::core::types::chat::ChatMessage as ResponseMessage;
use crate::core::types::responses::{ChatChoice, ChatResponse, FinishReason, Usage};
use crate::core::types::tools::ToolCall;
use crate::core::types::{chat::ChatRequest, message::MessageContent, message::MessageRole};

/// Cohere v2 chat request (OpenAI-compatible)
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereChatRequest {
    /// Model to use
    pub model: String,

    /// Messages in OpenAI format
    pub messages: Vec<Value>,

    /// Temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Top-p sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p: Option<f32>,

    /// Frequency penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Presence penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    /// Enable streaming
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Tools (functions) available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,

    /// Random seed for reproducibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Documents for RAG
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documents: Option<Vec<Value>>,

    /// Preamble (system message override)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preamble: Option<String>,
}

/// Cohere v2 chat response
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereChatResponse {
    /// Response ID
    pub id: String,

    /// Message content
    pub message: CohereMessage,

    /// Finish reason
    pub finish_reason: Option<String>,

    /// Usage information
    pub usage: CohereUsage,
}

/// Cohere message structure
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereMessage {
    /// Role of the message
    pub role: String,

    /// Content of the message
    #[serde(default)]
    pub content: Option<Vec<CohereContent>>,

    /// Tool calls if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,

    /// Citations from RAG
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<CohereCitation>>,
}

/// Cohere content structure
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereContent {
    /// Content type
    #[serde(rename = "type")]
    pub content_type: String,

    /// Text content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Cohere citation for RAG
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereCitation {
    /// Start position in text
    pub start: u32,

    /// End position in text
    pub end: u32,

    /// Cited text
    pub text: String,

    /// Source documents
    #[serde(default)]
    pub sources: Vec<CohereSource>,
}

/// Cohere source document
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereSource {
    /// Source type
    #[serde(rename = "type")]
    pub source_type: String,

    /// Source ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Document data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<Value>,
}

/// Cohere usage information
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereUsage {
    /// Token usage
    pub tokens: CohereTokens,
}

/// Cohere token counts
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereTokens {
    /// Input tokens
    pub input_tokens: u32,

    /// Output tokens
    pub output_tokens: u32,
}

/// Chat handler utilities
pub struct CohereChatHandler;

impl CohereChatHandler {
    /// Transform ChatRequest to Cohere format
    pub fn transform_request(
        request: &ChatRequest,
        config: &CohereConfig,
    ) -> Result<Value, CohereError> {
        let mut body = json!({
            "model": request.model,
            "messages": request.messages,
        });

        // Add optional parameters
        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }

        if let Some(max_tokens) = request.max_tokens.or(request.max_completion_tokens) {
            body["max_tokens"] = json!(max_tokens);
        }

        if let Some(top_p) = request.top_p {
            body["p"] = json!(top_p);
        }

        if let Some(frequency_penalty) = request.frequency_penalty {
            body["frequency_penalty"] = json!(frequency_penalty);
        }

        if let Some(presence_penalty) = request.presence_penalty {
            body["presence_penalty"] = json!(presence_penalty);
        }

        if let Some(stop) = &request.stop {
            body["stop_sequences"] = json!(stop);
        }

        if request.stream {
            body["stream"] = json!(true);
        }

        if let Some(tools) = &request.tools {
            // Transform OpenAI tools to Cohere format if using v1
            if config.api_version == CohereApiVersion::V1 {
                // Convert tools to JSON values first
                let tools_json: Vec<Value> = tools
                    .iter()
                    .filter_map(|t| serde_json::to_value(t).ok())
                    .collect();
                let cohere_tools = Self::transform_tools_to_v1(&tools_json)?;
                body["tools"] = cohere_tools;
            } else {
                body["tools"] = json!(tools);
            }
        }

        if let Some(seed) = request.seed {
            body["seed"] = json!(seed);
        }

        Ok(body)
    }

    /// Transform OpenAI tools to Cohere v1 format
    fn transform_tools_to_v1(tools: &[Value]) -> Result<Value, CohereError> {
        let mut cohere_tools = Vec::new();

        for tool in tools {
            if let Some(function) = tool.get("function") {
                let name = function.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let description = function
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");

                let mut parameter_definitions = HashMap::new();

                if let Some(params) = function.get("parameters") {
                    if let Some(properties) = params.get("properties").and_then(|p| p.as_object()) {
                        let required = params
                            .get("required")
                            .and_then(|r| r.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                            .unwrap_or_default();

                        for (param_name, param_def) in properties {
                            let param_type = param_def
                                .get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("string");
                            let param_desc = param_def
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("");

                            parameter_definitions.insert(
                                param_name.clone(),
                                json!({
                                    "type": param_type,
                                    "description": param_desc,
                                    "required": required.contains(&param_name.as_str())
                                }),
                            );
                        }
                    }
                }

                cohere_tools.push(json!({
                    "name": name,
                    "description": description,
                    "parameter_definitions": parameter_definitions
                }));
            }
        }

        Ok(json!(cohere_tools))
    }

    /// Transform Cohere v2 response to standard ChatResponse
    pub fn transform_response(
        response_json: Value,
        model: &str,
    ) -> Result<ChatResponse, CohereError> {
        let id = response_json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extract content from v2 format
        let content = Self::extract_content(&response_json)?;

        // Extract tool calls if present
        // Note: tool_calls parsing is handled by parse_tool_calls method

        // Extract usage
        let usage = Self::extract_usage(&response_json)?;

        let message = ResponseMessage {
            role: MessageRole::Assistant,
            content: Some(MessageContent::Text(content)),
            thinking: None,
            tool_calls: Self::parse_tool_calls(&response_json),
            name: None,
            function_call: None,
            tool_call_id: None,
        };

        let finish_reason = response_json
            .get("finish_reason")
            .and_then(|v| v.as_str())
            .map(|s| match s.to_lowercase().as_str() {
                "stop" | "complete" | "end_turn" => FinishReason::Stop,
                "length" | "max_tokens" => FinishReason::Length,
                "tool_calls" | "tool_use" => FinishReason::ToolCalls,
                "content_filter" => FinishReason::ContentFilter,
                _ => FinishReason::Stop,
            });

        let choice = ChatChoice {
            index: 0,
            message,
            finish_reason,
            logprobs: None,
        };

        Ok(ChatResponse {
            id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![choice],
            usage: Some(usage),
            system_fingerprint: None,
        })
    }

    /// Extract content from Cohere response
    fn extract_content(response_json: &Value) -> Result<String, CohereError> {
        // v2 format: message.content is an array of content blocks
        if let Some(message) = response_json.get("message") {
            if let Some(content) = message.get("content") {
                if let Some(content_array) = content.as_array() {
                    let text: String = content_array
                        .iter()
                        .filter_map(|c| c.get("text").and_then(|t| t.as_str()))
                        .collect::<Vec<_>>()
                        .join("");
                    return Ok(text);
                }
            }
        }

        // v1 format: text is at top level
        if let Some(text) = response_json.get("text").and_then(|t| t.as_str()) {
            return Ok(text.to_string());
        }

        Ok(String::new())
    }

    /// Extract usage from Cohere response
    fn extract_usage(response_json: &Value) -> Result<Usage, CohereError> {
        // v2 format: usage.tokens
        if let Some(usage) = response_json.get("usage") {
            if let Some(tokens) = usage.get("tokens") {
                let prompt_tokens = tokens
                    .get("input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                let completion_tokens = tokens
                    .get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                return Ok(Usage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                    prompt_tokens_details: None,
                    completion_tokens_details: None,
                    thinking_usage: None,
                });
            }
        }

        // v1 format: meta.billed_units
        if let Some(meta) = response_json.get("meta") {
            if let Some(billed_units) = meta.get("billed_units") {
                let prompt_tokens = billed_units
                    .get("input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                let completion_tokens = billed_units
                    .get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                return Ok(Usage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                    prompt_tokens_details: None,
                    completion_tokens_details: None,
                    thinking_usage: None,
                });
            }
        }

        Ok(Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        })
    }

    /// Parse tool calls from Cohere response
    fn parse_tool_calls(response_json: &Value) -> Option<Vec<ToolCall>> {
        let tool_calls_arr = response_json
            .get("message")
            .and_then(|m| m.get("tool_calls"))
            .and_then(|tc| tc.as_array())?;

        let tool_calls: Vec<ToolCall> = tool_calls_arr
            .iter()
            .map(|tc| {
                let id = tc
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = tc
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let arguments = tc
                    .get("function")
                    .and_then(|f| f.get("arguments"))
                    .map(|a| {
                        if a.is_string() {
                            a.as_str().unwrap_or("{}").to_string()
                        } else {
                            serde_json::to_string(a).unwrap_or_else(|_| "{}".to_string())
                        }
                    })
                    .unwrap_or_else(|| "{}".to_string());

                ToolCall {
                    id,
                    tool_type: "function".to_string(),
                    function: crate::core::types::tools::FunctionCall { name, arguments },
                }
            })
            .collect();

        if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        }
    }

    /// Get supported OpenAI parameters for Cohere
    pub fn get_supported_params() -> &'static [&'static str] {
        &[
            "stream",
            "temperature",
            "max_tokens",
            "max_completion_tokens",
            "top_p",
            "frequency_penalty",
            "presence_penalty",
            "stop",
            "n",
            "tools",
            "tool_choice",
            "seed",
        ]
    }

    /// Map OpenAI parameters to Cohere format
    pub fn map_openai_params(params: HashMap<String, Value>) -> HashMap<String, Value> {
        let mut mapped = HashMap::new();

        for (key, value) in params {
            match key.as_str() {
                "stream" => {
                    mapped.insert("stream".to_string(), value);
                }
                "temperature" => {
                    mapped.insert("temperature".to_string(), value);
                }
                "max_tokens" | "max_completion_tokens" => {
                    mapped.insert("max_tokens".to_string(), value);
                }
                "top_p" => {
                    mapped.insert("p".to_string(), value);
                }
                "frequency_penalty" => {
                    mapped.insert("frequency_penalty".to_string(), value);
                }
                "presence_penalty" => {
                    mapped.insert("presence_penalty".to_string(), value);
                }
                "stop" => {
                    mapped.insert("stop_sequences".to_string(), value);
                }
                "n" => {
                    mapped.insert("num_generations".to_string(), value);
                }
                "tools" => {
                    mapped.insert("tools".to_string(), value);
                }
                "seed" => {
                    mapped.insert("seed".to_string(), value);
                }
                _ => {}
            }
        }

        mapped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_params() {
        let params = CohereChatHandler::get_supported_params();
        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"tools"));
    }

    #[test]
    fn test_map_openai_params() {
        let mut params = HashMap::new();
        params.insert("temperature".to_string(), json!(0.7));
        params.insert("max_tokens".to_string(), json!(100));
        params.insert("top_p".to_string(), json!(0.9));
        params.insert("stop".to_string(), json!(["END"]));

        let mapped = CohereChatHandler::map_openai_params(params);

        assert_eq!(mapped.get("temperature").unwrap(), &json!(0.7));
        assert_eq!(mapped.get("max_tokens").unwrap(), &json!(100));
        assert_eq!(mapped.get("p").unwrap(), &json!(0.9));
        assert_eq!(mapped.get("stop_sequences").unwrap(), &json!(["END"]));
    }

    #[test]
    fn test_extract_usage_v2() {
        let response = json!({
            "usage": {
                "tokens": {
                    "input_tokens": 100,
                    "output_tokens": 50
                }
            }
        });

        let usage = CohereChatHandler::extract_usage(&response).unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_extract_usage_v1() {
        let response = json!({
            "meta": {
                "billed_units": {
                    "input_tokens": 80,
                    "output_tokens": 40
                }
            }
        });

        let usage = CohereChatHandler::extract_usage(&response).unwrap();
        assert_eq!(usage.prompt_tokens, 80);
        assert_eq!(usage.completion_tokens, 40);
        assert_eq!(usage.total_tokens, 120);
    }

    #[test]
    fn test_extract_content_v2() {
        let response = json!({
            "message": {
                "content": [
                    {"type": "text", "text": "Hello, "},
                    {"type": "text", "text": "world!"}
                ]
            }
        });

        let content = CohereChatHandler::extract_content(&response).unwrap();
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_extract_content_v1() {
        let response = json!({
            "text": "Hello from v1!"
        });

        let content = CohereChatHandler::extract_content(&response).unwrap();
        assert_eq!(content, "Hello from v1!");
    }

    #[test]
    fn test_transform_tools_to_v1() {
        let tools = vec![json!({
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get weather info",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "City name"
                        }
                    },
                    "required": ["location"]
                }
            }
        })];

        let cohere_tools = CohereChatHandler::transform_tools_to_v1(&tools).unwrap();
        let tools_array = cohere_tools.as_array().unwrap();

        assert_eq!(tools_array.len(), 1);
        assert_eq!(tools_array[0]["name"], "get_weather");
        assert!(
            tools_array[0]["parameter_definitions"]["location"]["required"]
                .as_bool()
                .unwrap()
        );
    }
}
