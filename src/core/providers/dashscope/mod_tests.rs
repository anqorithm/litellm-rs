    use super::*;
    use crate::core::types::{chat::ChatMessage, message::MessageContent, message::MessageRole};

    fn create_test_config() -> DashscopeConfig {
        DashscopeConfig {
            api_key: "test_api_key".to_string(),
            ..Default::default()
        }
    }

    // ==================== Provider Creation Tests ====================

    #[tokio::test]
    async fn test_dashscope_provider_creation() {
        let config = DashscopeConfig {
            api_key: "test_key".to_string(),
            ..Default::default()
        };

        let provider = DashscopeProvider::new(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "dashscope");
        assert!(
            provider
                .capabilities()
                .contains(&ProviderCapability::ChatCompletionStream)
        );
    }

    #[tokio::test]
    async fn test_dashscope_provider_creation_custom_base() {
        let config = DashscopeConfig {
            api_key: "test_key".to_string(),
            api_base: "https://custom.aliyuncs.com/v1".to_string(),
            ..Default::default()
        };

        let provider = DashscopeProvider::new(config).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_dashscope_provider_creation_no_api_key() {
        let config = DashscopeConfig::default();
        let provider = DashscopeProvider::new(config).await;
        assert!(provider.is_err());
    }

    #[tokio::test]
    async fn test_dashscope_provider_creation_empty_api_key() {
        let config = DashscopeConfig {
            api_key: "".to_string(),
            ..Default::default()
        };

        let provider = DashscopeProvider::new(config).await;
        assert!(provider.is_err());
    }

    // ==================== Config Validation Tests ====================

    #[test]
    fn test_dashscope_config_validation() {
        let mut config = DashscopeConfig::default();
        assert!(config.validate().is_err()); // No API key

        config.api_key = "test_key".to_string();
        assert!(config.validate().is_ok());

        config.timeout_seconds = 0;
        assert!(config.validate().is_err()); // Invalid timeout

        config.timeout_seconds = 60;
        config.max_retries = 11;
        assert!(config.validate().is_err()); // Too many retries
    }

    #[test]
    fn test_dashscope_config_default() {
        let config = DashscopeConfig::default();

        assert_eq!(config.api_key, "");
        assert_eq!(
            config.api_base,
            "https://dashscope.aliyuncs.com/compatible-mode/v1"
        );
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_dashscope_config_provider_config_trait() {
        let config = create_test_config();

        assert_eq!(config.api_key(), Some("test_api_key"));
        assert_eq!(
            config.api_base(),
            Some("https://dashscope.aliyuncs.com/compatible-mode/v1")
        );
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 3);
    }

    #[test]
    fn test_dashscope_config_validation_max_retries_boundary() {
        let mut config = create_test_config();

        config.max_retries = 10;
        assert!(config.validate().is_ok());

        config.max_retries = 11;
        assert!(config.validate().is_err());
    }

    // ==================== Provider Capabilities Tests ====================

    #[tokio::test]
    async fn test_provider_name() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();
        assert_eq!(provider.name(), "dashscope");
    }

    #[tokio::test]
    async fn test_provider_capabilities() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();
        let caps = provider.capabilities();

        assert!(caps.contains(&ProviderCapability::ChatCompletion));
        assert!(caps.contains(&ProviderCapability::ChatCompletionStream));
        assert!(caps.contains(&ProviderCapability::FunctionCalling));
        assert_eq!(caps.len(), 3);
    }

    #[tokio::test]
    async fn test_provider_models() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();
        let models = provider.models();

        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "qwen-turbo"));
        assert!(models.iter().any(|m| m.id == "qwen-plus"));
        assert!(models.iter().any(|m| m.id == "qwen-max"));
        assert!(models.iter().any(|m| m.id == "qwen-vl-plus"));
    }

    #[tokio::test]
    async fn test_provider_models_have_pricing() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();
        let models = provider.models();

        for model in models {
            assert_eq!(model.provider, "dashscope");
            assert_eq!(model.currency, "CNY");
            assert!(model.input_cost_per_1k_tokens.is_some());
            assert!(model.output_cost_per_1k_tokens.is_some());
        }
    }

    #[tokio::test]
    async fn test_provider_models_context_lengths() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();
        let models = provider.models();

        let qwen_turbo = models.iter().find(|m| m.id == "qwen-turbo").unwrap();
        assert_eq!(qwen_turbo.max_context_length, 131072);

        let qwen_max = models.iter().find(|m| m.id == "qwen-max").unwrap();
        assert_eq!(qwen_max.max_context_length, 32768);

        let qwen_max_long = models
            .iter()
            .find(|m| m.id == "qwen-max-longcontext")
            .unwrap();
        assert_eq!(qwen_max_long.max_context_length, 1000000);
    }

    // ==================== URL Building Tests ====================

    #[tokio::test]
    async fn test_build_chat_url_default() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();
        let url = provider.build_chat_url();
        assert_eq!(
            url,
            "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
        );
    }

    #[tokio::test]
    async fn test_build_chat_url_with_trailing_slash() {
        let config = DashscopeConfig {
            api_key: "test_key".to_string(),
            api_base: "https://dashscope.aliyuncs.com/compatible-mode/v1/".to_string(),
            ..Default::default()
        };
        let provider = DashscopeProvider::new(config).await.unwrap();
        let url = provider.build_chat_url();
        assert_eq!(
            url,
            "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
        );
    }

    #[tokio::test]
    async fn test_build_chat_url_already_complete() {
        let config = DashscopeConfig {
            api_key: "test_key".to_string(),
            api_base: "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
                .to_string(),
            ..Default::default()
        };
        let provider = DashscopeProvider::new(config).await.unwrap();
        let url = provider.build_chat_url();
        assert_eq!(
            url,
            "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
        );
    }

    // ==================== Supported Params Tests ====================

    #[tokio::test]
    async fn test_get_supported_openai_params() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();
        let params = provider.get_supported_openai_params("qwen-turbo");

        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"top_p"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"stream"));
        assert!(params.contains(&"stop"));
        assert!(params.contains(&"presence_penalty"));
        assert!(params.contains(&"frequency_penalty"));
        assert!(params.contains(&"n"));
        assert!(params.contains(&"user"));
        assert!(params.contains(&"tools"));
        assert!(params.contains(&"tool_choice"));
        assert!(params.contains(&"seed"));
        assert!(params.contains(&"top_k")); // Qwen-specific
    }

    // ==================== Map OpenAI Params Tests ====================

    #[tokio::test]
    async fn test_map_openai_params_passthrough() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();

        let mut params = HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(0.7));
        params.insert("max_tokens".to_string(), serde_json::json!(100));
        params.insert("top_p".to_string(), serde_json::json!(0.9));

        let mapped = provider
            .map_openai_params(params.clone(), "qwen-turbo")
            .await
            .unwrap();

        // Dashscope is OpenAI-compatible, should pass through
        assert_eq!(mapped, params);
    }

    // ==================== Transform Request Tests ====================

    #[tokio::test]
    async fn test_transform_request_basic() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();

        let request = ChatRequest {
            model: "qwen-turbo".to_string(),
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
        assert_eq!(transformed["model"], "qwen-turbo");
        assert!(transformed["messages"].is_array());
    }

    #[tokio::test]
    async fn test_transform_request_with_temperature() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();

        let request = ChatRequest {
            model: "qwen-plus".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            temperature: Some(0.7),
            ..Default::default()
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;

        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert!(transformed.get("temperature").is_some());
    }

    // ==================== Embeddings Not Supported Test ====================

    #[tokio::test]
    async fn test_embeddings_not_supported() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();

        let request = crate::core::types::embedding::EmbeddingRequest {
            model: "qwen-turbo".to_string(),
            input: crate::core::types::embedding::EmbeddingInput::Text("test".to_string()),
            encoding_format: None,
            dimensions: None,
            user: None,
            task_type: None,
        };

        let context = RequestContext::default();
        let result = provider.embeddings(request, context).await;

        assert!(result.is_err());
    }

    // ==================== Cost Calculation Tests ====================

    #[tokio::test]
    async fn test_calculate_cost_qwen_turbo() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();

        let cost = provider.calculate_cost("qwen-turbo", 1000, 500).await;
        assert!(cost.is_ok());

        let cost_value = cost.unwrap();
        // qwen-turbo: 0.0008 CNY input, 0.002 CNY output per 1k
        // (1000/1000 * 0.0008) + (500/1000 * 0.002) = 0.0008 + 0.001 = 0.0018
        assert!((cost_value - 0.0018).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_calculate_cost_qwen_max() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();

        let cost = provider.calculate_cost("qwen-max", 1000, 500).await;
        assert!(cost.is_ok());

        let cost_value = cost.unwrap();
        // qwen-max: 0.02 CNY input, 0.06 CNY output per 1k
        // (1000/1000 * 0.02) + (500/1000 * 0.06) = 0.02 + 0.03 = 0.05
        assert!((cost_value - 0.05).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_calculate_cost_unknown_model() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();

        let cost = provider.calculate_cost("unknown-model", 1000, 500).await;
        assert!(cost.is_err());
    }

    #[tokio::test]
    async fn test_calculate_cost_zero_tokens() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();

        let cost = provider.calculate_cost("qwen-turbo", 0, 0).await;
        assert!(cost.is_ok());
        assert!((cost.unwrap() - 0.0).abs() < 0.0001);
    }

    // ==================== Error Mapper Tests ====================

    #[test]
    fn test_error_mapper_authentication() {
        let mapper = DashscopeErrorMapper;
        let error = mapper.map_http_error(401, "Unauthorized");

        match error {
            ProviderError::Authentication { provider, .. } => {
                assert_eq!(provider, "dashscope");
            }
            _ => panic!("Expected Authentication error"),
        }
    }

    #[test]
    fn test_error_mapper_rate_limit() {
        let mapper = DashscopeErrorMapper;
        let error = mapper.map_http_error(429, "Rate limit exceeded");

        match error {
            ProviderError::RateLimit { provider, .. } => {
                assert_eq!(provider, "dashscope");
            }
            _ => panic!("Expected RateLimit error"),
        }
    }

    #[test]
    fn test_error_mapper_network_error() {
        let mapper = DashscopeErrorMapper;
        let error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
        let mapped = mapper.map_network_error(&error);

        match mapped {
            ProviderError::Network { provider, .. } => {
                assert_eq!(provider, "dashscope");
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_error_mapper_parsing_error() {
        let mapper = DashscopeErrorMapper;
        let error = std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid JSON");
        let mapped = mapper.map_parsing_error(&error);

        match mapped {
            ProviderError::ResponseParsing { provider, .. } => {
                assert_eq!(provider, "dashscope");
            }
            _ => panic!("Expected ResponseParsing error"),
        }
    }

    #[test]
    fn test_error_mapper_timeout_error() {
        let mapper = DashscopeErrorMapper;
        let mapped = mapper.map_timeout_error(std::time::Duration::from_secs(60));

        match mapped {
            ProviderError::Timeout { provider, .. } => {
                assert_eq!(provider, "dashscope");
            }
            _ => panic!("Expected Timeout error"),
        }
    }

    // ==================== Get Error Mapper Tests ====================

    #[tokio::test]
    async fn test_get_error_mapper() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();
        let _mapper = provider.get_error_mapper();
        // Just verify it doesn't panic
    }

    // ==================== Clone/Debug Tests ====================

    #[tokio::test]
    async fn test_provider_clone() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();
        let cloned = provider.clone();

        assert_eq!(provider.name(), cloned.name());
        assert_eq!(provider.models().len(), cloned.models().len());
    }

    #[tokio::test]
    async fn test_provider_debug() {
        let provider = DashscopeProvider::new(create_test_config()).await.unwrap();
        let debug_str = format!("{:?}", provider);

        assert!(debug_str.contains("DashscopeProvider"));
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

        assert!(debug_str.contains("DashscopeConfig"));
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_config_serialization() {
        let config = create_test_config();
        let json = serde_json::to_value(&config).unwrap();

        assert_eq!(json["api_key"], "test_api_key");
        assert_eq!(
            json["api_base"],
            "https://dashscope.aliyuncs.com/compatible-mode/v1"
        );
        assert_eq!(json["timeout_seconds"], 60);
        assert_eq!(json["max_retries"], 3);
    }

    #[test]
    fn test_config_deserialization() {
        let json = r#"{
            "api_key": "my_key",
            "api_base": "https://custom.api.com",
            "timeout_seconds": 120,
            "max_retries": 5
        }"#;

        let config: DashscopeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, "my_key");
        assert_eq!(config.api_base, "https://custom.api.com");
        assert_eq!(config.timeout_seconds, 120);
        assert_eq!(config.max_retries, 5);
    }

    // ==================== Static Capabilities Constant Tests ====================

    #[test]
    fn test_dashscope_capabilities_constant() {
        assert!(DASHSCOPE_CAPABILITIES.contains(&ProviderCapability::ChatCompletion));
        assert!(DASHSCOPE_CAPABILITIES.contains(&ProviderCapability::ChatCompletionStream));
        assert!(DASHSCOPE_CAPABILITIES.contains(&ProviderCapability::FunctionCalling));
        assert_eq!(DASHSCOPE_CAPABILITIES.len(), 3);
    }
