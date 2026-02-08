//! DeepSeek Client
//!
//! Request transformation, response processing, and model definitions

use serde_json::{Value, json};

use super::models::get_deepseek_registry;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{ChatRequest, model::ModelInfo, responses::ChatResponse};

/// DeepSeek API client logic
pub struct DeepSeekClient;

impl DeepSeekClient {
    /// Request
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

    /// Response
    pub fn transform_chat_response(response: Value) -> Result<ChatResponse, ProviderError> {
        // Response
        if let Some(error) = response.get("error") {
            let error_message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error from DeepSeek API");

            let error_code = error
                .get("code")
                .and_then(|c| c.as_str())
                .unwrap_or("unknown_error");

            return Err(match error_code {
                "authentication_error" | "invalid_request_error" => {
                    ProviderError::authentication("deepseek", error_message)
                }
                "rate_limit_exceeded" => ProviderError::rate_limit("deepseek", None),
                _ => ProviderError::api_error("deepseek", 400, error_message),
            });
        }

        // First try direct deserialization
        if let Ok(chat_response) = serde_json::from_value::<ChatResponse>(response.clone()) {
            return Ok(chat_response);
        }

        // Build
        let mut response_obj = response
            .as_object()
            .ok_or_else(|| {
                ProviderError::response_parsing("deepseek", "Response is not an object".to_string())
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
                Value::String(format!("chatcmpl-deepseek-{}", timestamp)),
            );
        }

        // Default
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

        // Default
        if !response_obj.contains_key("model") {
            response_obj.insert(
                "model".to_string(),
                Value::String("deepseek-chat".to_string()),
            );
        }

        // Try deserialization again
        serde_json::from_value(Value::Object(response_obj))
            .map_err(|e| ProviderError::response_parsing("deepseek", e.to_string()))
    }

    /// Model
    pub fn supported_models() -> Vec<ModelInfo> {
        get_deepseek_registry().get_all_models()
    }

    /// Get
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
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{ChatMessage, message::MessageContent, message::MessageRole};
    use std::collections::HashMap;

    #[test]
    fn test_transform_request() {
        let request = ChatRequest {
            model: "deepseek-chat".to_string(),
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

        let transformed = DeepSeekClient::transform_chat_request(request);
        assert_eq!(transformed["model"], "deepseek-chat");
        // Check temperature is approximately 0.7 (accounting for floating point precision)
        let temp = transformed["temperature"].as_f64().unwrap();
        assert!((temp - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_supported_models() {
        let models = DeepSeekClient::supported_models();
        assert!(models.len() >= 2); // At least 2 models should be supported

        // Check that expected models are present
        let model_ids: Vec<String> = models.iter().map(|m| m.id.clone()).collect();
        assert!(model_ids.contains(&"deepseek-chat".to_string()));
        // Check for either deepseek-reasoner or deepseek-coder (model availability may vary)
        assert!(
            model_ids.contains(&"deepseek-reasoner".to_string())
                || model_ids.contains(&"deepseek-coder".to_string())
        );
    }
}
