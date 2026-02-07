//! Completion types - Python LiteLLM compatible

use crate::core::types::{ChatMessage, Tool, ToolChoice};
use crate::core::types::responses::{FinishReason, Usage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool call structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

/// Function call structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Completion options - Python LiteLLM compatible
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub extra_params: HashMap<String, serde_json::Value>,
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

/// Response choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<FinishReason>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::MessageRole;

    #[test]
    fn test_completion_options_default() {
        let opts = CompletionOptions::default();
        assert!(opts.temperature.is_none());
        assert!(opts.max_tokens.is_none());
        assert!(!opts.stream);
        assert!(opts.api_key.is_none());
    }

    #[test]
    fn test_completion_options_with_values() {
        let opts = CompletionOptions {
            temperature: Some(0.5),
            max_tokens: Some(100),
            stream: true,
            ..Default::default()
        };

        assert_eq!(opts.temperature, Some(0.5));
        assert_eq!(opts.max_tokens, Some(100));
        assert!(opts.stream);
    }

    #[test]
    fn test_completion_options_serialization() {
        let opts = CompletionOptions {
            temperature: Some(0.5),
            max_tokens: Some(100),
            ..Default::default()
        };

        let json = serde_json::to_value(&opts).unwrap();
        assert_eq!(json["temperature"], 0.5);
        assert_eq!(json["max_tokens"], 100);
        // stream defaults to false, should be present
        assert_eq!(json["stream"], false);
    }

    #[test]
    fn test_tool_call_structure() {
        let call = ToolCall {
            id: "call_123".to_string(),
            r#type: "function".to_string(),
            function: FunctionCall {
                name: "get_weather".to_string(),
                arguments: "{\"city\": \"NYC\"}".to_string(),
            },
        };

        assert_eq!(call.id, "call_123");
        assert_eq!(call.function.name, "get_weather");
    }

    #[test]
    fn test_completion_response() {
        let response = CompletionResponse {
            id: "cmpl-123".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            choices: vec![],
            usage: None,
        };

        assert_eq!(response.id, "cmpl-123");
        assert_eq!(response.model, "gpt-4");
        assert!(response.choices.is_empty());
    }

    #[test]
    fn test_choice_structure() {
        let choice = Choice {
            index: 0,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: Some(crate::core::types::MessageContent::Text(
                    "Hello".to_string(),
                )),
                ..Default::default()
            },
            finish_reason: Some(FinishReason::Stop),
        };

        assert_eq!(choice.index, 0);
        assert_eq!(choice.message.role, MessageRole::Assistant);
    }

    #[test]
    fn test_completion_options_api_config() {
        let opts = CompletionOptions {
            api_key: Some("sk-test".to_string()),
            api_base: Some("https://api.example.com".to_string()),
            timeout: Some(30),
            ..Default::default()
        };

        assert_eq!(opts.api_key, Some("sk-test".to_string()));
        assert_eq!(opts.timeout, Some(30));
    }
}
