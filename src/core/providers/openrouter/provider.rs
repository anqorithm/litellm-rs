//! OpenRouter Provider Implementation
//!
//! Unified OpenRouter provider using the modern architecture

use serde_json::Value;
use std::collections::HashMap;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::health::HealthStatus;
use crate::core::types::{
    ChatRequest, ModelInfo, ProviderCapability, responses::ChatResponse, thinking::ThinkingContent,
};

use super::config::OpenRouterConfig;
use super::models::get_openrouter_registry;

const PROVIDER_NAME: &str = "openrouter";
const OPENROUTER_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::FunctionCalling,
    ProviderCapability::ToolCalling,
];

fn get_supported_models() -> Vec<ModelInfo> {
    get_openrouter_registry().get_all_models()
}

crate::define_http_provider_with_hooks!(
    provider: PROVIDER_NAME,
    struct_name: OpenRouterProvider,
    config: super::config::OpenRouterConfig,
    error_mapper: crate::core::traits::error_mapper::implementations::OpenAIErrorMapper,
    model_info: get_supported_models,
    capabilities: OPENROUTER_CAPABILITIES,
    url_builder: |provider: &OpenRouterProvider| -> String {
        format!("{}/chat/completions", provider.config.base_url)
    },
    request_builder: |provider: &OpenRouterProvider, url: &str| -> reqwest::RequestBuilder {
        provider.http_client.post(url)
    },
    supported_params: [
        "temperature",
        "top_p",
        "max_tokens",
        "frequency_penalty",
        "presence_penalty",
        "stop",
        "tools",
        "tool_choice",
        "response_format",
    ],
    build_headers: |provider: &OpenRouterProvider, headers: &mut HashMap<String, String>| {
        headers.extend(provider.config.get_headers());
    },
    with_api_key: |api_key: String| -> Result<OpenRouterProvider, ProviderError> {
        OpenRouterProvider::new(OpenRouterConfig::new(api_key))
    },
    request_transform: |provider: &OpenRouterProvider, request: ChatRequest|
     -> Result<Value, ProviderError> { provider.transform_chat_request(request) },
    response_transform: |provider: &OpenRouterProvider,
                         raw_response: &[u8],
                         model: &str,
                         _request_id: &str|
     -> Result<ChatResponse, ProviderError> {
        let response: Value = serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing(PROVIDER_NAME, e.to_string()))?;
        provider.transform_chat_response(response, model)
    },
    error_map: |provider: &OpenRouterProvider,
                status: u16,
                error_text: String,
                request: &ChatRequest|
     -> ProviderError {
        if let Ok(value) = serde_json::from_str::<Value>(&error_text) {
            if let Some(obj) = value.as_object() {
                if let Some(err) = provider.map_error_response(obj, &request.model) {
                    return err;
                }
            }
        }
        ProviderError::api_error(PROVIDER_NAME, status, error_text)
    },
    health_check: |_provider: &OpenRouterProvider| async { HealthStatus::Healthy },
    streaming_error: "Streaming not yet implemented in unified architecture",
    calculate_cost: |provider: &OpenRouterProvider,
                     model: &str,
                     input_tokens: u32,
                     output_tokens: u32|
     -> Result<f64, ProviderError> {
        if let Some(model_info) = provider.supported_models.iter().find(|m| m.id == model) {
            let input_cost = model_info
                .input_cost_per_1k_tokens
                .unwrap_or(0.0)
                * input_tokens as f64
                / 1000.0;
            let output_cost = model_info
                .output_cost_per_1k_tokens
                .unwrap_or(0.0)
                * output_tokens as f64
                / 1000.0;
            Ok(input_cost + output_cost)
        } else {
            Ok(0.0)
        }
    },
);

impl OpenRouterProvider {
    /// Create provider from environment
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = OpenRouterConfig::from_env();
        Self::new(config)
    }

    /// Transform chat request to OpenRouter format
    fn transform_chat_request(&self, request: ChatRequest) -> Result<Value, ProviderError> {
        let mut body: Value = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "stream": request.stream
        });

        // Add OpenAI-compatible parameters
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = max_tokens.into();
        }

        if let Some(temperature) = request.temperature {
            body["temperature"] = temperature.into();
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = top_p.into();
        }

        if let Some(frequency_penalty) = request.frequency_penalty {
            body["frequency_penalty"] = frequency_penalty.into();
        }

        if let Some(presence_penalty) = request.presence_penalty {
            body["presence_penalty"] = presence_penalty.into();
        }

        if let Some(stop) = request.stop {
            body["stop"] = serde_json::to_value(stop)
                .map_err(|e| ProviderError::serialization(PROVIDER_NAME, e.to_string()))?;
        }

        if let Some(tools) = request.tools {
            body["tools"] = serde_json::to_value(tools)
                .map_err(|e| ProviderError::serialization(PROVIDER_NAME, e.to_string()))?;
        }

        if let Some(tool_choice) = request.tool_choice {
            body["tool_choice"] = serde_json::to_value(tool_choice)
                .map_err(|e| ProviderError::serialization(PROVIDER_NAME, e.to_string()))?;
        }

        // Add OpenRouter-specific parameters
        for (key, value) in &self.config.extra_params {
            body[key.as_str()] = value.clone();
        }

        Ok(body)
    }

    fn map_error_response(
        &self,
        response: &serde_json::Map<String, Value>,
        model: &str,
    ) -> Option<ProviderError> {
        let error = response.get("error")?;
        let error_obj = match error.as_object() {
            Some(obj) => obj,
            None => {
                return Some(ProviderError::response_parsing(
                    PROVIDER_NAME,
                    "Error field is not an object".to_string(),
                ));
            }
        };

        // Try to get detailed error from metadata.raw first, like Python LiteLLM
        let detailed_message = if let Some(metadata) = error_obj.get("metadata") {
            if let Some(raw) = metadata.get("raw").and_then(|v| v.as_str()) {
                // Try to parse the raw error JSON
                if let Ok(raw_error) = serde_json::from_str::<Value>(raw) {
                    if let Some(error_inner) = raw_error.get("error") {
                        if let Some(msg) = error_inner.get("message").and_then(|v| v.as_str()) {
                            // Include provider name for context
                            if let Some(provider_name) =
                                metadata.get("provider_name").and_then(|v| v.as_str())
                            {
                                format!("{}: {}", provider_name, msg)
                            } else {
                                msg.to_string()
                            }
                        } else {
                            error_obj
                                .get("message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown error")
                                .to_string()
                        }
                    } else {
                        error_obj
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error")
                            .to_string()
                    }
                } else {
                    error_obj
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                        .to_string()
                }
            } else {
                error_obj
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string()
            }
        } else {
            error_obj
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string()
        };

        let code = error_obj
            .get("code")
            .and_then(|v| v.as_i64())
            .unwrap_or(500);

        if code == 404
            || detailed_message.contains("Model not found")
            || detailed_message.contains("No endpoints found")
        {
            Some(ProviderError::model_not_found(PROVIDER_NAME, model))
        } else if code == 401 {
            Some(ProviderError::authentication(
                PROVIDER_NAME,
                &detailed_message,
            ))
        } else if code == 429 {
            Some(ProviderError::rate_limit(PROVIDER_NAME, None))
        } else {
            Some(ProviderError::api_error(
                PROVIDER_NAME,
                code as u16,
                detailed_message,
            ))
        }
    }

    /// Transform OpenRouter response to standard format
    fn transform_chat_response(
        &self,
        raw_response: Value,
        model: &str,
    ) -> Result<ChatResponse, ProviderError> {
        let response = raw_response.as_object().ok_or_else(|| {
            ProviderError::response_parsing(
                PROVIDER_NAME,
                "Response is not a JSON object".to_string(),
            )
        })?;

        if let Some(err) = self.map_error_response(response, model) {
            return Err(err);
        }

        // Extract ID
        let id = response
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("openrouter-response")
            .to_string();

        // Extract choices
        let choices = response
            .get("choices")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ProviderError::response_parsing(PROVIDER_NAME, "No choices in response".to_string())
            })?;

        let mut response_choices = Vec::new();
        for (index, choice) in choices.iter().enumerate() {
            let choice_obj = choice.as_object().ok_or_else(|| {
                ProviderError::response_parsing(
                    PROVIDER_NAME,
                    "Choice is not an object".to_string(),
                )
            })?;

            let message = choice_obj.get("message").ok_or_else(|| {
                ProviderError::response_parsing(PROVIDER_NAME, "No message in choice".to_string())
            })?;

            let finish_reason = choice_obj
                .get("finish_reason")
                .and_then(|v| v.as_str())
                .map(|s| match s {
                    "stop" => crate::core::types::responses::FinishReason::Stop,
                    "length" => crate::core::types::responses::FinishReason::Length,
                    "tool_calls" => crate::core::types::responses::FinishReason::ToolCalls,
                    "content_filter" => crate::core::types::responses::FinishReason::ContentFilter,
                    "function_call" => crate::core::types::responses::FinishReason::FunctionCall,
                    _ => crate::core::types::responses::FinishReason::Stop,
                });

            let mut chat_message: crate::core::types::ChatMessage =
                serde_json::from_value(message.clone())
                    .map_err(|e| ProviderError::response_parsing(PROVIDER_NAME, e.to_string()))?;

            if chat_message.thinking.is_none() {
                let thinking = message
                    .get("reasoning_content")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .or_else(|| {
                        message
                            .get("reasoning")
                            .and_then(|v| v.as_str())
                            .filter(|s| !s.is_empty())
                    })
                    .map(|text| ThinkingContent::Text {
                        text: text.to_string(),
                        signature: None,
                    });

                if thinking.is_some() {
                    chat_message.thinking = thinking;
                }
            }

            response_choices.push(crate::core::types::responses::ChatChoice {
                index: index as u32,
                message: chat_message,
                finish_reason,
                logprobs: None,
            });
        }

        // Extract usage
        let usage = response
            .get("usage")
            .map(|u| serde_json::from_value(u.clone()))
            .transpose()
            .map_err(|e| ProviderError::response_parsing(PROVIDER_NAME, e.to_string()))?;

        Ok(ChatResponse {
            id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices: response_choices,
            usage,
            system_fingerprint: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

    // ==================== Provider Creation Tests ====================

    #[tokio::test]
    async fn test_provider_creation() {
        let config = OpenRouterConfig::new("test-key-1234567890")
            .with_site_url("https://example.com")
            .with_site_name("Test Site");

        let provider = OpenRouterProvider::new(config);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_provider_creation_minimal() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_name() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();
        assert_eq!(provider.name(), "openrouter");
    }

    // ==================== Capabilities Tests ====================

    #[test]
    fn test_capabilities() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ChatCompletion));
        assert!(caps.contains(&ProviderCapability::ChatCompletionStream));
        assert!(caps.contains(&ProviderCapability::FunctionCalling));
    }

    #[test]
    fn test_capabilities_tool_calling() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ToolCalling));
    }

    // ==================== Models Tests ====================

    #[test]
    fn test_models() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let models = provider.models();
        assert!(!models.is_empty());

        // Should have OpenAI models
        assert!(models.iter().any(|m| m.id.contains("openai/gpt-4")));

        // Should have Anthropic models
        assert!(models.iter().any(|m| m.id.contains("anthropic/claude")));
    }

    #[test]
    fn test_models_count() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let models = provider.models();
        // Should have multiple models
        assert!(
            models.len() >= 5,
            "Expected at least 5 models, got {}",
            models.len()
        );
    }

    #[test]
    fn test_models_have_openrouter_prefix() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let models = provider.models();
        // Most models should have provider prefix
        let prefixed = models.iter().filter(|m| m.id.contains("/")).count();
        assert!(
            prefixed > models.len() / 2,
            "Most models should have provider prefix"
        );
    }

    // ==================== Request Transformation Tests ====================

    #[test]
    fn test_request_transformation() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let request = ChatRequest {
            model: "openai/gpt-4".to_string(),
            messages: vec![],
            stream: false,
            max_tokens: Some(100),
            temperature: Some(0.7),
            ..Default::default()
        };

        let transformed = provider.transform_chat_request(request).unwrap();

        assert_eq!(transformed["model"], "openai/gpt-4");
        assert_eq!(transformed["max_tokens"], 100);
        let temp_value = transformed["temperature"].as_f64().unwrap();
        assert!(
            (temp_value - 0.7).abs() < 1e-6,
            "Expected 0.7, got {}",
            temp_value
        );
        assert_eq!(transformed["stream"], false);
    }

    #[test]
    fn test_request_transformation_with_streaming() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let request = ChatRequest {
            model: "anthropic/claude-3-opus".to_string(),
            messages: vec![],
            stream: true,
            ..Default::default()
        };

        let transformed = provider.transform_chat_request(request).unwrap();
        assert_eq!(transformed["stream"], true);
    }

    #[test]
    fn test_request_transformation_with_top_p() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let request = ChatRequest {
            model: "openai/gpt-4".to_string(),
            messages: vec![],
            stream: false,
            top_p: Some(0.9),
            ..Default::default()
        };

        let transformed = provider.transform_chat_request(request).unwrap();
        let top_p_value = transformed["top_p"].as_f64().unwrap();
        assert!((top_p_value - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_request_transformation_with_penalties() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let request = ChatRequest {
            model: "openai/gpt-4".to_string(),
            messages: vec![],
            stream: false,
            frequency_penalty: Some(0.5),
            presence_penalty: Some(0.3),
            ..Default::default()
        };

        let transformed = provider.transform_chat_request(request).unwrap();

        let freq_value = transformed["frequency_penalty"].as_f64().unwrap();
        assert!((freq_value - 0.5).abs() < 1e-6);

        let pres_value = transformed["presence_penalty"].as_f64().unwrap();
        assert!((pres_value - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_request_transformation_minimal() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let request = ChatRequest {
            model: "openai/gpt-4".to_string(),
            messages: vec![],
            stream: false,
            ..Default::default()
        };

        let transformed = provider.transform_chat_request(request).unwrap();
        assert_eq!(transformed["model"], "openai/gpt-4");
        assert!(transformed.get("max_tokens").is_none() || transformed["max_tokens"].is_null());
    }

    // ==================== Response Transformation Tests ====================

    #[test]
    fn test_response_transformation_success() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let raw_response = serde_json::json!({
            "id": "test-response-id",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        });

        let response = provider.transform_chat_response(raw_response, "openai/gpt-4");
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.id, "test-response-id");
        assert_eq!(response.choices.len(), 1);
    }

    #[test]
    fn test_response_transformation_error() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let raw_response = serde_json::json!({
            "error": {
                "code": 404,
                "message": "Model not found"
            }
        });

        let response = provider.transform_chat_response(raw_response, "invalid/model");
        assert!(response.is_err());
    }

    #[test]
    fn test_response_transformation_rate_limit() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let raw_response = serde_json::json!({
            "error": {
                "code": 429,
                "message": "Rate limit exceeded"
            }
        });

        let response = provider.transform_chat_response(raw_response, "openai/gpt-4");
        assert!(response.is_err());
    }

    #[test]
    fn test_response_transformation_auth_error() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let raw_response = serde_json::json!({
            "error": {
                "code": 401,
                "message": "Invalid API key"
            }
        });

        let response = provider.transform_chat_response(raw_response, "openai/gpt-4");
        assert!(response.is_err());
    }
}
