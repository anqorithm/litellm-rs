use super::*;

#[test]
fn test_map_http_status_400_bad_request() {
    let error = ErrorUtils::map_http_status_to_error(400, Some("Bad request".to_string()));
    match error {
        ProviderError::InvalidRequest { provider, message } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "Bad request");
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_map_http_status_400_no_message() {
    let error = ErrorUtils::map_http_status_to_error(400, None);
    match error {
        ProviderError::InvalidRequest { message, .. } => {
            assert_eq!(message, "HTTP error 400");
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_map_http_status_401_unauthorized() {
    let error = ErrorUtils::map_http_status_to_error(401, Some("Unauthorized".to_string()));
    match error {
        ProviderError::Authentication { provider, message } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "Unauthorized");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[test]
fn test_map_http_status_403_forbidden() {
    let error = ErrorUtils::map_http_status_to_error(403, Some("Access denied".to_string()));
    match error {
        ProviderError::Authentication { provider, message } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "Permission denied: Access denied");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[test]
fn test_map_http_status_404_not_found() {
    let error = ErrorUtils::map_http_status_to_error(404, Some("Model not found".to_string()));
    match error {
        ProviderError::ModelNotFound { provider, model } => {
            assert_eq!(provider, "unknown");
            assert_eq!(model, "Model not found");
        }
        _ => panic!("Expected ModelNotFound error"),
    }
}

#[test]
fn test_map_http_status_429_rate_limit() {
    let error = ErrorUtils::map_http_status_to_error(429, Some("Too many requests".to_string()));
    match error {
        ProviderError::RateLimit {
            provider,
            message,
            retry_after,
            ..
        } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "Too many requests");
            assert_eq!(retry_after, Some(60));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_map_http_status_408_timeout() {
    let error = ErrorUtils::map_http_status_to_error(408, Some("Request timeout".to_string()));
    match error {
        ProviderError::Timeout { provider, message } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "Request timeout");
        }
        _ => panic!("Expected Timeout error"),
    }
}

#[test]
fn test_map_http_status_504_gateway_timeout() {
    let error = ErrorUtils::map_http_status_to_error(504, Some("Gateway timeout".to_string()));
    match error {
        ProviderError::Timeout { provider, message } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "Gateway timeout");
        }
        _ => panic!("Expected Timeout error"),
    }
}

#[test]
fn test_map_http_status_500_internal_server_error() {
    let error =
        ErrorUtils::map_http_status_to_error(500, Some("Internal server error".to_string()));
    match error {
        ProviderError::ProviderUnavailable { provider, message } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "Internal server error");
        }
        _ => panic!("Expected ProviderUnavailable error"),
    }
}

#[test]
fn test_map_http_status_502_bad_gateway() {
    let error = ErrorUtils::map_http_status_to_error(502, Some("Bad gateway".to_string()));
    match error {
        ProviderError::ProviderUnavailable { provider, message } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "Bad gateway");
        }
        _ => panic!("Expected ProviderUnavailable error"),
    }
}

#[test]
fn test_map_http_status_503_service_unavailable() {
    let error = ErrorUtils::map_http_status_to_error(503, Some("Service unavailable".to_string()));
    match error {
        ProviderError::ProviderUnavailable { provider, message } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "Service unavailable");
        }
        _ => panic!("Expected ProviderUnavailable error"),
    }
}

#[test]
fn test_map_http_status_unknown() {
    let error = ErrorUtils::map_http_status_to_error(418, Some("I'm a teapot".to_string()));
    match error {
        ProviderError::Other { provider, message } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "I'm a teapot");
        }
        _ => panic!("Expected Other error"),
    }
}

#[test]
fn test_extract_retry_after_seconds() {
    let mut headers = HashMap::new();
    headers.insert("retry-after".to_string(), "120".to_string());

    let duration = ErrorUtils::extract_retry_after(&headers);
    assert_eq!(duration, Some(Duration::from_secs(120)));
}

#[test]
fn test_extract_retry_after_invalid_format() {
    let mut headers = HashMap::new();
    headers.insert("retry-after".to_string(), "invalid".to_string());

    let duration = ErrorUtils::extract_retry_after(&headers);
    assert_eq!(duration, None);
}

#[test]
fn test_extract_retry_after_rate_limit_reset_future() {
    let mut headers = HashMap::new();
    let future_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + 300; // 5 minutes in the future
    headers.insert(
        "x-ratelimit-reset".to_string(),
        future_timestamp.to_string(),
    );

    let duration = ErrorUtils::extract_retry_after(&headers);
    assert!(duration.is_some());
    let duration = duration.unwrap();
    // Should be approximately 300 seconds (allow some variance for test execution time)
    assert!(duration.as_secs() >= 299 && duration.as_secs() <= 300);
}

#[test]
fn test_extract_retry_after_rate_limit_reset_past() {
    let mut headers = HashMap::new();
    let past_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        - 100; // 100 seconds in the past
    headers.insert("x-ratelimit-reset".to_string(), past_timestamp.to_string());

    let duration = ErrorUtils::extract_retry_after(&headers);
    assert_eq!(duration, None);
}

#[test]
fn test_extract_retry_after_no_headers() {
    let headers = HashMap::new();
    let duration = ErrorUtils::extract_retry_after(&headers);
    assert_eq!(duration, None);
}

#[test]
fn test_extract_retry_after_priority() {
    let mut headers = HashMap::new();
    headers.insert("retry-after".to_string(), "60".to_string());
    let future_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + 120;
    headers.insert(
        "x-ratelimit-reset".to_string(),
        future_timestamp.to_string(),
    );

    // retry-after should take priority
    let duration = ErrorUtils::extract_retry_after(&headers);
    assert_eq!(duration, Some(Duration::from_secs(60)));
}

#[test]
fn test_parse_openai_error_invalid_request() {
    let response =
        r#"{"error": {"type": "invalid_request_error", "message": "Invalid model specified"}}"#;
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::InvalidRequest { provider, message } => {
            assert_eq!(provider, "openai");
            assert_eq!(message, "Invalid model specified");
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_parse_openai_error_authentication() {
    let response = r#"{"error": {"type": "authentication_error", "message": "Invalid API key"}}"#;
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::Authentication { provider, message } => {
            assert_eq!(provider, "openai");
            assert_eq!(message, "Invalid API key");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[test]
fn test_parse_openai_error_permission() {
    let response =
        r#"{"error": {"type": "permission_error", "message": "Access denied to this model"}}"#;
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::Authentication { provider, message } => {
            assert_eq!(provider, "openai");
            assert_eq!(message, "Access denied to this model");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[test]
fn test_parse_openai_error_rate_limit() {
    let response = r#"{"error": {"type": "rate_limit_error", "message": "Rate limit exceeded"}}"#;
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::RateLimit {
            provider,
            message,
            retry_after,
            ..
        } => {
            assert_eq!(provider, "openai");
            assert_eq!(message, "Rate limit exceeded");
            assert_eq!(retry_after, Some(60));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_parse_openai_error_model_not_found() {
    let response = r#"{"error": {"type": "model_not_found_error", "message": "gpt-unknown"}}"#;
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::ModelNotFound { provider, model } => {
            assert_eq!(provider, "openai");
            assert_eq!(model, "gpt-unknown");
        }
        _ => panic!("Expected ModelNotFound error"),
    }
}

#[test]
fn test_parse_openai_error_context_length() {
    let response = r#"{"error": {"type": "context_length_exceeded", "message": "Maximum context length is 4096 tokens"}}"#;
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::InvalidRequest { provider, message } => {
            assert_eq!(provider, "openai");
            assert!(message.contains("Context length exceeded"));
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_parse_openai_error_timeout() {
    let response = r#"{"error": {"type": "timeout_error", "message": "Request timed out"}}"#;
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::Timeout { provider, message } => {
            assert_eq!(provider, "openai");
            assert_eq!(message, "Request timed out");
        }
        _ => panic!("Expected Timeout error"),
    }
}

#[test]
fn test_parse_openai_error_server_error() {
    let response = r#"{"error": {"type": "server_error", "message": "Internal server error"}}"#;
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::ProviderUnavailable { provider, message } => {
            assert_eq!(provider, "openai");
            assert_eq!(message, "Internal server error");
        }
        _ => panic!("Expected ProviderUnavailable error"),
    }
}

#[test]
fn test_parse_openai_error_unknown_type() {
    let response =
        r#"{"error": {"type": "unknown_error_type", "message": "Something went wrong"}}"#;
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::Other { provider, message } => {
            assert_eq!(provider, "openai");
            assert_eq!(message, "Something went wrong");
        }
        _ => panic!("Expected Other error"),
    }
}

#[test]
fn test_parse_openai_error_no_message() {
    let response = r#"{"error": {"type": "invalid_request_error"}}"#;
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::InvalidRequest { message, .. } => {
            assert_eq!(message, "Unknown error");
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_parse_openai_error_invalid_json() {
    let response = "not valid json";
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::Other { provider, message } => {
            assert_eq!(provider, "openai");
            assert_eq!(message, "not valid json");
        }
        _ => panic!("Expected Other error"),
    }
}

#[test]
fn test_parse_openai_error_no_error_field() {
    let response = r#"{"status": "failed", "message": "Something went wrong"}"#;
    let error = ErrorUtils::parse_openai_error(response);
    match error {
        ProviderError::Other { provider, message } => {
            assert_eq!(provider, "openai");
            assert!(message.contains("status"));
        }
        _ => panic!("Expected Other error"),
    }
}

#[test]
fn test_parse_anthropic_error_invalid_request() {
    let response =
        r#"{"error": {"type": "invalid_request_error", "message": "Invalid parameter"}}"#;
    let error = ErrorUtils::parse_anthropic_error(response);
    match error {
        ProviderError::InvalidRequest { provider, message } => {
            assert_eq!(provider, "anthropic");
            assert_eq!(message, "Invalid parameter");
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_parse_anthropic_error_authentication() {
    let response = r#"{"error": {"type": "authentication_error", "message": "Invalid API key"}}"#;
    let error = ErrorUtils::parse_anthropic_error(response);
    match error {
        ProviderError::Authentication { provider, message } => {
            assert_eq!(provider, "anthropic");
            assert_eq!(message, "Invalid API key");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[test]
fn test_parse_anthropic_error_permission() {
    let response = r#"{"error": {"type": "permission_error", "message": "Access not allowed"}}"#;
    let error = ErrorUtils::parse_anthropic_error(response);
    match error {
        ProviderError::Authentication { provider, message } => {
            assert_eq!(provider, "anthropic");
            assert_eq!(message, "Access not allowed");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[test]
fn test_parse_anthropic_error_rate_limit() {
    let response = r#"{"error": {"type": "rate_limit_error", "message": "Too many requests"}}"#;
    let error = ErrorUtils::parse_anthropic_error(response);
    match error {
        ProviderError::RateLimit {
            provider,
            message,
            retry_after,
            ..
        } => {
            assert_eq!(provider, "anthropic");
            assert_eq!(message, "Too many requests");
            assert_eq!(retry_after, Some(60));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_parse_anthropic_error_not_found() {
    let response = r#"{"error": {"type": "not_found_error", "message": "claude-unknown"}}"#;
    let error = ErrorUtils::parse_anthropic_error(response);
    match error {
        ProviderError::ModelNotFound { provider, model } => {
            assert_eq!(provider, "anthropic");
            assert_eq!(model, "claude-unknown");
        }
        _ => panic!("Expected ModelNotFound error"),
    }
}

#[test]
fn test_parse_anthropic_error_overloaded() {
    let response = r#"{"error": {"type": "overloaded_error", "message": "Service is overloaded"}}"#;
    let error = ErrorUtils::parse_anthropic_error(response);
    match error {
        ProviderError::ProviderUnavailable { provider, message } => {
            assert_eq!(provider, "anthropic");
            assert_eq!(message, "Service is overloaded");
        }
        _ => panic!("Expected ProviderUnavailable error"),
    }
}

#[test]
fn test_parse_anthropic_error_unknown_type() {
    let response = r#"{"error": {"type": "unknown_error", "message": "Something went wrong"}}"#;
    let error = ErrorUtils::parse_anthropic_error(response);
    match error {
        ProviderError::Other { provider, message } => {
            assert_eq!(provider, "anthropic");
            assert_eq!(message, "Something went wrong");
        }
        _ => panic!("Expected Other error"),
    }
}

#[test]
fn test_parse_anthropic_error_invalid_json() {
    let response = "invalid json";
    let error = ErrorUtils::parse_anthropic_error(response);
    match error {
        ProviderError::Other { provider, message } => {
            assert_eq!(provider, "anthropic");
            assert_eq!(message, "invalid json");
        }
        _ => panic!("Expected Other error"),
    }
}

#[test]
fn test_parse_google_error_invalid_argument() {
    let response = r#"{"error": {"status": "INVALID_ARGUMENT", "message": "Invalid input"}}"#;
    let error = ErrorUtils::parse_google_error(response);
    match error {
        ProviderError::InvalidRequest { provider, message } => {
            assert_eq!(provider, "google");
            assert_eq!(message, "Invalid input");
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_parse_google_error_unauthenticated() {
    let response = r#"{"error": {"status": "UNAUTHENTICATED", "message": "Invalid credentials"}}"#;
    let error = ErrorUtils::parse_google_error(response);
    match error {
        ProviderError::Authentication { provider, message } => {
            assert_eq!(provider, "google");
            assert_eq!(message, "Invalid credentials");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[test]
fn test_parse_google_error_permission_denied() {
    let response = r#"{"error": {"status": "PERMISSION_DENIED", "message": "Access denied"}}"#;
    let error = ErrorUtils::parse_google_error(response);
    match error {
        ProviderError::Authentication { provider, message } => {
            assert_eq!(provider, "google");
            assert_eq!(message, "Access denied");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[test]
fn test_parse_google_error_resource_exhausted() {
    let response = r#"{"error": {"status": "RESOURCE_EXHAUSTED", "message": "Quota exceeded"}}"#;
    let error = ErrorUtils::parse_google_error(response);
    match error {
        ProviderError::RateLimit {
            provider, message, ..
        } => {
            assert_eq!(provider, "google");
            assert_eq!(message, "Quota exceeded");
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_parse_google_error_not_found() {
    let response = r#"{"error": {"status": "NOT_FOUND", "message": "Model not found"}}"#;
    let error = ErrorUtils::parse_google_error(response);
    match error {
        ProviderError::ModelNotFound { provider, model } => {
            assert_eq!(provider, "google");
            assert_eq!(model, "Model not found");
        }
        _ => panic!("Expected ModelNotFound error"),
    }
}

#[test]
fn test_parse_google_error_internal() {
    let response = r#"{"error": {"status": "INTERNAL", "message": "Internal error"}}"#;
    let error = ErrorUtils::parse_google_error(response);
    match error {
        ProviderError::Other { provider, message } => {
            assert_eq!(provider, "google");
            assert_eq!(message, "Internal error");
        }
        _ => panic!("Expected Other error"),
    }
}

#[test]
fn test_parse_google_error_unavailable() {
    let response = r#"{"error": {"status": "UNAVAILABLE", "message": "Service unavailable"}}"#;
    let error = ErrorUtils::parse_google_error(response);
    match error {
        ProviderError::ProviderUnavailable { provider, message } => {
            assert_eq!(provider, "google");
            assert_eq!(message, "Service unavailable");
        }
        _ => panic!("Expected ProviderUnavailable error"),
    }
}

#[test]
fn test_parse_google_error_unknown_status() {
    let response = r#"{"error": {"status": "UNKNOWN_STATUS", "message": "Something went wrong"}}"#;
    let error = ErrorUtils::parse_google_error(response);
    match error {
        ProviderError::Other { provider, message } => {
            assert_eq!(provider, "google");
            assert_eq!(message, "Something went wrong");
        }
        _ => panic!("Expected Other error"),
    }
}

#[test]
fn test_parse_google_error_invalid_json() {
    let response = "not json";
    let error = ErrorUtils::parse_google_error(response);
    match error {
        ProviderError::Other { provider, message } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "not json");
        }
        _ => panic!("Expected Other error"),
    }
}

#[test]
fn test_parse_provider_error_openai() {
    let response = r#"{"error": {"type": "invalid_request_error", "message": "Invalid model"}}"#;
    let error = ErrorUtils::parse_provider_error("openai", 400, response);
    match error {
        ProviderError::InvalidRequest { provider, .. } => {
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_parse_provider_error_anthropic() {
    let response = r#"{"error": {"type": "authentication_error", "message": "Invalid API key"}}"#;
    let error = ErrorUtils::parse_provider_error("anthropic", 401, response);
    match error {
        ProviderError::Authentication { provider, .. } => {
            assert_eq!(provider, "anthropic");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[test]
fn test_parse_provider_error_google() {
    let response = r#"{"error": {"status": "INVALID_ARGUMENT", "message": "Invalid input"}}"#;
    let error = ErrorUtils::parse_provider_error("google", 400, response);
    match error {
        ProviderError::InvalidRequest { provider, .. } => {
            assert_eq!(provider, "google");
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_parse_provider_error_unknown_provider() {
    let response = "Error message";
    let error = ErrorUtils::parse_provider_error("unknown-provider", 500, response);
    match error {
        ProviderError::ProviderUnavailable { provider, message } => {
            assert_eq!(provider, "unknown");
            assert_eq!(message, "Error message");
        }
        _ => panic!("Expected ProviderUnavailable error"),
    }
}

#[test]
fn test_parse_provider_error_case_insensitive() {
    let response = r#"{"error": {"type": "invalid_request_error", "message": "Invalid"}}"#;
    let error = ErrorUtils::parse_provider_error("OpenAI", 400, response);
    match error {
        ProviderError::InvalidRequest { provider, .. } => {
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[test]
fn test_format_error_for_user_authentication() {
    let error = ProviderError::Authentication {
        provider: "openai",
        message: "Invalid API key".to_string(),
    };
    let formatted = ErrorUtils::format_error_for_user(&error);
    assert_eq!(formatted, "Authentication failed: Invalid API key");
}

#[test]
fn test_format_error_for_user_invalid_request() {
    let error = ProviderError::InvalidRequest {
        provider: "openai",
        message: "Missing required field".to_string(),
    };
    let formatted = ErrorUtils::format_error_for_user(&error);
    assert_eq!(
        formatted,
        "Request validation failed: Missing required field"
    );
}

#[test]
fn test_format_error_for_user_rate_limit() {
    let error = ProviderError::RateLimit {
        provider: "openai",
        message: "Too many requests".to_string(),
        retry_after: Some(60),
        rpm_limit: None,
        tpm_limit: None,
        current_usage: None,
    };
    let formatted = ErrorUtils::format_error_for_user(&error);
    assert_eq!(formatted, "Rate limit exceeded: Too many requests");
}

#[test]
fn test_format_error_for_user_quota_exceeded() {
    let error = ProviderError::QuotaExceeded {
        provider: "openai",
        message: "Monthly quota exceeded".to_string(),
    };
    let formatted = ErrorUtils::format_error_for_user(&error);
    assert_eq!(formatted, "Quota exceeded: Monthly quota exceeded");
}

#[test]
fn test_format_error_for_user_model_not_found() {
    let error = ProviderError::ModelNotFound {
        provider: "openai",
        model: "gpt-unknown".to_string(),
    };
    let formatted = ErrorUtils::format_error_for_user(&error);
    assert_eq!(formatted, "Model not supported: gpt-unknown");
}

#[test]
fn test_format_error_for_user_timeout() {
    let error = ProviderError::Timeout {
        provider: "openai",
        message: "Request timed out after 30s".to_string(),
    };
    let formatted = ErrorUtils::format_error_for_user(&error);
    assert_eq!(formatted, "Request timeout: Request timed out after 30s");
}

#[test]
fn test_format_error_for_user_network() {
    let error = ProviderError::Network {
        provider: "openai",
        message: "Connection refused".to_string(),
    };
    let formatted = ErrorUtils::format_error_for_user(&error);
    assert_eq!(formatted, "Network error: Connection refused");
}

#[test]
fn test_format_error_for_user_provider_unavailable() {
    let error = ProviderError::ProviderUnavailable {
        provider: "openai",
        message: "Service is down".to_string(),
    };
    let formatted = ErrorUtils::format_error_for_user(&error);
    assert_eq!(formatted, "Provider unavailable: Service is down");
}

#[test]
fn test_format_error_for_user_serialization() {
    let error = ProviderError::Serialization {
        provider: "openai",
        message: "Failed to parse JSON".to_string(),
    };
    let formatted = ErrorUtils::format_error_for_user(&error);
    assert_eq!(formatted, "Parsing error: Failed to parse JSON");
}

#[test]
fn test_format_error_for_user_other() {
    let error = ProviderError::Other {
        provider: "openai",
        message: "Unknown error".to_string(),
    };
    let formatted = ErrorUtils::format_error_for_user(&error);
    assert_eq!(formatted, "Provider error: Unknown error");
}

#[test]
fn test_get_error_category_client_errors() {
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::InvalidRequest {
            provider: "test",
            message: "test".to_string()
        }),
        ErrorCategory::ClientError
    );
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::Authentication {
            provider: "test",
            message: "test".to_string()
        }),
        ErrorCategory::ClientError
    );
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::ModelNotFound {
            provider: "test",
            model: "test".to_string()
        }),
        ErrorCategory::ClientError
    );
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::QuotaExceeded {
            provider: "test",
            message: "test".to_string()
        }),
        ErrorCategory::ClientError
    );
}

#[test]
fn test_get_error_category_transient_errors() {
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::RateLimit {
            provider: "test",
            message: "test".to_string(),
            retry_after: None,
            rpm_limit: None,
            tpm_limit: None,
            current_usage: None,
        }),
        ErrorCategory::TransientError
    );
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::Network {
            provider: "test",
            message: "test".to_string()
        }),
        ErrorCategory::TransientError
    );
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::Timeout {
            provider: "test",
            message: "test".to_string()
        }),
        ErrorCategory::TransientError
    );
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::ProviderUnavailable {
            provider: "test",
            message: "test".to_string()
        }),
        ErrorCategory::TransientError
    );
}

#[test]
fn test_get_error_category_permanent_errors() {
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::Configuration {
            provider: "test",
            message: "test".to_string()
        }),
        ErrorCategory::PermanentError
    );
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::NotSupported {
            provider: "test",
            feature: "test".to_string()
        }),
        ErrorCategory::PermanentError
    );
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::NotImplemented {
            provider: "test",
            feature: "test".to_string()
        }),
        ErrorCategory::PermanentError
    );
}

#[test]
fn test_get_error_category_server_error_default() {
    assert_eq!(
        ErrorUtils::get_error_category(&ProviderError::Other {
            provider: "test",
            message: "test".to_string()
        }),
        ErrorCategory::ServerError
    );
}

#[test]
fn test_should_retry_retryable_errors() {
    assert!(ErrorUtils::should_retry(&ProviderError::Network {
        provider: "test",
        message: "test".to_string()
    }));
    assert!(ErrorUtils::should_retry(&ProviderError::Timeout {
        provider: "test",
        message: "test".to_string()
    }));
    assert!(ErrorUtils::should_retry(
        &ProviderError::ProviderUnavailable {
            provider: "test",
            message: "test".to_string()
        }
    ));
    assert!(ErrorUtils::should_retry(&ProviderError::RateLimit {
        provider: "test",
        message: "test".to_string(),
        retry_after: None,
        rpm_limit: None,
        tpm_limit: None,
        current_usage: None,
    }));
}

#[test]
fn test_should_retry_non_retryable_errors() {
    assert!(!ErrorUtils::should_retry(&ProviderError::InvalidRequest {
        provider: "test",
        message: "test".to_string()
    }));
    assert!(!ErrorUtils::should_retry(&ProviderError::Authentication {
        provider: "test",
        message: "test".to_string()
    }));
    assert!(!ErrorUtils::should_retry(&ProviderError::ModelNotFound {
        provider: "test",
        model: "test".to_string()
    }));
    assert!(!ErrorUtils::should_retry(&ProviderError::QuotaExceeded {
        provider: "test",
        message: "test".to_string()
    }));
    assert!(!ErrorUtils::should_retry(&ProviderError::Configuration {
        provider: "test",
        message: "test".to_string()
    }));
}

#[test]
fn test_get_retry_delay_rate_limit_with_retry_after() {
    let error = ProviderError::RateLimit {
        provider: "test",
        message: "test".to_string(),
        retry_after: Some(120),
        rpm_limit: None,
        tpm_limit: None,
        current_usage: None,
    };
    assert_eq!(
        ErrorUtils::get_retry_delay(&error),
        Duration::from_secs(120)
    );
}

#[test]
fn test_get_retry_delay_rate_limit_without_retry_after() {
    let error = ProviderError::RateLimit {
        provider: "test",
        message: "test".to_string(),
        retry_after: None,
        rpm_limit: None,
        tpm_limit: None,
        current_usage: None,
    };
    assert_eq!(ErrorUtils::get_retry_delay(&error), Duration::from_secs(60));
}

#[test]
fn test_get_retry_delay_provider_unavailable() {
    let error = ProviderError::ProviderUnavailable {
        provider: "test",
        message: "test".to_string(),
    };
    assert_eq!(ErrorUtils::get_retry_delay(&error), Duration::from_secs(5));
}

#[test]
fn test_get_retry_delay_network() {
    let error = ProviderError::Network {
        provider: "test",
        message: "test".to_string(),
    };
    assert_eq!(ErrorUtils::get_retry_delay(&error), Duration::from_secs(1));
}

#[test]
fn test_get_retry_delay_timeout() {
    let error = ProviderError::Timeout {
        provider: "test",
        message: "test".to_string(),
    };
    assert_eq!(ErrorUtils::get_retry_delay(&error), Duration::from_secs(2));
}

#[test]
fn test_get_retry_delay_default() {
    let error = ProviderError::Other {
        provider: "test",
        message: "test".to_string(),
    };
    assert_eq!(ErrorUtils::get_retry_delay(&error), Duration::from_secs(1));
}

#[test]
fn test_error_context_serialization() {
    let mut metadata = HashMap::new();
    metadata.insert("key".to_string(), "value".to_string());

    let context = ErrorContext {
        error_type: "RateLimit".to_string(),
        message: "Too many requests".to_string(),
        provider: "openai".to_string(),
        request_id: Some("req-123".to_string()),
        timestamp: 1234567890,
        metadata,
    };

    // Test serialization
    let json = serde_json::to_string(&context).unwrap();
    assert!(json.contains("RateLimit"));
    assert!(json.contains("Too many requests"));
    assert!(json.contains("openai"));
    assert!(json.contains("req-123"));

    // Test deserialization
    let deserialized: ErrorContext = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.error_type, "RateLimit");
    assert_eq!(deserialized.message, "Too many requests");
    assert_eq!(deserialized.provider, "openai");
    assert_eq!(deserialized.request_id, Some("req-123".to_string()));
    assert_eq!(deserialized.timestamp, 1234567890);
    assert_eq!(deserialized.metadata.get("key"), Some(&"value".to_string()));
}

#[test]
fn test_error_category_equality() {
    assert_eq!(ErrorCategory::ClientError, ErrorCategory::ClientError);
    assert_eq!(ErrorCategory::ServerError, ErrorCategory::ServerError);
    assert_eq!(ErrorCategory::TransientError, ErrorCategory::TransientError);
    assert_eq!(ErrorCategory::PermanentError, ErrorCategory::PermanentError);
    assert_ne!(ErrorCategory::ClientError, ErrorCategory::ServerError);
}
