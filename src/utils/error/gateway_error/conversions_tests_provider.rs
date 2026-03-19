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
        GatewayError::RateLimit {
            message,
            retry_after,
            rpm_limit,
            tpm_limit,
        } => {
            assert_eq!(message, "Too many requests");
            assert_eq!(retry_after, Some(60));
            assert_eq!(rpm_limit, None);
            assert_eq!(tpm_limit, None);
        }
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

    assert!(matches!(gateway_err, GatewayError::RateLimit { .. }));
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

    assert!(matches!(gateway_err, GatewayError::Unavailable(_)));
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
    assert!(matches!(gateway_err, GatewayError::Unavailable(_)));
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
        GatewayError::Validation { .. } => {}
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
        GatewayError::Validation { .. } => {}
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
