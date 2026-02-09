    use super::*;

    #[test]
    fn test_authentication_error_conversion() {
        let provider_err = ProviderError::Authentication {
            provider: "openai",
            message: "Invalid API key".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();

        match gateway_err {
            GatewayError::Auth(msg) => assert_eq!(msg, "Invalid API key"),
            _ => panic!("Expected Auth error"),
        }
    }

    #[test]
    fn test_rate_limit_error_conversion() {
        let provider_err = ProviderError::RateLimit {
            provider: "anthropic",
            message: "Too many requests".to_string(),
            retry_after: Some(60),
            rpm_limit: None,
            tpm_limit: None,
            current_usage: None,
        };
        let gateway_err: GatewayError = provider_err.into();

        match gateway_err {
            GatewayError::RateLimit(msg) => assert_eq!(msg, "Too many requests"),
            _ => panic!("Expected RateLimit error"),
        }
    }

    #[test]
    fn test_model_not_found_conversion() {
        let provider_err = ProviderError::ModelNotFound {
            provider: "openai",
            model: "gpt-5".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();

        match gateway_err {
            GatewayError::NotFound(msg) => assert!(msg.contains("gpt-5")),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_api_error_401_becomes_auth() {
        let provider_err = ProviderError::ApiError {
            provider: "openai",
            status: 401,
            message: "Unauthorized".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();

        assert!(matches!(gateway_err, GatewayError::Auth(_)));
    }

    #[test]
    fn test_api_error_404_becomes_not_found() {
        let provider_err = ProviderError::ApiError {
            provider: "openai",
            status: 404,
            message: "Not found".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();

        assert!(matches!(gateway_err, GatewayError::NotFound(_)));
    }

    #[test]
    fn test_api_error_429_becomes_rate_limit() {
        let provider_err = ProviderError::ApiError {
            provider: "openai",
            status: 429,
            message: "Rate limited".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();

        assert!(matches!(gateway_err, GatewayError::RateLimit(_)));
    }

    #[test]
    fn test_api_error_400_becomes_bad_request() {
        let provider_err = ProviderError::ApiError {
            provider: "openai",
            status: 400,
            message: "Bad request".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();

        assert!(matches!(gateway_err, GatewayError::BadRequest(_)));
    }

    #[test]
    fn test_api_error_500_becomes_internal() {
        let provider_err = ProviderError::ApiError {
            provider: "openai",
            status: 500,
            message: "Server error".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();

        assert!(matches!(gateway_err, GatewayError::Internal(_)));
    }

    #[test]
    fn test_context_length_exceeded_conversion() {
        let provider_err = ProviderError::ContextLengthExceeded {
            provider: "anthropic",
            max: 100000,
            actual: 150000,
        };
        let gateway_err: GatewayError = provider_err.into();

        match gateway_err {
            GatewayError::BadRequest(msg) => {
                assert!(msg.contains("100000"));
                assert!(msg.contains("150000"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_configuration_error_conversion() {
        let provider_err = ProviderError::Configuration {
            provider: "azure",
            message: "Missing API key".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();

        assert!(matches!(gateway_err, GatewayError::Config(_)));
    }

    #[test]
    fn test_timeout_error_conversion() {
        let provider_err = ProviderError::Timeout {
            provider: "openai",
            message: "Request timed out".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();

        assert!(matches!(gateway_err, GatewayError::Timeout(_)));
    }

    #[test]
    fn test_not_implemented_conversion() {
        let provider_err = ProviderError::NotImplemented {
            provider: "mistral",
            feature: "image generation".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();

        match gateway_err {
            GatewayError::NotImplemented(msg) => {
                assert!(msg.contains("image generation"));
                assert!(msg.contains("mistral"));
            }
            _ => panic!("Expected NotImplemented error"),
        }
    }

    #[test]
    fn test_routing_error_conversion() {
        let provider_err = ProviderError::RoutingError {
            provider: "router",
            attempted_providers: vec!["openai".to_string(), "anthropic".to_string()],
            message: "All providers failed".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();

        assert!(matches!(gateway_err, GatewayError::ProviderUnavailable(_)));
    }

    #[test]
    fn test_streaming_error_conversion() {
        let provider_err = ProviderError::Streaming {
            provider: "openai",
            stream_type: "chat".to_string(),
            message: "Stream interrupted".to_string(),
            position: None,
            last_chunk: None,
        };
        let gateway_err: GatewayError = provider_err.into();

        match gateway_err {
            GatewayError::Internal(msg) => {
                assert!(msg.contains("openai"));
                assert!(msg.contains("chat"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_invalid_request_conversion() {
        let provider_err = ProviderError::InvalidRequest {
            provider: "openai",
            message: "Invalid parameters".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        assert!(matches!(gateway_err, GatewayError::BadRequest(_)));
    }

    #[test]
    fn test_network_error_conversion() {
        let provider_err = ProviderError::Network {
            provider: "anthropic",
            message: "Connection refused".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        // Network errors become Network variant
        match gateway_err {
            GatewayError::Network { .. } => {}
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_provider_unavailable_conversion() {
        let provider_err = ProviderError::ProviderUnavailable {
            provider: "openai",
            message: "Service down".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        assert!(matches!(gateway_err, GatewayError::ProviderUnavailable(_)));
    }

    #[test]
    fn test_not_supported_conversion() {
        let provider_err = ProviderError::NotSupported {
            provider: "groq",
            feature: "embeddings".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        match gateway_err {
            GatewayError::NotImplemented(msg) => {
                assert!(msg.contains("embeddings"));
                assert!(msg.contains("not supported"));
            }
            _ => panic!("Expected NotImplemented error"),
        }
    }

    #[test]
    fn test_serialization_error_conversion() {
        let provider_err = ProviderError::Serialization {
            provider: "openai",
            message: "Invalid JSON".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        match gateway_err {
            GatewayError::Parsing { .. } => {}
            _ => panic!("Expected Parsing error"),
        }
    }

    #[test]
    fn test_quota_exceeded_conversion() {
        let provider_err = ProviderError::QuotaExceeded {
            provider: "anthropic",
            message: "Monthly limit reached".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        match gateway_err {
            GatewayError::BadRequest(msg) => assert!(msg.contains("Quota exceeded")),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_other_error_conversion() {
        let provider_err = ProviderError::Other {
            provider: "unknown",
            message: "Unknown error".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        assert!(matches!(gateway_err, GatewayError::Internal(_)));
    }

    #[test]
    fn test_content_filtered_conversion() {
        let provider_err = ProviderError::ContentFiltered {
            provider: "openai",
            reason: "Violence detected".to_string(),
            policy_violations: None,
            potentially_retryable: Some(false),
        };
        let gateway_err: GatewayError = provider_err.into();
        match gateway_err {
            GatewayError::BadRequest(msg) => {
                assert!(msg.contains("Content filtered"));
                assert!(msg.contains("Violence detected"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_token_limit_exceeded_conversion() {
        let provider_err = ProviderError::TokenLimitExceeded {
            provider: "anthropic",
            message: "Max tokens exceeded".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        match gateway_err {
            GatewayError::BadRequest(msg) => assert!(msg.contains("Token limit exceeded")),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_feature_disabled_conversion() {
        let provider_err = ProviderError::FeatureDisabled {
            provider: "azure",
            feature: "streaming".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        match gateway_err {
            GatewayError::NotImplemented(msg) => {
                assert!(msg.contains("streaming"));
                assert!(msg.contains("disabled"));
            }
            _ => panic!("Expected NotImplemented error"),
        }
    }

    #[test]
    fn test_deployment_error_conversion() {
        let provider_err = ProviderError::DeploymentError {
            provider: "azure",
            deployment: "gpt4-deployment".to_string(),
            message: "Deployment not found".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        match gateway_err {
            GatewayError::NotFound(msg) => {
                assert!(msg.contains("gpt4-deployment"));
                assert!(msg.contains("Azure deployment"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_response_parsing_conversion() {
        let provider_err = ProviderError::ResponseParsing {
            provider: "openai",
            message: "Unexpected format".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        match gateway_err {
            GatewayError::Parsing { .. } => {}
            _ => panic!("Expected Parsing error"),
        }
    }

    #[test]
    fn test_transformation_error_conversion() {
        let provider_err = ProviderError::TransformationError {
            provider: "anthropic",
            from_format: "anthropic".to_string(),
            to_format: "openai".to_string(),
            message: "Format mismatch".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        match gateway_err {
            GatewayError::Internal(msg) => {
                assert!(msg.contains("Transformation error"));
                assert!(msg.contains("anthropic"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_cancelled_error_conversion() {
        let provider_err = ProviderError::Cancelled {
            provider: "openai",
            operation_type: "chat_completion".to_string(),
            cancellation_reason: Some("User cancelled".to_string()),
        };
        let gateway_err: GatewayError = provider_err.into();
        match gateway_err {
            GatewayError::BadRequest(msg) => {
                assert!(msg.contains("cancelled"));
                assert!(msg.contains("chat_completion"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    // ==================== A2A Error Conversion Tests ====================

    #[test]
    fn test_a2a_agent_not_found_conversion() {
        let a2a_err = A2AError::AgentNotFound {
            agent_name: "my-agent".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::NotFound(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("my-agent"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_a2a_agent_already_exists_conversion() {
        let a2a_err = A2AError::AgentAlreadyExists {
            agent_name: "existing-agent".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::Conflict(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("existing-agent"));
            }
            _ => panic!("Expected Conflict error"),
        }
    }

    #[test]
    fn test_a2a_connection_error_conversion() {
        let a2a_err = A2AError::ConnectionError {
            agent_name: "remote-agent".to_string(),
            message: "Connection refused".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::Network(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("remote-agent"));
                assert!(msg.contains("Connection refused"));
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_a2a_authentication_error_conversion() {
        let a2a_err = A2AError::AuthenticationError {
            agent_name: "secure-agent".to_string(),
            message: "Invalid token".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::Auth(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("secure-agent"));
            }
            _ => panic!("Expected Auth error"),
        }
    }

    #[test]
    fn test_a2a_task_not_found_conversion() {
        let a2a_err = A2AError::TaskNotFound {
            agent_name: "agent".to_string(),
            task_id: "task-456".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::NotFound(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("task-456"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_a2a_task_failed_conversion() {
        let a2a_err = A2AError::TaskFailed {
            agent_name: "agent".to_string(),
            task_id: "task-123".to_string(),
            message: "Something went wrong".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::Internal(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("task-123"));
                assert!(msg.contains("Something went wrong"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_a2a_protocol_error_conversion() {
        let a2a_err = A2AError::ProtocolError {
            message: "Invalid JSON-RPC".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::BadRequest(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("protocol error"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_a2a_timeout_conversion() {
        let a2a_err = A2AError::Timeout {
            agent_name: "slow-agent".to_string(),
            timeout_ms: 30000,
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::Timeout(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("slow-agent"));
                assert!(msg.contains("30000"));
            }
            _ => panic!("Expected Timeout error"),
        }
    }

    #[test]
    fn test_a2a_configuration_error_conversion() {
        let a2a_err = A2AError::ConfigurationError {
            message: "Missing endpoint".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        assert!(matches!(gateway_err, GatewayError::Config(_)));
    }

    #[test]
    fn test_a2a_serialization_error_conversion() {
        let a2a_err = A2AError::SerializationError {
            message: "Invalid UTF-8".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::Parsing(msg) => assert!(msg.contains("A2A")),
            _ => panic!("Expected Parsing error"),
        }
    }

    #[test]
    fn test_a2a_unsupported_provider_conversion() {
        let a2a_err = A2AError::UnsupportedProvider {
            provider: "unknown-provider".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::NotImplemented(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("unknown-provider"));
            }
            _ => panic!("Expected NotImplemented error"),
        }
    }

    #[test]
    fn test_a2a_rate_limit_with_retry_conversion() {
        let a2a_err = A2AError::RateLimitExceeded {
            agent_name: "agent".to_string(),
            retry_after_ms: Some(5000),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::RateLimit(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("5000ms"));
            }
            _ => panic!("Expected RateLimit error"),
        }
    }

    #[test]
    fn test_a2a_rate_limit_without_retry_conversion() {
        let a2a_err = A2AError::RateLimitExceeded {
            agent_name: "agent".to_string(),
            retry_after_ms: None,
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::RateLimit(msg) => {
                assert!(msg.contains("A2A"));
                assert!(!msg.contains("retry after"));
            }
            _ => panic!("Expected RateLimit error"),
        }
    }

    #[test]
    fn test_a2a_agent_busy_conversion() {
        let a2a_err = A2AError::AgentBusy {
            agent_name: "busy-agent".to_string(),
            message: "Processing another request".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::ProviderUnavailable(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("busy-agent"));
            }
            _ => panic!("Expected ProviderUnavailable error"),
        }
    }

    #[test]
    fn test_a2a_content_blocked_conversion() {
        let a2a_err = A2AError::ContentBlocked {
            agent_name: "safe-agent".to_string(),
            reason: "Harmful content".to_string(),
        };
        let gateway_err: GatewayError = a2a_err.into();
        match gateway_err {
            GatewayError::BadRequest(msg) => {
                assert!(msg.contains("A2A"));
                assert!(msg.contains("blocked"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    // ==================== MCP Error Conversion Tests ====================

    #[test]
    fn test_mcp_server_not_found_conversion() {
        let mcp_err = McpError::ServerNotFound {
            server_name: "github".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::NotFound(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("github"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_mcp_tool_not_found_conversion() {
        let mcp_err = McpError::ToolNotFound {
            server_name: "github".to_string(),
            tool_name: "get_repo".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::NotFound(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("get_repo"));
                assert!(msg.contains("github"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_mcp_connection_error_conversion() {
        let mcp_err = McpError::ConnectionError {
            server_name: "github".to_string(),
            message: "Connection refused".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::Network(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("github"));
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_mcp_transport_error_conversion() {
        let mcp_err = McpError::TransportError {
            transport: "http".to_string(),
            message: "Connection reset".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::Network(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("http"));
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_mcp_authentication_error_conversion() {
        let mcp_err = McpError::AuthenticationError {
            server_name: "github".to_string(),
            message: "Invalid token".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::Auth(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("github"));
            }
            _ => panic!("Expected Auth error"),
        }
    }

    #[test]
    fn test_mcp_authorization_error_with_tool_conversion() {
        let mcp_err = McpError::AuthorizationError {
            server_name: "github".to_string(),
            tool_name: Some("delete_repo".to_string()),
            message: "Admin required".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::Authorization(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("delete_repo"));
            }
            _ => panic!("Expected Authorization error"),
        }
    }

    #[test]
    fn test_mcp_authorization_error_without_tool_conversion() {
        let mcp_err = McpError::AuthorizationError {
            server_name: "github".to_string(),
            tool_name: None,
            message: "Access denied".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::Authorization(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("github"));
            }
            _ => panic!("Expected Authorization error"),
        }
    }

    #[test]
    fn test_mcp_protocol_error_conversion() {
        let mcp_err = McpError::ProtocolError {
            message: "Invalid JSON-RPC".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::BadRequest(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("protocol error"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_mcp_tool_execution_error_conversion() {
        let mcp_err = McpError::ToolExecutionError {
            server_name: "github".to_string(),
            tool_name: "create_issue".to_string(),
            code: -32000,
            message: "Repository not found".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::Internal(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("create_issue"));
                assert!(msg.contains("-32000"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_mcp_timeout_conversion() {
        let mcp_err = McpError::Timeout {
            server_name: "slow-server".to_string(),
            timeout_ms: 30000,
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::Timeout(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("slow-server"));
                assert!(msg.contains("30000"));
            }
            _ => panic!("Expected Timeout error"),
        }
    }

    #[test]
    fn test_mcp_configuration_error_conversion() {
        let mcp_err = McpError::ConfigurationError {
            message: "Missing URL".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        assert!(matches!(gateway_err, GatewayError::Config(_)));
    }

    #[test]
    fn test_mcp_serialization_error_conversion() {
        let mcp_err = McpError::SerializationError {
            message: "Invalid JSON".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::Parsing(msg) => assert!(msg.contains("MCP")),
            _ => panic!("Expected Parsing error"),
        }
    }

    #[test]
    fn test_mcp_server_already_exists_conversion() {
        let mcp_err = McpError::ServerAlreadyExists {
            server_name: "github".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::Conflict(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("github"));
            }
            _ => panic!("Expected Conflict error"),
        }
    }

    #[test]
    fn test_mcp_invalid_url_conversion() {
        let mcp_err = McpError::InvalidUrl {
            url: "not-a-url".to_string(),
            message: "Invalid format".to_string(),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::BadRequest(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("not-a-url"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_mcp_rate_limit_with_retry_conversion() {
        let mcp_err = McpError::RateLimitExceeded {
            server_name: "github".to_string(),
            retry_after_ms: Some(5000),
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::RateLimit(msg) => {
                assert!(msg.contains("MCP"));
                assert!(msg.contains("5000ms"));
            }
            _ => panic!("Expected RateLimit error"),
        }
    }

    #[test]
    fn test_mcp_rate_limit_without_retry_conversion() {
        let mcp_err = McpError::RateLimitExceeded {
            server_name: "github".to_string(),
            retry_after_ms: None,
        };
        let gateway_err: GatewayError = mcp_err.into();
        match gateway_err {
            GatewayError::RateLimit(msg) => {
                assert!(msg.contains("MCP"));
                assert!(!msg.contains("retry after"));
            }
            _ => panic!("Expected RateLimit error"),
        }
    }
