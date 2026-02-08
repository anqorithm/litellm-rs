//! Heroku Client
//!
//! Request transformation, response processing, and model definitions for Heroku AI Inference

use serde_json::{Value, json};

use super::config::PROVIDER_NAME;
use super::models::get_heroku_registry;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{ChatRequest, model::ModelInfo, responses::ChatResponse};

/// Heroku API client logic
pub struct HerokuClient;

impl HerokuClient {
    /// Transform chat request to Heroku format (OpenAI compatible)
    pub fn transform_chat_request(request: ChatRequest) -> Value {
        json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            "top_p": request.top_p,
            "stream": request.stream,
            "tools": request.tools,
            "tool_choice": request.tool_choice,
            "stop": request.stop,
            "seed": request.seed,
            "response_format": request.response_format,
            "user": request.user,
        })
    }

    /// Transform Heroku response to standard format
    pub fn transform_chat_response(response: Value) -> Result<ChatResponse, ProviderError> {
        // Check for error in response
        if let Some(error) = response.get("error") {
            let error_message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error from Heroku Inference API");

            let error_code = error
                .get("code")
                .and_then(|c| c.as_str())
                .or_else(|| error.get("type").and_then(|t| t.as_str()))
                .unwrap_or("unknown_error");

            return Err(match error_code {
                "authentication_error" | "invalid_api_key" => {
                    ProviderError::authentication(PROVIDER_NAME, error_message)
                }
                "rate_limit_exceeded" | "rate_limit_error" => {
                    ProviderError::rate_limit(PROVIDER_NAME, None)
                }
                "model_not_found" | "invalid_model" => {
                    let model = error
                        .get("param")
                        .and_then(|p| p.as_str())
                        .unwrap_or("unknown");
                    ProviderError::model_not_found(PROVIDER_NAME, model)
                }
                _ => ProviderError::api_error(PROVIDER_NAME, 400, error_message),
            });
        }

        // First try direct deserialization
        if let Ok(chat_response) = serde_json::from_value::<ChatResponse>(response.clone()) {
            return Ok(chat_response);
        }

        // Build response with defaults for missing fields
        let mut response_obj = response
            .as_object()
            .ok_or_else(|| {
                ProviderError::response_parsing(
                    PROVIDER_NAME,
                    "Response is not an object".to_string(),
                )
            })?
            .clone();

        // If no id field, generate one
        if !response_obj.contains_key("id") {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            response_obj.insert(
                "id".to_string(),
                Value::String(format!("chatcmpl-heroku-{}", timestamp)),
            );
        }

        // Default object type
        if !response_obj.contains_key("object") {
            response_obj.insert(
                "object".to_string(),
                Value::String("chat.completion".to_string()),
            );
        }

        // If no created field, add current timestamp
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

        // Default model if not present
        if !response_obj.contains_key("model") {
            response_obj.insert(
                "model".to_string(),
                Value::String("claude-4-5-sonnet".to_string()),
            );
        }

        // Try deserialization again
        serde_json::from_value(Value::Object(response_obj))
            .map_err(|e| ProviderError::response_parsing(PROVIDER_NAME, e.to_string()))
    }

    /// Get supported models
    pub fn supported_models() -> Vec<ModelInfo> {
        get_heroku_registry().get_chat_models()
    }

    /// Get all models including embeddings and image generation
    pub fn all_models() -> Vec<ModelInfo> {
        get_heroku_registry().get_all_models()
    }

    /// Get supported OpenAI-compatible parameters
    pub fn supported_openai_params() -> &'static [&'static str] {
        &[
            "temperature",
            "max_tokens",
            "max_completion_tokens",
            "top_p",
            "stream",
            "tools",
            "tool_choice",
            "stop",
            "seed",
            "response_format",
            "user",
            "presence_penalty",
            "frequency_penalty",
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
            model: "claude-4-5-sonnet".to_string(),
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

        let transformed = HerokuClient::transform_chat_request(request);
        assert_eq!(transformed["model"], "claude-4-5-sonnet");
        let temp = transformed["temperature"].as_f64().unwrap();
        assert!((temp - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_supported_models() {
        let models = HerokuClient::supported_models();
        assert!(!models.is_empty());

        // Check for Claude models
        let claude_models: Vec<_> = models.iter().filter(|m| m.id.contains("claude")).collect();
        assert!(!claude_models.is_empty());
    }

    #[test]
    fn test_all_models() {
        let models = HerokuClient::all_models();
        let chat_models = HerokuClient::supported_models();

        // All models should include embedding and image generation models
        assert!(models.len() >= chat_models.len());
    }

    #[test]
    fn test_supported_openai_params() {
        let params = HerokuClient::supported_openai_params();
        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"tools"));
        assert!(params.contains(&"stream"));
        assert!(params.contains(&"user"));
    }

    #[test]
    fn test_transform_response_with_error() {
        let error_response = json!({
            "error": {
                "message": "Invalid API key",
                "code": "authentication_error"
            }
        });

        let result = HerokuClient::transform_chat_response(error_response);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_response_rate_limit() {
        let error_response = json!({
            "error": {
                "message": "Rate limit exceeded",
                "code": "rate_limit_exceeded"
            }
        });

        let result = HerokuClient::transform_chat_response(error_response);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProviderError::RateLimit { .. }
        ));
    }

    #[test]
    fn test_transform_response_model_not_found() {
        let error_response = json!({
            "error": {
                "message": "Model not found",
                "code": "model_not_found",
                "param": "invalid-model"
            }
        });

        let result = HerokuClient::transform_chat_response(error_response);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProviderError::ModelNotFound { .. }
        ));
    }

    #[test]
    fn test_transform_response_not_object() {
        let invalid_response = json!("not an object");

        let result = HerokuClient::transform_chat_response(invalid_response);
        assert!(result.is_err());
    }
}
