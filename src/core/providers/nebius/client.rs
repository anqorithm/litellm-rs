//! Nebius Client
//!
//! Request transformation and response processing for Nebius API

use serde_json::{Value, json};

use super::models::get_nebius_registry;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{ChatRequest, model::ModelInfo, responses::ChatResponse};

/// Nebius API client logic
pub struct NebiusClient;

impl NebiusClient {
    /// Transform ChatRequest to Nebius API format
    ///
    /// Nebius uses OpenAI-compatible format
    pub fn transform_chat_request(request: ChatRequest) -> Value {
        json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            "top_p": request.top_p,
            "frequency_penalty": request.frequency_penalty,
            "presence_penalty": request.presence_penalty,
            "stream": request.stream,
            "tools": request.tools,
            "tool_choice": request.tool_choice,
        })
    }

    /// Transform Nebius response to standard ChatResponse
    pub fn transform_chat_response(response: Value) -> Result<ChatResponse, ProviderError> {
        // Check for error in response
        if let Some(error) = response.get("error") {
            let error_message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error from Nebius API");

            let error_code = error
                .get("code")
                .and_then(|c| c.as_str())
                .unwrap_or("unknown_error");

            return Err(match error_code {
                "invalid_api_key" | "authentication_failed" => {
                    ProviderError::authentication("nebius", error_message)
                }
                "rate_limit_exceeded" => ProviderError::rate_limit("nebius", None),
                "model_not_found" => ProviderError::model_not_found("nebius", error_message),
                _ => ProviderError::api_error("nebius", 400, error_message),
            });
        }

        // Try direct deserialization first (OpenAI-compatible format)
        if let Ok(chat_response) = serde_json::from_value::<ChatResponse>(response.clone()) {
            return Ok(chat_response);
        }

        // Build response with defaults if needed
        let mut response_obj = response
            .as_object()
            .ok_or_else(|| {
                ProviderError::response_parsing("nebius", "Response is not an object".to_string())
            })?
            .clone();

        // Generate ID if missing
        if !response_obj.contains_key("id") {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            response_obj.insert(
                "id".to_string(),
                Value::String(format!("chatcmpl-nebius-{}", timestamp)),
            );
        }

        // Set default object type
        if !response_obj.contains_key("object") {
            response_obj.insert(
                "object".to_string(),
                Value::String("chat.completion".to_string()),
            );
        }

        // Add created timestamp if missing
        if !response_obj.contains_key("created") {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            response_obj.insert(
                "created".to_string(),
                Value::Number(serde_json::Number::from(timestamp)),
            );
        }

        // Set default model if missing
        if !response_obj.contains_key("model") {
            response_obj.insert(
                "model".to_string(),
                Value::String("meta-llama/Meta-Llama-3.1-8B-Instruct".to_string()),
            );
        }

        // Try deserialization again
        serde_json::from_value(Value::Object(response_obj))
            .map_err(|e| ProviderError::response_parsing("nebius", e.to_string()))
    }

    /// Get supported models
    pub fn supported_models() -> Vec<ModelInfo> {
        get_nebius_registry().get_all_models()
    }

    /// Get supported OpenAI parameters
    pub fn supported_openai_params() -> &'static [&'static str] {
        &[
            "temperature",
            "max_tokens",
            "top_p",
            "frequency_penalty",
            "presence_penalty",
            "stream",
            "tools",
            "tool_choice",
            "stop",
            "n",
            "seed",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{ChatMessage, MessageContent, MessageRole};
    use std::collections::HashMap;

    #[test]
    fn test_transform_request() {
        let request = ChatRequest {
            model: "meta-llama/Meta-Llama-3.1-8B-Instruct".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                thinking: None,
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: false,
            tools: None,
            tool_choice: None,
            user: None,
            response_format: None,
            seed: None,
            max_completion_tokens: None,
            stop: None,
            parallel_tool_calls: None,
            n: None,
            logit_bias: None,
            functions: None,
            function_call: None,
            logprobs: None,
            top_logprobs: None,
            thinking: None,
            extra_params: HashMap::new(),
        };

        let transformed = NebiusClient::transform_chat_request(request);
        assert_eq!(
            transformed["model"],
            "meta-llama/Meta-Llama-3.1-8B-Instruct"
        );
        let temp = transformed["temperature"].as_f64().unwrap();
        assert!((temp - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_supported_models() {
        let models = NebiusClient::supported_models();
        assert!(models.len() >= 5); // At least 5 models should be supported

        let model_ids: Vec<String> = models.iter().map(|m| m.id.clone()).collect();
        assert!(model_ids.contains(&"meta-llama/Meta-Llama-3.1-8B-Instruct".to_string()));
    }

    #[test]
    fn test_supported_openai_params() {
        let params = NebiusClient::supported_openai_params();
        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"tools"));
    }
}
