    use super::*;
    use crate::core::types::content::ContentPart;

    // ==================== HttpClientBuilder Tests ====================

    #[test]
    fn test_http_client_builder_default() {
        let builder = HttpClientBuilder::default();
        let result = builder.build();
        assert!(result.is_ok());

        let (_client, retry_config) = result.unwrap();
        // Client exists (built successfully)
        assert_eq!(retry_config.max_retries, 3);
    }

    #[test]
    fn test_http_client_builder_new() {
        let builder = HttpClientBuilder::new();
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_client_builder_with_timeout() {
        let builder = HttpClientBuilder::new().timeout(Duration::from_secs(120));
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_client_builder_with_max_retries() {
        let builder = HttpClientBuilder::new().max_retries(5);
        let result = builder.build();
        assert!(result.is_ok());

        let (_, retry_config) = result.unwrap();
        assert_eq!(retry_config.max_retries, 5);
    }

    #[test]
    fn test_http_client_builder_with_default_header() {
        let builder = HttpClientBuilder::new().default_header("X-Custom-Header", "test-value");
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_client_builder_with_multiple_headers() {
        let builder = HttpClientBuilder::new()
            .default_header("X-Header-1", "value1")
            .default_header("X-Header-2", "value2")
            .default_header("Authorization", "Bearer token");
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_client_builder_chained() {
        let builder = HttpClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .max_retries(2)
            .default_header("Content-Type", "application/json");
        let result = builder.build();
        assert!(result.is_ok());

        let (_, retry_config) = result.unwrap();
        assert_eq!(retry_config.max_retries, 2);
    }

    // ==================== RetryConfig Tests ====================

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.exponential_base, 2);
    }

    #[test]
    fn test_retry_config_clone() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            exponential_base: 3,
        };
        let cloned = config.clone();

        assert_eq!(cloned.max_retries, 5);
        assert_eq!(cloned.initial_delay, Duration::from_millis(500));
        assert_eq!(cloned.max_delay, Duration::from_secs(30));
        assert_eq!(cloned.exponential_base, 3);
    }

    #[test]
    fn test_retry_config_debug() {
        let config = RetryConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("RetryConfig"));
        assert!(debug_str.contains("max_retries"));
    }

    // ==================== RequestExecutor Tests ====================

    #[test]
    fn test_request_executor_new() {
        let (client, retry_config) = HttpClientBuilder::new().build().unwrap();
        let executor = RequestExecutor::new(client, retry_config);
        // Just verify it doesn't panic
        let _ = executor;
    }

    // ==================== MessageTransformer Tests ====================

    #[test]
    fn test_message_transformer_role_to_string_system() {
        assert_eq!(
            MessageTransformer::role_to_string(&MessageRole::System),
            "system"
        );
    }

    #[test]
    fn test_message_transformer_role_to_string_user() {
        assert_eq!(
            MessageTransformer::role_to_string(&MessageRole::User),
            "user"
        );
    }

    #[test]
    fn test_message_transformer_role_to_string_assistant() {
        assert_eq!(
            MessageTransformer::role_to_string(&MessageRole::Assistant),
            "assistant"
        );
    }

    #[test]
    fn test_message_transformer_role_to_string_tool() {
        assert_eq!(
            MessageTransformer::role_to_string(&MessageRole::Tool),
            "tool"
        );
    }

    #[test]
    fn test_message_transformer_role_to_string_function() {
        assert_eq!(
            MessageTransformer::role_to_string(&MessageRole::Function),
            "function"
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_system() {
        assert_eq!(
            MessageTransformer::string_to_role("system"),
            MessageRole::System
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_user() {
        assert_eq!(
            MessageTransformer::string_to_role("user"),
            MessageRole::User
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_assistant() {
        assert_eq!(
            MessageTransformer::string_to_role("assistant"),
            MessageRole::Assistant
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_tool() {
        assert_eq!(
            MessageTransformer::string_to_role("tool"),
            MessageRole::Tool
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_function() {
        assert_eq!(
            MessageTransformer::string_to_role("function"),
            MessageRole::Function
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_unknown() {
        assert_eq!(
            MessageTransformer::string_to_role("unknown"),
            MessageRole::User
        );
        assert_eq!(MessageTransformer::string_to_role(""), MessageRole::User);
    }

    #[test]
    fn test_message_transformer_content_to_value_text() {
        let content = Some(MessageContent::Text("Hello, world!".to_string()));
        let value = MessageTransformer::content_to_value(&content);
        assert_eq!(value, Value::String("Hello, world!".to_string()));
    }

    #[test]
    fn test_message_transformer_content_to_value_parts() {
        let content = Some(MessageContent::Parts(vec![
            ContentPart::Text {
                text: "Part 1".to_string(),
            },
            ContentPart::Text {
                text: "Part 2".to_string(),
            },
        ]));
        let value = MessageTransformer::content_to_value(&content);
        assert!(value.is_array());
    }

    #[test]
    fn test_message_transformer_content_to_value_none() {
        let content: Option<MessageContent> = None;
        let value = MessageTransformer::content_to_value(&content);
        assert!(value.is_null());
    }

    #[test]
    fn test_message_transformer_parse_finish_reason_stop() {
        assert_eq!(
            MessageTransformer::parse_finish_reason("stop"),
            Some(FinishReason::Stop)
        );
    }

    #[test]
    fn test_message_transformer_parse_finish_reason_length() {
        assert_eq!(
            MessageTransformer::parse_finish_reason("length"),
            Some(FinishReason::Length)
        );
        assert_eq!(
            MessageTransformer::parse_finish_reason("max_tokens"),
            Some(FinishReason::Length)
        );
    }

    #[test]
    fn test_message_transformer_parse_finish_reason_tool_calls() {
        assert_eq!(
            MessageTransformer::parse_finish_reason("tool_calls"),
            Some(FinishReason::ToolCalls)
        );
        assert_eq!(
            MessageTransformer::parse_finish_reason("function_call"),
            Some(FinishReason::ToolCalls)
        );
    }

    #[test]
    fn test_message_transformer_parse_finish_reason_content_filter() {
        assert_eq!(
            MessageTransformer::parse_finish_reason("content_filter"),
            Some(FinishReason::ContentFilter)
        );
    }

    #[test]
    fn test_message_transformer_parse_finish_reason_unknown() {
        assert_eq!(MessageTransformer::parse_finish_reason("unknown"), None);
        assert_eq!(MessageTransformer::parse_finish_reason(""), None);
    }

    // ==================== CommonProviderConfig Tests ====================

    #[test]
    fn test_common_provider_config_default() {
        let config = CommonProviderConfig::default();
        assert_eq!(config.api_key, "");
        assert_eq!(config.base_url, "");
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
        assert!(config.custom_headers.is_empty());
    }

    #[test]
    fn test_common_provider_config_with_values() {
        let config = CommonProviderConfig {
            api_key: "test-api-key".to_string(),
            base_url: "https://api.example.com".to_string(),
            timeout: 120,
            max_retries: 5,
            custom_headers: HashMap::from([("X-Custom".to_string(), "value".to_string())]),
        };

        assert_eq!(config.api_key, "test-api-key");
        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.custom_headers.len(), 1);
    }

    #[test]
    fn test_common_provider_config_serialization() {
        let config = CommonProviderConfig {
            api_key: "key123".to_string(),
            base_url: "https://api.test.com".to_string(),
            timeout: 30,
            max_retries: 2,
            custom_headers: HashMap::new(),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "key123");
        assert_eq!(json["base_url"], "https://api.test.com");
        assert_eq!(json["timeout"], 30);
        assert_eq!(json["max_retries"], 2);
    }

    #[test]
    fn test_common_provider_config_deserialization() {
        let json = r#"{
            "api_key": "my-key",
            "base_url": "https://example.com",
            "timeout": 45,
            "max_retries": 4
        }"#;

        let config: CommonProviderConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, "my-key");
        assert_eq!(config.base_url, "https://example.com");
        assert_eq!(config.timeout, 45);
        assert_eq!(config.max_retries, 4);
    }

    #[test]
    fn test_common_provider_config_clone() {
        let config = CommonProviderConfig {
            api_key: "key".to_string(),
            base_url: "url".to_string(),
            timeout: 10,
            max_retries: 1,
            custom_headers: HashMap::from([("h".to_string(), "v".to_string())]),
        };
        let cloned = config.clone();

        assert_eq!(cloned.api_key, config.api_key);
        assert_eq!(cloned.custom_headers, config.custom_headers);
    }

    #[test]
    fn test_common_provider_config_debug() {
        let config = CommonProviderConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("CommonProviderConfig"));
    }

    // ==================== RateLimiter Tests ====================

    #[test]
    fn test_rate_limiter_new() {
        let limiter = RateLimiter::new(10);
        assert_eq!(limiter.available_permits(), 10);
        assert_eq!(limiter._requests_per_second, 10);
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let limiter = RateLimiter::new(10);
        assert_eq!(limiter.available_permits(), 10);

        let _permit = limiter.acquire().await.unwrap();
        assert_eq!(limiter.available_permits(), 9);
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire_multiple() {
        let limiter = RateLimiter::new(5);

        let _permit1 = limiter.acquire().await.unwrap();
        let _permit2 = limiter.acquire().await.unwrap();
        let _permit3 = limiter.acquire().await.unwrap();

        assert_eq!(limiter.available_permits(), 2);
    }

    #[tokio::test]
    async fn test_rate_limiter_release() {
        let limiter = RateLimiter::new(10);

        {
            let _permit = limiter.acquire().await.unwrap();
            assert_eq!(limiter.available_permits(), 9);
        }
        // Permit is dropped, should be released
        assert_eq!(limiter.available_permits(), 10);
    }

    // ==================== ResponseValidator Tests ====================

    #[test]
    fn test_response_validator_valid_response() {
        let response = serde_json::json!({
            "id": "test-id",
            "choices": [{"message": {"content": "Hello"}}],
            "created": 1234567890,
            "model": "gpt-4"
        });

        let result = ResponseValidator::validate_chat_response(&response, "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_response_validator_missing_id() {
        let response = serde_json::json!({
            "choices": [{"message": {"content": "Hello"}}],
            "created": 1234567890,
            "model": "gpt-4"
        });

        let result = ResponseValidator::validate_chat_response(&response, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_response_validator_missing_choices() {
        let response = serde_json::json!({
            "id": "test-id",
            "created": 1234567890,
            "model": "gpt-4"
        });

        let result = ResponseValidator::validate_chat_response(&response, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_response_validator_empty_choices() {
        let response = serde_json::json!({
            "id": "test-id",
            "choices": [],
            "created": 1234567890,
            "model": "gpt-4"
        });

        let result = ResponseValidator::validate_chat_response(&response, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_response_validator_not_object() {
        let response = serde_json::json!([1, 2, 3]);

        let result = ResponseValidator::validate_chat_response(&response, "test");
        assert!(result.is_err());
    }

    // ==================== TokenCostCalculator Tests ====================

    #[test]
    fn test_token_cost_calculator_new() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        assert_eq!(calculator.input_cost_per_1k, 0.01);
        assert_eq!(calculator.output_cost_per_1k, 0.02);
    }

    #[test]
    fn test_token_cost_calculator_calculate_cost() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let usage = Usage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        };
        let cost = calculator.calculate_cost(&usage);
        // (1000/1000 * 0.01) + (500/1000 * 0.02) = 0.01 + 0.01 = 0.02
        assert!((cost - 0.02).abs() < 0.0001);
    }

    #[test]
    fn test_token_cost_calculator_zero_tokens() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let usage = Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        };
        let cost = calculator.calculate_cost(&usage);
        assert!((cost - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_token_cost_calculator_only_input() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let usage = Usage {
            prompt_tokens: 1000,
            completion_tokens: 0,
            total_tokens: 1000,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        };
        let cost = calculator.calculate_cost(&usage);
        assert!((cost - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_token_cost_calculator_only_output() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let usage = Usage {
            prompt_tokens: 0,
            completion_tokens: 1000,
            total_tokens: 1000,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        };
        let cost = calculator.calculate_cost(&usage);
        assert!((cost - 0.02).abs() < 0.0001);
    }

    #[test]
    fn test_token_cost_calculator_large_tokens() {
        let calculator = TokenCostCalculator::new(0.003, 0.015);
        let usage = Usage {
            prompt_tokens: 100000,
            completion_tokens: 50000,
            total_tokens: 150000,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        };
        let cost = calculator.calculate_cost(&usage);
        // (100 * 0.003) + (50 * 0.015) = 0.3 + 0.75 = 1.05
        assert!((cost - 1.05).abs() < 0.001);
    }

    #[test]
    fn test_token_cost_calculator_clone() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let cloned = calculator.clone();

        assert_eq!(cloned.input_cost_per_1k, calculator.input_cost_per_1k);
        assert_eq!(cloned.output_cost_per_1k, calculator.output_cost_per_1k);
    }

    #[test]
    fn test_token_cost_calculator_debug() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let debug_str = format!("{:?}", calculator);
        assert!(debug_str.contains("TokenCostCalculator"));
    }

    // ==================== Test Utilities Tests ====================

    #[test]
    fn test_mock_message() {
        let message = test_utils::mock_message(MessageRole::User, "Hello");

        assert_eq!(message.role, MessageRole::User);
        match &message.content {
            Some(MessageContent::Text(text)) => assert_eq!(text, "Hello"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_mock_usage() {
        let usage = test_utils::mock_usage(100, 50);

        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }
