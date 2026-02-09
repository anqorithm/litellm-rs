    use super::*;
    use crate::core::types::{chat::ChatMessage, message::MessageContent, message::MessageRole};

    fn create_test_config() -> SambanovaConfig {
        SambanovaConfig {
            api_key: "test_api_key".to_string(),
            ..Default::default()
        }
    }

    // ==================== Provider Creation Tests ====================

    #[tokio::test]
    async fn test_sambanova_provider_creation() {
        let config = SambanovaConfig {
            api_key: "test_key".to_string(),
            ..Default::default()
        };

        let provider = SambanovaProvider::new(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(LLMProvider::name(&provider), "sambanova");
        assert!(
            provider
                .capabilities()
                .contains(&ProviderCapability::ChatCompletionStream)
        );
    }

    #[tokio::test]
    async fn test_sambanova_provider_creation_custom_base() {
        let config = SambanovaConfig {
            api_key: "test_key".to_string(),
            api_base: "https://custom.sambanova.ai/v1".to_string(),
            ..Default::default()
        };

        let provider = SambanovaProvider::new(config).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_sambanova_provider_creation_no_api_key() {
        let config = SambanovaConfig::default();
        let provider = SambanovaProvider::new(config).await;
        assert!(provider.is_err());
    }

    #[tokio::test]
    async fn test_sambanova_provider_creation_empty_api_key() {
        let config = SambanovaConfig {
            api_key: "".to_string(),
            ..Default::default()
        };

        let provider = SambanovaProvider::new(config).await;
        assert!(provider.is_err());
    }

    #[tokio::test]
    async fn test_sambanova_with_api_key() {
        let provider = SambanovaProvider::with_api_key("test_key").await;
        assert!(provider.is_ok());
    }

    // ==================== Config Validation Tests ====================

    #[test]
    fn test_sambanova_config_validation() {
        let mut config = SambanovaConfig::default();
        assert!(config.validate().is_err()); // No API key

        config.api_key = "test_key".to_string();
        assert!(config.validate().is_ok());

        config.timeout_seconds = 0;
        assert!(config.validate().is_err()); // Invalid timeout

        config.timeout_seconds = 30;
        config.max_retries = 11;
        assert!(config.validate().is_err()); // Too many retries
    }

    #[test]
    fn test_sambanova_config_default() {
        let config = SambanovaConfig::default();

        assert_eq!(config.api_key, "");
        assert_eq!(config.api_base, "https://api.sambanova.ai/v1");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_sambanova_config_provider_config_trait() {
        let config = create_test_config();

        assert_eq!(config.api_key(), Some("test_api_key"));
        assert_eq!(config.api_base(), Some("https://api.sambanova.ai/v1"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(30));
        assert_eq!(config.max_retries(), 3);
    }

    // ==================== Provider Capabilities Tests ====================

    #[tokio::test]
    async fn test_provider_name() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();
        assert_eq!(provider.name(), "sambanova");
    }

    #[tokio::test]
    async fn test_provider_capabilities() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();
        let caps = provider.capabilities();

        assert!(caps.contains(&ProviderCapability::ChatCompletion));
        assert!(caps.contains(&ProviderCapability::ChatCompletionStream));
        assert!(caps.contains(&ProviderCapability::ToolCalling));
        assert!(caps.contains(&ProviderCapability::Embeddings));
        assert_eq!(caps.len(), 4);
    }

    #[tokio::test]
    async fn test_provider_models() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();
        let models = provider.models();

        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "Meta-Llama-3.1-8B-Instruct"));
        assert!(models.iter().any(|m| m.id == "Meta-Llama-3.1-70B-Instruct"));
        assert!(
            models
                .iter()
                .any(|m| m.id == "Meta-Llama-3.1-405B-Instruct")
        );
    }

    #[tokio::test]
    async fn test_provider_models_have_pricing() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();
        let models = provider.models();

        for model in models {
            assert_eq!(model.provider, "sambanova");
            assert!(model.input_cost_per_1k_tokens.is_some());
            assert!(model.output_cost_per_1k_tokens.is_some());
        }
    }

    // ==================== Supported Params Tests ====================

    #[tokio::test]
    async fn test_get_supported_openai_params_instruct() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();
        let params = provider.get_supported_openai_params("Meta-Llama-3.1-70B-Instruct");

        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"top_p"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"stream"));
        assert!(params.contains(&"stop"));
        assert!(params.contains(&"tools"));
        assert!(params.contains(&"tool_choice"));
    }

    // ==================== Map OpenAI Params Tests ====================

    #[tokio::test]
    async fn test_map_openai_params_max_completion_tokens() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();

        let mut params = HashMap::new();
        params.insert("max_completion_tokens".to_string(), serde_json::json!(100));

        let mapped = provider
            .map_openai_params(params, "Meta-Llama-3.1-70B-Instruct")
            .await
            .unwrap();

        assert!(!mapped.contains_key("max_completion_tokens"));
        assert!(mapped.contains_key("max_tokens"));
        assert_eq!(mapped.get("max_tokens").unwrap(), &serde_json::json!(100));
    }

    #[tokio::test]
    async fn test_map_openai_params_passthrough() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();

        let mut params = HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(0.7));
        params.insert("max_tokens".to_string(), serde_json::json!(100));
        params.insert("top_p".to_string(), serde_json::json!(0.9));

        let mapped = provider
            .map_openai_params(params, "Meta-Llama-3.1-70B-Instruct")
            .await
            .unwrap();

        assert_eq!(mapped.get("temperature").unwrap(), &serde_json::json!(0.7));
        assert_eq!(mapped.get("max_tokens").unwrap(), &serde_json::json!(100));
        assert_eq!(mapped.get("top_p").unwrap(), &serde_json::json!(0.9));
    }

    #[tokio::test]
    async fn test_map_openai_params_unsupported_filtered() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();

        let mut params = HashMap::new();
        params.insert("unsupported_param".to_string(), serde_json::json!("value"));
        params.insert("temperature".to_string(), serde_json::json!(0.5));

        let mapped = provider
            .map_openai_params(params, "Meta-Llama-3.1-70B-Instruct")
            .await
            .unwrap();

        assert!(!mapped.contains_key("unsupported_param"));
        assert!(mapped.contains_key("temperature"));
    }

    // ==================== Transform Request Tests ====================

    #[tokio::test]
    async fn test_transform_request_basic() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();

        let request = ChatRequest {
            model: "Meta-Llama-3.1-70B-Instruct".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            ..Default::default()
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;

        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed["model"], "Meta-Llama-3.1-70B-Instruct");
        assert!(transformed["messages"].is_array());
    }

    // ==================== Is Embedding Model Tests ====================

    #[tokio::test]
    async fn test_is_embedding_model() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();

        assert!(provider.is_embedding_model("sambanova-embed"));
        assert!(provider.is_embedding_model("text-embedding-model"));
        assert!(!provider.is_embedding_model("Meta-Llama-3.1-70B-Instruct"));
    }

    // ==================== Supports Function Calling Tests ====================

    #[tokio::test]
    async fn test_supports_function_calling() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();

        assert!(provider.supports_function_calling("Meta-Llama-3.1-70B-Instruct"));
        assert!(provider.supports_function_calling("SomeModel-Chat"));
        assert!(!provider.supports_function_calling("sambanova-embed"));
    }

    // ==================== Cost Calculation Tests ====================

    #[tokio::test]
    async fn test_calculate_cost_known_model() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();

        let cost = provider
            .calculate_cost("Meta-Llama-3.1-70B-Instruct", 1000, 500)
            .await;
        assert!(cost.is_ok());

        let cost_value = cost.unwrap();
        // Meta-Llama-3.1-70B: $0.0005 input, $0.001 output per 1k
        // (1000/1000 * 0.0005) + (500/1000 * 0.001) = 0.0005 + 0.0005 = 0.001
        assert!((cost_value - 0.001).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_calculate_cost_embed_model() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();

        let cost = provider.calculate_cost("sambanova-embed", 1000, 0).await;
        assert!(cost.is_ok());

        let cost_value = cost.unwrap();
        // sambanova-embed: $0.0001 input, $0.0 output per 1k
        assert!((cost_value - 0.0001).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_calculate_cost_unknown_model() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();

        let cost = provider.calculate_cost("unknown-model", 1000, 500).await;
        assert!(cost.is_err());
    }

    #[tokio::test]
    async fn test_calculate_cost_zero_tokens() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();

        let cost = provider
            .calculate_cost("Meta-Llama-3.1-70B-Instruct", 0, 0)
            .await;
        assert!(cost.is_ok());
        assert!((cost.unwrap() - 0.0).abs() < 0.0001);
    }

    // ==================== Error Mapper Tests ====================

    #[test]
    fn test_error_mapper_authentication() {
        let mapper = SambanovaErrorMapper;
        let error = mapper.map_http_error(401, "Unauthorized");

        match error {
            ProviderError::Authentication { provider, .. } => {
                assert_eq!(provider, "sambanova");
            }
            _ => panic!("Expected Authentication error"),
        }
    }

    #[test]
    fn test_error_mapper_rate_limit() {
        let mapper = SambanovaErrorMapper;
        let error = mapper.map_http_error(429, "Rate limit exceeded");

        match error {
            ProviderError::RateLimit { provider, .. } => {
                assert_eq!(provider, "sambanova");
            }
            _ => panic!("Expected RateLimit error"),
        }
    }

    #[test]
    fn test_error_mapper_network_error() {
        let mapper = SambanovaErrorMapper;
        let error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
        let mapped = mapper.map_network_error(&error);

        match mapped {
            ProviderError::Network { provider, .. } => {
                assert_eq!(provider, "sambanova");
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_error_mapper_parsing_error() {
        let mapper = SambanovaErrorMapper;
        let error = std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid JSON");
        let mapped = mapper.map_parsing_error(&error);

        match mapped {
            ProviderError::ResponseParsing { provider, .. } => {
                assert_eq!(provider, "sambanova");
            }
            _ => panic!("Expected ResponseParsing error"),
        }
    }

    #[test]
    fn test_error_mapper_timeout_error() {
        let mapper = SambanovaErrorMapper;
        let mapped = mapper.map_timeout_error(std::time::Duration::from_secs(30));

        match mapped {
            ProviderError::Timeout { provider, .. } => {
                assert_eq!(provider, "sambanova");
            }
            _ => panic!("Expected Timeout error"),
        }
    }

    // ==================== Get Error Mapper Tests ====================

    #[tokio::test]
    async fn test_get_error_mapper() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();
        let _mapper = provider.get_error_mapper();
        // Just verify it doesn't panic
    }

    // ==================== Clone/Debug Tests ====================

    #[tokio::test]
    async fn test_provider_clone() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();
        let cloned = provider.clone();

        assert_eq!(provider.name(), cloned.name());
        assert_eq!(provider.models().len(), cloned.models().len());
    }

    #[tokio::test]
    async fn test_provider_debug() {
        let provider = SambanovaProvider::new(create_test_config()).await.unwrap();
        let debug_str = format!("{:?}", provider);

        assert!(debug_str.contains("SambanovaProvider"));
    }

    #[test]
    fn test_config_clone() {
        let config = create_test_config();
        let cloned = config.clone();

        assert_eq!(config.api_key, cloned.api_key);
        assert_eq!(config.api_base, cloned.api_base);
    }

    #[test]
    fn test_config_debug() {
        let config = create_test_config();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("SambanovaConfig"));
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_config_serialization() {
        let config = create_test_config();
        let json = serde_json::to_value(&config).unwrap();

        assert_eq!(json["api_key"], "test_api_key");
        assert_eq!(json["api_base"], "https://api.sambanova.ai/v1");
        assert_eq!(json["timeout_seconds"], 30);
        assert_eq!(json["max_retries"], 3);
    }

    #[test]
    fn test_config_deserialization() {
        let json = r#"{
            "api_key": "my_key",
            "api_base": "https://custom.api.com",
            "timeout_seconds": 60,
            "max_retries": 5
        }"#;

        let config: SambanovaConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, "my_key");
        assert_eq!(config.api_base, "https://custom.api.com");
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_retries, 5);
    }
