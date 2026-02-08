//! Tests for Fireworks AI Provider

use super::*;

#[cfg(test)]
mod provider_tests {
    use super::*;
    use crate::core::types::{model::ProviderCapability, context::RequestContext};
    use crate::core::types::{ChatMessage, ChatRequest, MessageContent, MessageRole};

    async fn create_test_provider() -> FireworksProvider {
        let config = FireworksConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        FireworksProvider::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_provider_creation() {
        let config = FireworksConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        let provider = FireworksProvider::new(config).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_provider_with_api_key() {
        let provider = FireworksProvider::with_api_key("test-key").await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_provider_name() {
        let provider = create_test_provider().await;
        assert_eq!(provider.name(), "fireworks");
    }

    #[tokio::test]
    async fn test_provider_capabilities() {
        let provider = create_test_provider().await;
        let capabilities = provider.capabilities();
        assert!(capabilities.contains(&ProviderCapability::ChatCompletion));
        assert!(capabilities.contains(&ProviderCapability::ChatCompletionStream));
        assert!(capabilities.contains(&ProviderCapability::ToolCalling));
    }

    #[tokio::test]
    async fn test_provider_models() {
        let provider = create_test_provider().await;
        let models = provider.models();
        assert!(!models.is_empty());

        // Check that we have Llama models
        assert!(models.iter().any(|m| m.id.contains("llama")));
    }

    #[tokio::test]
    async fn test_get_supported_openai_params_basic() {
        let provider = create_test_provider().await;
        let params = provider.get_supported_openai_params("gemma2-9b-it");
        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"stream"));
    }

    #[tokio::test]
    async fn test_get_supported_openai_params_with_tools() {
        let provider = create_test_provider().await;
        let params = provider.get_supported_openai_params("llama-v3p1-70b-instruct");
        assert!(params.contains(&"tools"));
        assert!(params.contains(&"tool_choice"));
    }

    #[tokio::test]
    async fn test_get_supported_openai_params_with_reasoning() {
        let provider = create_test_provider().await;
        let params = provider.get_supported_openai_params("qwen3-8b");
        assert!(params.contains(&"reasoning_effort"));
    }

    #[tokio::test]
    async fn test_map_openai_params() {
        let provider = create_test_provider().await;
        let mut params = std::collections::HashMap::new();
        params.insert("max_completion_tokens".to_string(), serde_json::json!(1000));
        params.insert("tool_choice".to_string(), serde_json::json!("required"));

        let mapped = provider
            .map_openai_params(params, "llama-v3p1-70b-instruct")
            .await
            .unwrap();

        // max_completion_tokens should be mapped to max_tokens
        assert!(mapped.contains_key("max_tokens"));
        assert!(!mapped.contains_key("max_completion_tokens"));

        // tool_choice "required" should be mapped to "any"
        assert_eq!(mapped.get("tool_choice"), Some(&serde_json::json!("any")));
    }

    #[tokio::test]
    async fn test_calculate_cost() {
        let provider = create_test_provider().await;
        let cost = provider
            .calculate_cost(
                "accounts/fireworks/models/llama-v3p1-70b-instruct",
                1000,
                500,
            )
            .await
            .unwrap();

        // Llama 3.1 70B costs $0.9 per million tokens
        // 1000 input + 500 output = 1500 tokens
        // Cost = 1500 * (0.9 / 1_000_000) = 0.00135
        assert!((cost - 0.00135).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_calculate_cost_unknown_model() {
        let provider = create_test_provider().await;
        let result = provider.calculate_cost("unknown-model", 1000, 500).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transform_request() {
        let provider = create_test_provider().await;

        let request = ChatRequest {
            model: "llama-v3p1-70b-instruct".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                metadata: None,
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            stream: false,
            tools: None,
            tool_choice: None,
            response_format: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            n: None,
            logprobs: None,
            top_logprobs: None,
            user: None,
            metadata: None,
            max_completion_tokens: None,
            seed: None,
            logit_bias: None,
            reasoning_effort: None,
            thinking: None,
            parallel_tool_calls: None,
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;
        assert!(result.is_ok());

        let json = result.unwrap();
        // Model should be formatted
        assert!(
            json["model"]
                .as_str()
                .unwrap()
                .starts_with("accounts/fireworks/models/")
        );
    }

    #[tokio::test]
    async fn test_transform_response() {
        let provider = create_test_provider().await;

        let response_json = serde_json::json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "accounts/fireworks/models/llama-v3p1-70b-instruct",
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

        let raw_response = serde_json::to_vec(&response_json).unwrap();
        let result = provider
            .transform_response(&raw_response, "llama-v3p1-70b-instruct", "req-123")
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        // Model should be prefixed with provider name
        assert!(response.model.unwrap().starts_with("fireworks_ai/"));
    }

    #[tokio::test]
    async fn test_supports_model() {
        let provider = create_test_provider().await;

        // Test that known models are supported
        assert!(provider.supports_model("accounts/fireworks/models/llama-v3p1-70b-instruct"));
    }

    #[tokio::test]
    async fn test_supports_tools() {
        let provider = create_test_provider().await;
        assert!(provider.supports_tools());
    }

    #[tokio::test]
    async fn test_supports_streaming() {
        let provider = create_test_provider().await;
        assert!(provider.supports_streaming());
    }

}

#[cfg(test)]
mod model_info_tests {
    use super::model_info::*;

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());
    }

    #[test]
    fn test_format_model_id() {
        assert_eq!(
            format_model_id("llama-v3p1-70b-instruct"),
            "accounts/fireworks/models/llama-v3p1-70b-instruct"
        );

        // Already formatted model should not change
        assert_eq!(
            format_model_id("accounts/fireworks/models/llama-v3p1-70b-instruct"),
            "accounts/fireworks/models/llama-v3p1-70b-instruct"
        );

        // Custom model with hash should not change
        assert_eq!(format_model_id("my-model#v1"), "my-model#v1");
    }

    #[test]
    fn test_is_reasoning_model() {
        assert!(is_reasoning_model("qwen3-8b"));
        assert!(is_reasoning_model("qwen3-32b"));
        assert!(is_reasoning_model("deepseek-v3"));
        assert!(!is_reasoning_model("llama-v3p1-70b-instruct"));
    }

    #[test]
    fn test_supports_function_calling() {
        assert!(supports_function_calling("llama-v3p1-70b-instruct"));
        assert!(supports_function_calling("firefunction-v2"));
    }

    #[test]
    fn test_supports_tool_choice() {
        assert!(supports_tool_choice("llama-v3p1-70b-instruct"));
    }
}

#[cfg(test)]
mod config_tests {
    use super::config::*;
    use crate::core::traits::ProviderConfig;

    #[test]
    fn test_config_default() {
        let config = FireworksConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_config_validation_with_key() {
        let config = FireworksConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_zero_timeout() {
        let config = FireworksConfig {
            api_key: Some("test-key".to_string()),
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_get_api_base_default() {
        let config = FireworksConfig::default();
        assert_eq!(
            config.get_api_base(),
            "https://api.fireworks.ai/inference/v1"
        );
    }

    #[test]
    fn test_get_api_base_custom() {
        let config = FireworksConfig {
            api_base: Some("https://custom.api.com".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_base(), "https://custom.api.com");
    }
}
