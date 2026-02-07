//! Unit tests for Clarifai provider

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::ProviderCapability;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = ClarifaiConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = ClarifaiProvider::new(config).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_provider_with_api_key() {
        let provider = ClarifaiProvider::with_api_key("test-key").await;
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_name() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = ClarifaiProvider::with_api_key("test-key").await.unwrap();
            assert_eq!(provider.name(), "clarifai");
        });
    }

    #[test]
    fn test_provider_capabilities() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = ClarifaiProvider::with_api_key("test-key").await.unwrap();
            let capabilities = provider.capabilities();

            assert!(capabilities.contains(&ProviderCapability::ChatCompletion));
            assert!(capabilities.contains(&ProviderCapability::ChatCompletionStream));
            assert!(capabilities.contains(&ProviderCapability::ToolCalling));
        });
    }

    #[test]
    fn test_model_info() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = ClarifaiProvider::with_api_key("test-key").await.unwrap();
            let models = provider.models();

            // Should have at least the default custom model
            assert!(!models.is_empty());

            let default_model = models.iter().find(|m| m.id == "clarifai-custom");
            assert!(default_model.is_some());

            let model = default_model.unwrap();
            assert!(model.supports_streaming);
            assert!(model.supports_tools);
        });
    }

    #[test]
    fn test_supported_openai_params() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = ClarifaiProvider::with_api_key("test-key").await.unwrap();

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
        let provider = ClarifaiProvider::with_api_key("test-key").await.unwrap();

        let mut params = std::collections::HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(0.7));
        params.insert("max_tokens".to_string(), serde_json::json!(100));

        let mapped = provider
            .map_openai_params(params.clone(), "test-model")
            .await;
        assert!(mapped.is_ok());

        let result = mapped.unwrap();
        // Clarifai passes through OpenAI params as-is
        assert_eq!(result["temperature"], serde_json::json!(0.7));
        assert_eq!(result["max_tokens"], serde_json::json!(100));
    }

    #[test]
    fn test_model_url_conversion() {
        // Valid model format
        let url = ClarifaiConfig::get_model_url("openai.chat-completion.gpt-4");
        assert_eq!(
            url,
            Some("https://clarifai.com/openai/chat-completion/models/gpt-4".to_string())
        );

        // Invalid format
        assert!(ClarifaiConfig::get_model_url("invalid-model").is_none());
        assert!(ClarifaiConfig::get_model_url("user.app").is_none());
    }

    #[test]
    fn test_model_format_validation() {
        // Valid formats
        assert!(ClarifaiConfig::is_valid_model_format("user.app.model"));
        assert!(ClarifaiConfig::is_valid_model_format(
            "openai.chat-completion.gpt-4"
        ));
        assert!(ClarifaiConfig::is_valid_model_format("anthropic.ai.claude"));

        // Invalid formats
        assert!(!ClarifaiConfig::is_valid_model_format("user.app"));
        assert!(!ClarifaiConfig::is_valid_model_format("singlepart"));
        assert!(!ClarifaiConfig::is_valid_model_format("user..model"));
        assert!(!ClarifaiConfig::is_valid_model_format(".app.model"));
        assert!(!ClarifaiConfig::is_valid_model_format("user.app."));
    }

    // Note: test_transform_model removed - transform_model is now a private method

    // Note: Error mapping tests removed - ClarifaiError is now a type alias to ProviderError

    #[tokio::test]
    async fn test_cost_calculation() {
        let provider = ClarifaiProvider::with_api_key("test-key").await.unwrap();

        // Clarifai cost calculation returns 0 (depends on deployment)
        let cost = provider.calculate_cost("any-model", 1000, 1000).await;
        assert!(cost.is_ok());
        assert_eq!(cost.unwrap(), 0.0);
    }

    #[test]
    fn test_config_with_custom_api_base() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let config = ClarifaiConfig {
                api_key: Some("test-key".to_string()),
                api_base: Some("https://custom.clarifai.com/v1".to_string()),
                ..Default::default()
            };

            let provider = ClarifaiProvider::new(config.clone()).await.unwrap();

            // Verify provider was created successfully
            assert_eq!(provider.name(), "clarifai");
        });
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = ClarifaiConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.clarifai.com/v1".to_string()),
            timeout: 60,
            max_retries: 5,
            debug: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ClarifaiConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.api_key, deserialized.api_key);
        assert_eq!(config.api_base, deserialized.api_base);
        assert_eq!(config.timeout, deserialized.timeout);
        assert_eq!(config.max_retries, deserialized.max_retries);
        assert_eq!(config.debug, deserialized.debug);
    }

    #[test]
    fn test_api_base_default() {
        let config = ClarifaiConfig::default();
        assert_eq!(
            config.get_api_base(),
            "https://api.clarifai.com/v2/ext/openai/v1"
        );
    }
}
