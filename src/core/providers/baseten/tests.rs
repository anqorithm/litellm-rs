//! Unit tests for Baseten provider

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::model::ProviderCapability;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = BasetenConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = BasetenProvider::new(config).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_provider_with_api_key() {
        let provider = BasetenProvider::with_api_key("test-key").await;
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_name() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = BasetenProvider::with_api_key("test-key").await.unwrap();
            assert_eq!(provider.name(), "baseten");
        });
    }

    #[test]
    fn test_provider_capabilities() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = BasetenProvider::with_api_key("test-key").await.unwrap();
            let capabilities = provider.capabilities();

            assert!(capabilities.contains(&ProviderCapability::ChatCompletion));
            assert!(capabilities.contains(&ProviderCapability::ChatCompletionStream));
            assert!(capabilities.contains(&ProviderCapability::ToolCalling));
        });
    }

    #[test]
    fn test_model_info() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = BasetenProvider::with_api_key("test-key").await.unwrap();
            let models = provider.models();

            // Should have at least the default custom model
            assert!(!models.is_empty());

            let default_model = models.iter().find(|m| m.id == "baseten-custom");
            assert!(default_model.is_some());

            let model = default_model.unwrap();
            assert!(model.supports_streaming);
            assert!(model.supports_tools);
        });
    }

    #[test]
    fn test_supported_openai_params() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = BasetenProvider::with_api_key("test-key").await.unwrap();

            let params = provider.get_supported_openai_params("any-model");
            assert!(params.contains(&"temperature"));
            assert!(params.contains(&"max_tokens"));
            assert!(params.contains(&"max_completion_tokens"));
            assert!(params.contains(&"tools"));
            assert!(params.contains(&"response_format"));
            assert!(params.contains(&"stream"));
            assert!(params.contains(&"top_p"));
            assert!(params.contains(&"presence_penalty"));
            assert!(params.contains(&"frequency_penalty"));
        });
    }

    #[tokio::test]
    async fn test_map_openai_params() {
        let provider = BasetenProvider::with_api_key("test-key").await.unwrap();

        let mut params = std::collections::HashMap::new();
        params.insert("max_completion_tokens".to_string(), serde_json::json!(100));
        params.insert("temperature".to_string(), serde_json::json!(0.7));

        let mapped = provider.map_openai_params(params, "test-model").await;
        assert!(mapped.is_ok());

        let result = mapped.unwrap();
        // max_completion_tokens should be mapped to max_tokens
        assert!(result.contains_key("max_tokens"));
        assert!(!result.contains_key("max_completion_tokens"));
        assert_eq!(result["max_tokens"], serde_json::json!(100));
        assert_eq!(result["temperature"], serde_json::json!(0.7));
    }

    #[test]
    fn test_api_base_for_model() {
        // Regular model should use default API
        let base = BasetenConfig::get_api_base_for_model("llama-3.1-70b");
        assert_eq!(base, "https://inference.baseten.co/v1");

        // Dedicated deployment should use model-specific URL
        let base = BasetenConfig::get_api_base_for_model("abc12345");
        assert!(base.contains("model-abc12345"));
        assert!(base.contains("api.baseten.co"));

        // With baseten/ prefix
        let base = BasetenConfig::get_api_base_for_model("baseten/xyz98765");
        assert!(base.contains("model-xyz98765"));
    }

    // Note: Error mapping tests removed - BasetenError is now a type alias to ProviderError

    #[tokio::test]
    async fn test_cost_calculation() {
        let provider = BasetenProvider::with_api_key("test-key").await.unwrap();

        // Baseten cost calculation returns 0 (depends on deployment)
        let cost = provider.calculate_cost("any-model", 1000, 1000).await;
        assert!(cost.is_ok());
        assert_eq!(cost.unwrap(), 0.0);
    }

    #[test]
    fn test_dedicated_deployment_detection() {
        // Valid 8-character alphanumeric codes
        assert!(BasetenConfig::is_dedicated_deployment("abc12345"));
        assert!(BasetenConfig::is_dedicated_deployment("ABCDEFGH"));
        assert!(BasetenConfig::is_dedicated_deployment("12345678"));
        assert!(BasetenConfig::is_dedicated_deployment("baseten/abc12345"));

        // Invalid cases
        assert!(!BasetenConfig::is_dedicated_deployment("abc1234")); // 7 chars
        assert!(!BasetenConfig::is_dedicated_deployment("abc123456")); // 9 chars
        assert!(!BasetenConfig::is_dedicated_deployment("abc-1234")); // contains hyphen
        assert!(!BasetenConfig::is_dedicated_deployment("llama-3.1-70b")); // model name
        assert!(!BasetenConfig::is_dedicated_deployment("")); // empty
        assert!(!BasetenConfig::is_dedicated_deployment("abc_1234")); // contains underscore
    }

    #[test]
    fn test_config_with_custom_api_base() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let config = BasetenConfig {
                api_key: Some("test-key".to_string()),
                api_base: Some("https://custom.baseten.co/v1".to_string()),
                ..Default::default()
            };

            let provider = BasetenProvider::new(config).await.unwrap();

            // Verify provider was created successfully with custom api_base
            assert_eq!(provider.name(), "baseten");
        });
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = BasetenConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.baseten.co/v1".to_string()),
            timeout: 60,
            max_retries: 5,
            debug: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: BasetenConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.api_key, deserialized.api_key);
        assert_eq!(config.api_base, deserialized.api_base);
        assert_eq!(config.timeout, deserialized.timeout);
        assert_eq!(config.max_retries, deserialized.max_retries);
        assert_eq!(config.debug, deserialized.debug);
    }
}
