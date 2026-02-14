#[cfg(test)]
use super::types::{HttpClientConfig, RequestMetrics, RetryConfig};
use super::utils::ClientUtils;
use std::collections::HashMap;
use std::time::Duration;

// ==================== Retry Logic Tests ====================

#[test]
fn test_retry_logic() {
    assert!(ClientUtils::should_retry_request(429, 0, 3));
    assert!(ClientUtils::should_retry_request(500, 0, 3));
    assert!(ClientUtils::should_retry_request(502, 1, 3));
    assert!(!ClientUtils::should_retry_request(400, 0, 3));
    assert!(!ClientUtils::should_retry_request(429, 3, 3));
}

#[test]
fn test_retry_logic_all_retryable_codes() {
    // Rate limit
    assert!(ClientUtils::should_retry_request(429, 0, 3));

    // Request timeout
    assert!(ClientUtils::should_retry_request(408, 0, 3));

    // Server errors
    assert!(ClientUtils::should_retry_request(500, 0, 3));
    assert!(ClientUtils::should_retry_request(501, 0, 3));
    assert!(ClientUtils::should_retry_request(502, 0, 3));
    assert!(ClientUtils::should_retry_request(503, 0, 3));
    assert!(ClientUtils::should_retry_request(504, 0, 3));
    assert!(ClientUtils::should_retry_request(599, 0, 3));
}

#[test]
fn test_retry_logic_non_retryable_codes() {
    // Client errors (except 408, 429)
    assert!(!ClientUtils::should_retry_request(400, 0, 3));
    assert!(!ClientUtils::should_retry_request(401, 0, 3));
    assert!(!ClientUtils::should_retry_request(403, 0, 3));
    assert!(!ClientUtils::should_retry_request(404, 0, 3));
    assert!(!ClientUtils::should_retry_request(422, 0, 3));

    // Success codes
    assert!(!ClientUtils::should_retry_request(200, 0, 3));
    assert!(!ClientUtils::should_retry_request(201, 0, 3));

    // Redirect codes
    assert!(!ClientUtils::should_retry_request(301, 0, 3));
    assert!(!ClientUtils::should_retry_request(302, 0, 3));
}

#[test]
fn test_retry_logic_max_retries() {
    // Should not retry when at max attempts
    assert!(!ClientUtils::should_retry_request(500, 3, 3));
    assert!(!ClientUtils::should_retry_request(500, 4, 3));
    assert!(!ClientUtils::should_retry_request(429, 5, 3));
}

// ==================== Timeout Tests ====================

#[test]
fn test_timeout_for_provider() {
    assert_eq!(
        ClientUtils::get_timeout_for_provider("openai"),
        Duration::from_secs(120)
    );
    assert_eq!(
        ClientUtils::get_timeout_for_provider("anthropic"),
        Duration::from_secs(180)
    );
    assert_eq!(
        ClientUtils::get_timeout_for_provider("unknown"),
        Duration::from_secs(60)
    );
}

#[test]
fn test_timeout_for_all_known_providers() {
    assert_eq!(
        ClientUtils::get_timeout_for_provider("openai"),
        Duration::from_secs(120)
    );
    assert_eq!(
        ClientUtils::get_timeout_for_provider("anthropic"),
        Duration::from_secs(180)
    );
    assert_eq!(
        ClientUtils::get_timeout_for_provider("google"),
        Duration::from_secs(90)
    );
    assert_eq!(
        ClientUtils::get_timeout_for_provider("azure"),
        Duration::from_secs(120)
    );
    assert_eq!(
        ClientUtils::get_timeout_for_provider("cohere"),
        Duration::from_secs(60)
    );
}

#[test]
fn test_timeout_case_insensitive() {
    assert_eq!(
        ClientUtils::get_timeout_for_provider("OpenAI"),
        Duration::from_secs(120)
    );
    assert_eq!(
        ClientUtils::get_timeout_for_provider("ANTHROPIC"),
        Duration::from_secs(180)
    );
    assert_eq!(
        ClientUtils::get_timeout_for_provider("Google"),
        Duration::from_secs(90)
    );
}

// ==================== URL Path Tests ====================

#[test]
fn test_add_path_to_api_base() {
    assert_eq!(
        ClientUtils::add_path_to_api_base("https://api.openai.com", "/v1/chat/completions"),
        "https://api.openai.com/v1/chat/completions"
    );

    assert_eq!(
        ClientUtils::add_path_to_api_base("https://api.openai.com/", "v1/chat/completions"),
        "https://api.openai.com/v1/chat/completions"
    );
}

#[test]
fn test_add_path_edge_cases() {
    // Both have slashes
    assert_eq!(
        ClientUtils::add_path_to_api_base("https://api.example.com/", "/v1/endpoint"),
        "https://api.example.com/v1/endpoint"
    );

    // Neither have slashes
    assert_eq!(
        ClientUtils::add_path_to_api_base("https://api.example.com", "v1/endpoint"),
        "https://api.example.com/v1/endpoint"
    );

    // Empty path
    assert_eq!(
        ClientUtils::add_path_to_api_base("https://api.example.com", ""),
        "https://api.example.com/"
    );

    // Base with trailing slash, empty path
    assert_eq!(
        ClientUtils::add_path_to_api_base("https://api.example.com/", ""),
        "https://api.example.com/"
    );
}

// ==================== URL Validation Tests ====================

#[test]
fn test_url_validation() {
    assert!(ClientUtils::validate_url("https://api.openai.com").is_ok());
    assert!(ClientUtils::validate_url("http://localhost:8080").is_ok());
    assert!(ClientUtils::validate_url("ftp://example.com").is_err());
    assert!(ClientUtils::validate_url("not-a-url").is_err());
}

#[test]
fn test_url_validation_various_formats() {
    // Valid URLs
    assert!(ClientUtils::validate_url("https://api.example.com/v1").is_ok());
    assert!(ClientUtils::validate_url("http://127.0.0.1:3000").is_ok());
    assert!(ClientUtils::validate_url("https://user:pass@example.com").is_ok());
    assert!(ClientUtils::validate_url("http://[::1]:8080").is_ok());

    // Invalid URLs
    assert!(ClientUtils::validate_url("ws://example.com").is_err());
    assert!(ClientUtils::validate_url("file:///path/to/file").is_err());
    assert!(ClientUtils::validate_url("").is_err());
    assert!(ClientUtils::validate_url("://missing.scheme").is_err());
}

// ==================== HTTPX Timeout Support Tests ====================

#[test]
fn test_supports_httpx_timeout() {
    assert!(ClientUtils::supports_httpx_timeout("openai"));
    assert!(ClientUtils::supports_httpx_timeout("anthropic"));
    assert!(!ClientUtils::supports_httpx_timeout("unknown"));
}

#[test]
fn test_supports_httpx_timeout_all_providers() {
    assert!(ClientUtils::supports_httpx_timeout("openai"));
    assert!(ClientUtils::supports_httpx_timeout("anthropic"));
    assert!(ClientUtils::supports_httpx_timeout("google"));
    assert!(ClientUtils::supports_httpx_timeout("azure"));
    assert!(ClientUtils::supports_httpx_timeout("cohere"));
    assert!(ClientUtils::supports_httpx_timeout("mistral"));
    assert!(ClientUtils::supports_httpx_timeout("replicate"));

    // Unsupported
    assert!(!ClientUtils::supports_httpx_timeout("custom"));
    assert!(!ClientUtils::supports_httpx_timeout(""));
}

#[test]
fn test_supports_httpx_timeout_case_insensitive() {
    assert!(ClientUtils::supports_httpx_timeout("OpenAI"));
    assert!(ClientUtils::supports_httpx_timeout("ANTHROPIC"));
    assert!(ClientUtils::supports_httpx_timeout("Mistral"));
}

// ==================== User Agent Tests ====================

#[test]
fn test_user_agent_for_provider() {
    assert_eq!(
        ClientUtils::get_user_agent_for_provider("openai"),
        "litellm-rust-openai/1.0"
    );
    assert_eq!(
        ClientUtils::get_user_agent_for_provider("unknown"),
        "litellm-rust/1.0"
    );
}

#[test]
fn test_user_agent_for_all_providers() {
    assert_eq!(
        ClientUtils::get_user_agent_for_provider("openai"),
        "litellm-rust-openai/1.0"
    );
    assert_eq!(
        ClientUtils::get_user_agent_for_provider("anthropic"),
        "litellm-rust-anthropic/1.0"
    );
    assert_eq!(
        ClientUtils::get_user_agent_for_provider("google"),
        "litellm-rust-google/1.0"
    );
    assert_eq!(
        ClientUtils::get_user_agent_for_provider("default"),
        "litellm-rust/1.0"
    );
}

// ==================== Content-Type Parsing Tests ====================

#[test]
fn test_parse_content_type() {
    let (media_type, params) =
        ClientUtils::parse_content_type("text/html; charset=utf-8; boundary=something");
    assert_eq!(media_type, "text/html");
    assert_eq!(params.get("charset"), Some(&"utf-8".to_string()));
    assert_eq!(params.get("boundary"), Some(&"something".to_string()));
}

#[test]
fn test_parse_content_type_simple() {
    let (media_type, params) = ClientUtils::parse_content_type("application/json");
    assert_eq!(media_type, "application/json");
    assert!(params.is_empty());
}

#[test]
fn test_parse_content_type_with_quotes() {
    let (media_type, params) = ClientUtils::parse_content_type("text/plain; charset=\"UTF-8\"");
    assert_eq!(media_type, "text/plain");
    assert_eq!(params.get("charset"), Some(&"UTF-8".to_string()));
}

#[test]
fn test_parse_content_type_case_normalization() {
    let (media_type, _) = ClientUtils::parse_content_type("Application/JSON");
    assert_eq!(media_type, "application/json");
}

// ==================== Default Headers Tests ====================

#[test]
fn test_get_default_headers_common() {
    let headers = ClientUtils::get_default_headers_for_provider("openai");
    assert_eq!(
        headers.get("Content-Type"),
        Some(&"application/json".to_string())
    );
    assert_eq!(headers.get("Accept"), Some(&"application/json".to_string()));
}

#[test]
fn test_get_default_headers_anthropic() {
    let headers = ClientUtils::get_default_headers_for_provider("anthropic");
    assert_eq!(
        headers.get("anthropic-version"),
        Some(&"2023-06-01".to_string())
    );
}

#[test]
fn test_get_default_headers_google() {
    let headers = ClientUtils::get_default_headers_for_provider("google");
    assert!(headers.contains_key("x-goog-api-key"));
}

#[test]
fn test_get_default_headers_azure() {
    let headers = ClientUtils::get_default_headers_for_provider("azure");
    assert!(headers.contains_key("api-key"));
}

// ==================== RequestMetrics Tests ====================

#[test]
fn test_request_metrics() {
    let mut metrics = RequestMetrics::new("openai".to_string(), "gpt-4".to_string());
    assert_eq!(metrics.retry_count, 0);
    assert!(metrics.end_time.is_none());

    metrics.increment_retry();
    assert_eq!(metrics.retry_count, 1);

    metrics.finish(Some(200));
    assert!(metrics.end_time.is_some());
    assert_eq!(metrics.status_code, Some(200));
}

#[test]
fn test_request_metrics_initial_state() {
    let metrics = RequestMetrics::new("anthropic".to_string(), "claude-3".to_string());
    assert_eq!(metrics.provider, "anthropic");
    assert_eq!(metrics.model, "claude-3");
    assert_eq!(metrics.retry_count, 0);
    assert!(metrics.end_time.is_none());
    assert!(metrics.duration.is_none());
    assert!(metrics.status_code.is_none());
}

#[test]
fn test_request_metrics_multiple_retries() {
    let mut metrics = RequestMetrics::new("openai".to_string(), "gpt-4".to_string());

    metrics.increment_retry();
    assert_eq!(metrics.retry_count, 1);

    metrics.increment_retry();
    assert_eq!(metrics.retry_count, 2);

    metrics.increment_retry();
    assert_eq!(metrics.retry_count, 3);
}

#[test]
fn test_request_metrics_finish_with_error() {
    let mut metrics = RequestMetrics::new("openai".to_string(), "gpt-4".to_string());
    metrics.finish(Some(500));

    assert!(metrics.end_time.is_some());
    assert!(metrics.duration.is_some());
    assert_eq!(metrics.status_code, Some(500));
}

#[test]
fn test_request_metrics_finish_no_status() {
    let mut metrics = RequestMetrics::new("openai".to_string(), "gpt-4".to_string());
    metrics.finish(None);

    assert!(metrics.end_time.is_some());
    assert!(metrics.duration.is_some());
    assert!(metrics.status_code.is_none());
}

// ==================== HttpClientConfig Tests ====================

#[test]
fn test_http_client_config_default() {
    let config = HttpClientConfig::default();

    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.retry_delay, Duration::from_millis(1000));
    assert!(config.proxy.is_none());
    assert_eq!(config.user_agent, "litellm-rust/1.0");
    assert!(config.default_headers.is_empty());
}

#[test]
fn test_http_client_config_custom() {
    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "value".to_string());

    let config = HttpClientConfig {
        timeout: Duration::from_secs(120),
        max_retries: 5,
        retry_delay: Duration::from_millis(2000),
        proxy: Some("http://proxy.example.com:8080".to_string()),
        user_agent: "custom-agent/2.0".to_string(),
        default_headers: headers,
    };

    assert_eq!(config.timeout, Duration::from_secs(120));
    assert_eq!(config.max_retries, 5);
    assert_eq!(
        config.proxy,
        Some("http://proxy.example.com:8080".to_string())
    );
    assert_eq!(config.user_agent, "custom-agent/2.0");
    assert!(config.default_headers.contains_key("X-Custom-Header"));
}

#[test]
fn test_http_client_config_clone() {
    let config = HttpClientConfig::default();
    let cloned = config.clone();

    assert_eq!(cloned.timeout, config.timeout);
    assert_eq!(cloned.max_retries, config.max_retries);
    assert_eq!(cloned.user_agent, config.user_agent);
}

// ==================== RetryConfig Tests ====================

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();

    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_delay, Duration::from_millis(1000));
    assert_eq!(config.max_delay, Duration::from_secs(60));
    assert_eq!(config.backoff_multiplier, 2.0);
    assert!(config.jitter);
}

#[test]
fn test_retry_config_custom() {
    let config = RetryConfig {
        max_retries: 5,
        initial_delay: Duration::from_millis(500),
        max_delay: Duration::from_secs(30),
        backoff_multiplier: 1.5,
        jitter: false,
    };

    assert_eq!(config.max_retries, 5);
    assert_eq!(config.initial_delay, Duration::from_millis(500));
    assert_eq!(config.max_delay, Duration::from_secs(30));
    assert_eq!(config.backoff_multiplier, 1.5);
    assert!(!config.jitter);
}

#[test]
fn test_retry_config_clone() {
    let config = RetryConfig::default();
    let cloned = config.clone();

    assert_eq!(cloned.max_retries, config.max_retries);
    assert_eq!(cloned.initial_delay, config.initial_delay);
    assert_eq!(cloned.jitter, config.jitter);
}

// ==================== Calculate Retry Delay Tests ====================

#[test]
fn test_calculate_retry_delay_with_server_delay() {
    let config = RetryConfig::default();
    let server_delay = Duration::from_secs(10);

    let delay = ClientUtils::calculate_retry_delay(&config, 0, Some(server_delay));
    assert_eq!(delay, server_delay);
}

#[test]
fn test_calculate_retry_delay_exponential() {
    let config = RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(1000),
        max_delay: Duration::from_secs(60),
        backoff_multiplier: 2.0,
        jitter: false,
    };

    let delay0 = ClientUtils::calculate_retry_delay(&config, 0, None);
    let delay1 = ClientUtils::calculate_retry_delay(&config, 1, None);
    let delay2 = ClientUtils::calculate_retry_delay(&config, 2, None);

    // Without jitter, delays should be exactly: 1000, 2000, 4000 ms
    assert_eq!(delay0, Duration::from_millis(1000));
    assert_eq!(delay1, Duration::from_millis(2000));
    assert_eq!(delay2, Duration::from_millis(4000));
}

#[test]
fn test_calculate_retry_delay_capped() {
    let config = RetryConfig {
        max_retries: 10,
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(5),
        backoff_multiplier: 2.0,
        jitter: false,
    };

    // After several retries, should be capped at max_delay
    let delay = ClientUtils::calculate_retry_delay(&config, 10, None);
    assert!(delay <= Duration::from_secs(5));
}

// ==================== Create HTTP Client Tests ====================

#[test]
fn test_create_http_client_default_config() {
    let config = HttpClientConfig::default();
    let result = ClientUtils::create_http_client(&config);
    assert!(result.is_ok());
}

#[test]
fn test_create_provider_specific_client() {
    let result = ClientUtils::create_provider_specific_client("openai");
    assert!(result.is_ok());

    let result = ClientUtils::create_provider_specific_client("anthropic");
    assert!(result.is_ok());

    let result = ClientUtils::create_provider_specific_client("unknown");
    assert!(result.is_ok());
}
