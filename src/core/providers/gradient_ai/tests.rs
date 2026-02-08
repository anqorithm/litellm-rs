//! Unit tests for Gradient AI provider

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::model::ProviderCapability;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = GradientAIConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = GradientAIProvider::new(config).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_provider_with_api_key() {
        let provider = GradientAIProvider::with_api_key("test-key").await;
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_name() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = GradientAIProvider::with_api_key("test-key").await.unwrap();
            assert_eq!(provider.name(), "gradient_ai");
        });
    }

    #[test]
    fn test_provider_capabilities() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = GradientAIProvider::with_api_key("test-key").await.unwrap();
            let capabilities = provider.capabilities();

            assert!(capabilities.contains(&ProviderCapability::ChatCompletion));
            assert!(capabilities.contains(&ProviderCapability::ChatCompletionStream));
        });
    }

    #[test]
    fn test_model_info() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = GradientAIProvider::with_api_key("test-key").await.unwrap();
            let models = provider.models();

            // Should have at least the default agent model
            assert!(!models.is_empty());

            let default_model = models.iter().find(|m| m.id == "gradient-ai-agent");
            assert!(default_model.is_some());

            let model = default_model.unwrap();
            assert!(model.supports_streaming);
        });
    }

    #[test]
    fn test_supported_openai_params() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = GradientAIProvider::with_api_key("test-key").await.unwrap();

            let params = provider.get_supported_openai_params("any-model");
            assert!(params.contains(&"temperature"));
            assert!(params.contains(&"max_tokens"));
            assert!(params.contains(&"max_completion_tokens"));
            assert!(params.contains(&"stream"));
            assert!(params.contains(&"top_p"));
            assert!(params.contains(&"presence_penalty"));
            assert!(params.contains(&"frequency_penalty"));

            // Gradient AI specific params
            assert!(params.contains(&"k"));
            assert!(params.contains(&"kb_filters"));
            assert!(params.contains(&"retrieval_method"));
            assert!(params.contains(&"instruction_override"));
            assert!(params.contains(&"provide_citations"));
        });
    }

    #[tokio::test]
    async fn test_map_openai_params() {
        let provider = GradientAIProvider::with_api_key("test-key").await.unwrap();

        let mut params = std::collections::HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(0.7));
        params.insert("max_tokens".to_string(), serde_json::json!(100));
        params.insert("k".to_string(), serde_json::json!(5));

        let mapped = provider
            .map_openai_params(params.clone(), "test-model")
            .await;
        assert!(mapped.is_ok());

        let result = mapped.unwrap();
        assert_eq!(result["temperature"], serde_json::json!(0.7));
        assert_eq!(result["max_tokens"], serde_json::json!(100));
        assert_eq!(result["k"], serde_json::json!(5));
    }

    #[test]
    fn test_complete_url_default() {
        let config = GradientAIConfig::default();
        let url = config.get_complete_url();
        assert!(url.contains("inference.do-ai.run"));
        assert!(url.contains("/v1/chat/completions"));
    }

    #[test]
    fn test_complete_url_with_api_base() {
        let config = GradientAIConfig {
            api_base: Some("https://custom.gradient.ai".to_string()),
            ..Default::default()
        };
        let url = config.get_complete_url();
        assert_eq!(url, "https://custom.gradient.ai/api/v1/chat/completions");
    }

    #[test]
    fn test_complete_url_with_agent_endpoint() {
        let config = GradientAIConfig {
            agent_endpoint: Some("https://agent.gradient.ai".to_string()),
            ..Default::default()
        };
        let url = config.get_complete_url();
        assert_eq!(url, "https://agent.gradient.ai/api/v1/chat/completions");
    }

    #[test]
    fn test_build_request_body_with_gradient_params() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let config = GradientAIConfig {
                api_key: Some("test-key".to_string()),
                k: Some(5),
                provide_citations: Some(true),
                retrieval_method: Some(self::config::RetrievalMethod::SubQueries),
                instruction_override: Some("Custom instruction".to_string()),
                ..Default::default()
            };

            let provider = GradientAIProvider::new(config).await.unwrap();

            // Verify provider was created successfully
            assert_eq!(provider.name(), "gradient_ai");
        });
    }

    #[test]
    fn test_error_mapping() {
        use crate::core::providers::unified_provider::ProviderError;
        use crate::core::traits::error_mapper::trait_def::ErrorMapper;

        let mapper = error::GradientAIErrorMapper;

        // Test 401 error mapping
        let auth_error = mapper.map_http_error(401, "Unauthorized");
        match auth_error {
            ProviderError::Authentication { .. } => {}
            _ => panic!("Expected Authentication error"),
        }

        // Test 429 error mapping
        let rate_error = mapper.map_http_error(429, "Too many requests");
        match rate_error {
            ProviderError::RateLimit { .. } => {}
            _ => panic!("Expected RateLimit error"),
        }

        // Test 404 error mapping
        let not_found = mapper.map_http_error(404, "Not found");
        match not_found {
            ProviderError::ModelNotFound { .. } => {}
            _ => panic!("Expected ModelNotFound error"),
        }
    }

    #[test]
    fn test_error_retryability() {
        use crate::core::providers::unified_provider::ProviderError;

        // Rate limit errors should be retryable
        let rate_error = ProviderError::rate_limit("gradient_ai", Some(60));
        assert!(rate_error.is_retryable());
        assert!(rate_error.retry_delay().is_some());
        assert_eq!(rate_error.retry_delay(), Some(60));

        // Rate limit without retry_after is still retryable but no delay
        let rate_error_no_delay = ProviderError::rate_limit("gradient_ai", None);
        assert!(rate_error_no_delay.is_retryable());
        assert!(rate_error_no_delay.retry_delay().is_none());

        // Provider unavailable should be retryable
        let service_error = ProviderError::provider_unavailable("gradient_ai", "Service down");
        assert!(service_error.is_retryable());
        assert!(service_error.retry_delay().is_some());

        // Authentication errors should not be retryable
        let auth_error = ProviderError::authentication("gradient_ai", "Bad key");
        assert!(!auth_error.is_retryable());
        assert!(auth_error.retry_delay().is_none());

        // Not supported errors should not be retryable
        let unsupported_error = ProviderError::not_supported("gradient_ai", "feature");
        assert!(!unsupported_error.is_retryable());
    }

    #[tokio::test]
    async fn test_cost_calculation() {
        let provider = GradientAIProvider::with_api_key("test-key").await.unwrap();

        // Gradient AI cost calculation returns 0 (depends on agent configuration)
        let cost = provider.calculate_cost("any-model", 1000, 1000).await;
        assert!(cost.is_ok());
        assert_eq!(cost.unwrap(), 0.0);
    }

    #[test]
    fn test_retrieval_method_enum() {
        use self::config::RetrievalMethod;

        assert_eq!(
            serde_json::to_string(&RetrievalMethod::Rewrite).unwrap(),
            "\"rewrite\""
        );
        assert_eq!(
            serde_json::to_string(&RetrievalMethod::StepBack).unwrap(),
            "\"step_back\""
        );
        assert_eq!(
            serde_json::to_string(&RetrievalMethod::SubQueries).unwrap(),
            "\"sub_queries\""
        );
        assert_eq!(
            serde_json::to_string(&RetrievalMethod::None).unwrap(),
            "\"none\""
        );
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = GradientAIConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.gradient.ai".to_string()),
            timeout: 60,
            max_retries: 5,
            debug: true,
            k: Some(10),
            retrieval_method: Some(self::config::RetrievalMethod::SubQueries),
            provide_citations: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GradientAIConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.api_key, deserialized.api_key);
        assert_eq!(config.api_base, deserialized.api_base);
        assert_eq!(config.timeout, deserialized.timeout);
        assert_eq!(config.max_retries, deserialized.max_retries);
        assert_eq!(config.debug, deserialized.debug);
        assert_eq!(config.k, deserialized.k);
        assert_eq!(config.retrieval_method, deserialized.retrieval_method);
        assert_eq!(config.provide_citations, deserialized.provide_citations);
    }

    #[test]
    fn test_kb_filter_serialization() {
        let filter = config::KBFilter {
            key: "category".to_string(),
            value: serde_json::json!("tech"),
            operation: Some("eq".to_string()),
        };

        let json = serde_json::to_value(&filter).unwrap();
        assert_eq!(json["key"], "category");
        assert_eq!(json["value"], "tech");
        assert_eq!(json["operation"], "eq");

        // Test without operation
        let filter_no_op = config::KBFilter {
            key: "tag".to_string(),
            value: serde_json::json!(["rust", "ai"]),
            operation: None,
        };

        let json_no_op = serde_json::to_value(&filter_no_op).unwrap();
        assert_eq!(json_no_op["key"], "tag");
        assert!(json_no_op.get("operation").is_none());
    }

    #[test]
    fn test_config_with_all_gradient_params() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let kb_filters = vec![config::KBFilter {
                key: "category".to_string(),
                value: serde_json::json!("tech"),
                operation: Some("eq".to_string()),
            }];

            let config = GradientAIConfig {
                api_key: Some("test-key".to_string()),
                k: Some(5),
                kb_filters: Some(kb_filters),
                filter_kb_content_by_query_metadata: Some(true),
                instruction_override: Some("Custom instruction".to_string()),
                include_functions_info: Some(true),
                include_retrieval_info: Some(true),
                include_guardrails_info: Some(false),
                provide_citations: Some(true),
                retrieval_method: Some(self::config::RetrievalMethod::SubQueries),
                ..Default::default()
            };

            let provider = GradientAIProvider::new(config.clone()).await.unwrap();

            // Just verify provider was created successfully
            assert_eq!(provider.name(), "gradient_ai");
        });
    }
}
