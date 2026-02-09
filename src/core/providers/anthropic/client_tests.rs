    use super::*;
    use crate::core::providers::anthropic::config::AnthropicConfig;
    use crate::core::types::message::MessageContent;

    // ==================== Client Creation Tests ====================

    #[test]
    fn test_client_creation() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_creation_with_custom_config() {
        let mut config = AnthropicConfig::new_test("test-key");
        config.request_timeout = 120;
        config.connect_timeout = 30;
        let client = AnthropicClient::new(config);
        assert!(client.is_ok());
    }

    // ==================== Header Building Tests ====================

    #[test]
    fn test_header_building() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let headers = client.build_headers();

        // Anthropic uses x-api-key header instead of Authorization
        assert!(headers.contains_key("x-api-key"));
        assert!(headers.contains_key("anthropic-version"));
        assert!(headers.contains_key("content-type"));
        assert!(headers.contains_key("user-agent"));
    }

    #[test]
    fn test_header_content_type() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let headers = client.build_headers();

        let content_type = headers.get("content-type").unwrap();
        assert_eq!(content_type, "application/json");
    }

    #[test]
    fn test_header_user_agent() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let headers = client.build_headers();

        let user_agent = headers.get("user-agent").unwrap();
        assert_eq!(user_agent, "LiteLLM-Rust/1.0");
    }

    #[test]
    fn test_header_with_custom_headers() {
        let mut config = AnthropicConfig::new_test("test-key");
        config
            .custom_headers
            .insert("X-Custom-Header".to_string(), "custom-value".to_string());
        let client = AnthropicClient::new(config).unwrap();
        let headers = client.build_headers();

        assert!(headers.contains_key("x-custom-header"));
    }

    // ==================== Error Mapping Tests ====================

    #[test]
    fn test_map_http_error_400() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(400, "invalid request");

        // Should return an API error for 400
        let error_string = format!("{}", error);
        assert!(error_string.contains("400") || error_string.contains("request"));
    }

    #[test]
    fn test_map_http_error_401() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(401, "unauthorized");

        // Should return an authentication error
        let error_string = format!("{}", error);
        assert!(error_string.to_lowercase().contains("auth") || error_string.contains("key"));
    }

    #[test]
    fn test_map_http_error_403() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(403, "forbidden");

        // Should return an authentication error
        let error_string = format!("{}", error);
        assert!(
            error_string.to_lowercase().contains("forbidden")
                || error_string.to_lowercase().contains("permission")
                || error_string.to_lowercase().contains("auth")
        );
    }

    #[test]
    fn test_map_http_error_404() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(404, "not found");

        let error_string = format!("{}", error);
        assert!(error_string.contains("404") || error_string.contains("not found"));
    }

    #[test]
    fn test_map_http_error_429() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(429, "rate limited");

        // Should return a rate limit error
        let error_string = format!("{}", error);
        assert!(
            error_string.to_lowercase().contains("rate")
                || error_string.to_lowercase().contains("limit")
        );
    }

    #[test]
    fn test_map_http_error_500() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(500, "server error");

        let error_string = format!("{}", error);
        assert!(error_string.contains("500") || error_string.to_lowercase().contains("server"));
    }

    // ==================== Retry-After Extraction Tests ====================

    #[test]
    fn test_extract_retry_after_from_root() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let body = r#"{"retry_after": 60}"#;

        let retry = client.extract_retry_after(body);
        assert_eq!(retry, Some(60));
    }

    #[test]
    fn test_extract_retry_after_from_error() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let body = r#"{"error": {"retry_after": 30}}"#;

        let retry = client.extract_retry_after(body);
        assert_eq!(retry, Some(30));
    }

    #[test]
    fn test_extract_retry_after_missing() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let body = r#"{"message": "rate limited"}"#;

        let retry = client.extract_retry_after(body);
        assert!(retry.is_none());
    }

    #[test]
    fn test_extract_retry_after_invalid_json() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let body = "not json";

        let retry = client.extract_retry_after(body);
        assert!(retry.is_none());
    }

    // ==================== System Message Separation Tests ====================

    #[test]
    fn test_separate_system_messages_no_system() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let messages = vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
            thinking: None,
        }];

        let (system, user_msgs) = client.separate_system_messages(&messages).unwrap();
        assert!(system.is_none());
        assert_eq!(user_msgs.len(), 1);
    }

    #[test]
    fn test_separate_system_messages_with_system() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text(
                    "You are a helpful assistant.".to_string(),
                )),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
                thinking: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
                thinking: None,
            },
        ];

        let (system, user_msgs) = client.separate_system_messages(&messages).unwrap();
        assert_eq!(system, Some("You are a helpful assistant.".to_string()));
        assert_eq!(user_msgs.len(), 1);
    }

    #[test]
    fn test_separate_system_messages_multiple_system() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text("Rule 1".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
                thinking: None,
            },
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text("Rule 2".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
                thinking: None,
            },
        ];

        let (system, _) = client.separate_system_messages(&messages).unwrap();
        assert_eq!(system, Some("Rule 1\nRule 2".to_string()));
    }

    // ==================== Tool Choice Transformation Tests ====================

    #[test]
    fn test_transform_tool_choice_auto() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let tool_choice = crate::core::types::tools::ToolChoice::String("auto".to_string());
        let result = client.transform_tool_choice(&tool_choice).unwrap();

        assert_eq!(result["type"], "auto");
    }

    #[test]
    fn test_transform_tool_choice_none() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let tool_choice = crate::core::types::tools::ToolChoice::String("none".to_string());
        let result = client.transform_tool_choice(&tool_choice).unwrap();

        assert_eq!(result["type"], "none");
    }

    #[test]
    fn test_transform_tool_choice_required() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let tool_choice = crate::core::types::tools::ToolChoice::String("required".to_string());
        let result = client.transform_tool_choice(&tool_choice).unwrap();

        assert_eq!(result["type"], "any");
    }

    // ==================== Tool Transformation Tests ====================

    #[test]
    fn test_transform_tools() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let tools = vec![crate::core::types::tools::Tool {
            tool_type: crate::core::types::tools::ToolType::Function,
            function: crate::core::types::tools::FunctionDefinition {
                name: "get_weather".to_string(),
                description: Some("Get weather for a location".to_string()),
                parameters: Some(json!({"type": "object"})),
            },
        }];

        let result = client.transform_tools(&tools).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "get_weather");
        assert_eq!(result[0]["description"], "Get weather for a location");
    }

    #[test]
    fn test_transform_tools_empty() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let tools: Vec<crate::core::types::tools::Tool> = vec![];
        let result = client.transform_tools(&tools).unwrap();
        assert!(result.is_empty());
    }

    // ==================== Chat Response Transformation Tests ====================

    #[test]
    fn test_transform_chat_response_text() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [
                {"type": "text", "text": "Hello, world!"}
            ],
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 10,
                "output_tokens": 20
            }
        });

        let result = client.transform_chat_response(response).unwrap();
        assert_eq!(result.id, "msg_123");
        assert_eq!(result.model, "claude-3-opus-20240229");
        assert_eq!(result.choices.len(), 1);

        if let Some(MessageContent::Text(text)) = &result.choices[0].message.content {
            assert_eq!(text, "Hello, world!");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_transform_chat_response_usage() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "text", "text": "Hi"}],
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50
            }
        });

        let result = client.transform_chat_response(response).unwrap();
        let usage = result.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_transform_chat_response_tool_use() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [
                {
                    "type": "tool_use",
                    "id": "tool_1",
                    "name": "get_weather",
                    "input": {"location": "San Francisco"}
                }
            ],
            "stop_reason": "tool_use"
        });

        let result = client.transform_chat_response(response).unwrap();
        let tool_calls = result.choices[0].message.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "tool_1");
        assert_eq!(tool_calls[0].function.name, "get_weather");
    }

    #[test]
    fn test_transform_chat_response_finish_reasons() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        // end_turn -> Stop
        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "text", "text": "Hi"}],
            "stop_reason": "end_turn"
        });
        let result = client.transform_chat_response(response).unwrap();
        assert!(matches!(
            result.choices[0].finish_reason,
            Some(crate::core::types::responses::FinishReason::Stop)
        ));

        // max_tokens -> Length
        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "text", "text": "Hi"}],
            "stop_reason": "max_tokens"
        });
        let result = client.transform_chat_response(response).unwrap();
        assert!(matches!(
            result.choices[0].finish_reason,
            Some(crate::core::types::responses::FinishReason::Length)
        ));

        // tool_use -> ToolCalls
        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "text", "text": "Hi"}],
            "stop_reason": "tool_use"
        });
        let result = client.transform_chat_response(response).unwrap();
        assert!(matches!(
            result.choices[0].finish_reason,
            Some(crate::core::types::responses::FinishReason::ToolCalls)
        ));
    }
