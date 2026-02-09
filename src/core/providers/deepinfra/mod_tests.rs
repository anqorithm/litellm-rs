use super::*;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

// DeepInfraConfig tests
#[test]
fn test_deepinfra_config_default() {
    let config = DeepInfraConfig::default();
    assert!(config.api_key.is_none());
    assert_eq!(
        config.api_base,
        Some("https://api.deepinfra.com".to_string())
    );
    assert_eq!(config.timeout, 60);
    assert_eq!(config.max_retries, 3);
}

#[test]
fn test_deepinfra_config_with_api_key() {
    let config = DeepInfraConfig {
        api_key: Some("test-key".to_string()),
        ..Default::default()
    };
    assert_eq!(
        config.get_effective_api_key(),
        Some(&"test-key".to_string())
    );
}

#[test]
fn test_deepinfra_config_get_effective_api_base() {
    let config = DeepInfraConfig::default();
    assert_eq!(config.get_effective_api_base(), "https://api.deepinfra.com");

    let config_custom = DeepInfraConfig {
        api_base: Some("https://custom.api.com".to_string()),
        ..Default::default()
    };
    assert_eq!(
        config_custom.get_effective_api_base(),
        "https://custom.api.com"
    );

    let config_none = DeepInfraConfig {
        api_base: None,
        ..Default::default()
    };
    assert_eq!(
        config_none.get_effective_api_base(),
        "https://api.deepinfra.com"
    );
}

#[test]
fn test_deepinfra_config_validate_missing_key() {
    let config = DeepInfraConfig::default();
    let result = config.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "DeepInfra API key is required");
}

#[test]
fn test_deepinfra_config_validate_with_key() {
    let config = DeepInfraConfig {
        api_key: Some("test-key".to_string()),
        ..Default::default()
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_deepinfra_config_provider_trait() {
    let config = DeepInfraConfig {
        api_key: Some("my-key".to_string()),
        api_base: Some("https://api.example.com".to_string()),
        timeout: 120,
        max_retries: 5,
    };

    assert_eq!(config.api_key(), Some("my-key"));
    assert_eq!(config.api_base(), Some("https://api.example.com"));
    assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
    assert_eq!(config.max_retries(), 5);
}

// ProviderError tests (using unified error type)
#[test]
fn test_deepinfra_error_display() {
    let err = ProviderError::configuration("deepinfra", "missing config");
    assert!(err.to_string().contains("missing config"));

    let err = ProviderError::authentication("deepinfra", "bad key");
    assert!(err.to_string().contains("bad key"));

    let err = ProviderError::network("deepinfra", "timeout");
    assert!(err.to_string().contains("timeout"));

    let err = ProviderError::api_error("deepinfra", 500, "server error");
    assert!(err.to_string().contains("500"));
    assert!(err.to_string().contains("server error"));

    let err = ProviderError::serialization("deepinfra", "parse error");
    assert!(err.to_string().contains("parse error"));

    let err = ProviderError::rate_limit_simple("deepinfra", "too many requests");
    assert!(err.to_string().contains("too many requests"));

    let err = ProviderError::not_implemented("deepinfra", "streaming");
    assert!(err.to_string().contains("streaming"));

    let err = ProviderError::model_not_found("deepinfra", "gpt-5");
    assert!(err.to_string().contains("gpt-5"));
}

#[test]
fn test_deepinfra_error_is_retryable() {
    assert!(ProviderError::network("deepinfra", "").is_retryable());
    assert!(ProviderError::rate_limit("deepinfra", None).is_retryable());
    assert!(ProviderError::api_error("deepinfra", 500, "").is_retryable());
    assert!(ProviderError::api_error("deepinfra", 503, "").is_retryable());
    assert!(ProviderError::api_error("deepinfra", 429, "").is_retryable());

    assert!(!ProviderError::configuration("deepinfra", "").is_retryable());
    assert!(!ProviderError::authentication("deepinfra", "").is_retryable());
    assert!(!ProviderError::api_error("deepinfra", 400, "").is_retryable());
    assert!(!ProviderError::api_error("deepinfra", 404, "").is_retryable());
    assert!(!ProviderError::invalid_request("deepinfra", "").is_retryable());
}

#[test]
fn test_deepinfra_error_retry_delay() {
    assert_eq!(
        ProviderError::network("deepinfra", "").retry_delay(),
        Some(1)
    );
    assert!(
        ProviderError::rate_limit("deepinfra", Some(60))
            .retry_delay()
            .is_some()
    );
    assert_eq!(
        ProviderError::api_error("deepinfra", 429, "").retry_delay(),
        Some(60)
    );
    assert_eq!(
        ProviderError::api_error("deepinfra", 500, "").retry_delay(),
        Some(3)
    );
    assert_eq!(
        ProviderError::api_error("deepinfra", 503, "").retry_delay(),
        Some(3)
    );
    assert_eq!(
        ProviderError::configuration("deepinfra", "").retry_delay(),
        None
    );
    assert_eq!(
        ProviderError::authentication("deepinfra", "").retry_delay(),
        None
    );
}

#[test]
fn test_deepinfra_error_http_status() {
    assert_eq!(
        ProviderError::api_error("deepinfra", 503, "").http_status(),
        503
    );
    assert_eq!(
        ProviderError::authentication("deepinfra", "").http_status(),
        401
    );
    assert_eq!(
        ProviderError::configuration("deepinfra", "").http_status(),
        400
    );
    assert_eq!(
        ProviderError::invalid_request("deepinfra", "").http_status(),
        400
    );
    assert_eq!(
        ProviderError::rate_limit("deepinfra", None).http_status(),
        429
    );
    assert_eq!(
        ProviderError::model_not_found("deepinfra", "").http_status(),
        404
    );
    assert_eq!(
        ProviderError::not_implemented("deepinfra", "").http_status(),
        501
    );
    assert_eq!(ProviderError::network("deepinfra", "").http_status(), 503);
    assert_eq!(
        ProviderError::serialization("deepinfra", "").http_status(),
        500
    );
}

#[test]
fn test_deepinfra_error_factory_methods() {
    let err = ProviderError::not_supported("deepinfra", "vision");
    assert!(matches!(err, ProviderError::NotSupported { .. }));

    let err = ProviderError::authentication("deepinfra", "bad token");
    assert!(matches!(err, ProviderError::Authentication { .. }));

    let err = ProviderError::rate_limit("deepinfra", Some(30));
    assert!(matches!(err, ProviderError::RateLimit { .. }));

    let err = ProviderError::network("deepinfra", "timeout");
    assert!(matches!(err, ProviderError::Network { .. }));

    let err = ProviderError::serialization("deepinfra", "invalid json");
    assert!(matches!(err, ProviderError::Serialization { .. }));

    let err = ProviderError::not_implemented("deepinfra", "streaming");
    assert!(matches!(err, ProviderError::NotImplemented { .. }));
}

// DeepInfraErrorMapper tests
#[test]
fn test_deepinfra_error_mapper_401() {
    let mapper = DeepInfraErrorMapper;
    let err = mapper.map_http_error(401, "invalid key");
    assert!(matches!(err, ProviderError::Authentication { .. }));
}

#[test]
fn test_deepinfra_error_mapper_403() {
    let mapper = DeepInfraErrorMapper;
    let err = mapper.map_http_error(403, "forbidden");
    assert!(matches!(err, ProviderError::Authentication { .. }));
}

#[test]
fn test_deepinfra_error_mapper_404() {
    let mapper = DeepInfraErrorMapper;
    let err = mapper.map_http_error(404, "not found");
    assert!(matches!(err, ProviderError::ModelNotFound { .. }));
}

#[test]
fn test_deepinfra_error_mapper_429() {
    let mapper = DeepInfraErrorMapper;
    let err = mapper.map_http_error(429, "rate limited");
    assert!(matches!(err, ProviderError::RateLimit { .. }));
}

#[test]
fn test_deepinfra_error_mapper_500() {
    let mapper = DeepInfraErrorMapper;
    let err = mapper.map_http_error(500, "server error");
    assert!(matches!(err, ProviderError::ApiError { status: 500, .. }));
}

#[test]
fn test_deepinfra_error_mapper_503() {
    let mapper = DeepInfraErrorMapper;
    let err = mapper.map_http_error(503, "service unavailable");
    assert!(matches!(err, ProviderError::ApiError { status: 503, .. }));
}

#[test]
fn test_deepinfra_error_mapper_unknown() {
    let mapper = DeepInfraErrorMapper;
    let err = mapper.map_http_error(418, "teapot");
    assert!(matches!(err, ProviderError::ApiError { status: 418, .. }));
}

// DeepInfraProvider tests
#[test]
fn test_deepinfra_provider_supports_model() {
    let config = DeepInfraConfig {
        api_key: Some("test".to_string()),
        ..Default::default()
    };
    let provider = DeepInfraProvider::new(config).unwrap();

    assert!(provider.supports_model("meta-llama/Llama-2-70b"));
    assert!(provider.supports_model("mistralai/Mixtral-8x7B"));
    assert!(provider.supports_model("tiiuae/falcon-40b"));
    assert!(!provider.supports_model("gpt-4"));
    assert!(!provider.supports_model("claude-3"));
}

#[test]
fn test_deepinfra_provider_name() {
    let config = DeepInfraConfig {
        api_key: Some("test".to_string()),
        ..Default::default()
    };
    let provider = DeepInfraProvider::new(config).unwrap();
    assert_eq!(provider.name(), "deepinfra");
}

#[test]
fn test_deepinfra_provider_capabilities() {
    let config = DeepInfraConfig {
        api_key: Some("test".to_string()),
        ..Default::default()
    };
    let provider = DeepInfraProvider::new(config).unwrap();
    let capabilities = provider.capabilities();

    assert!(capabilities.contains(&ProviderCapability::ChatCompletion));
    assert!(capabilities.contains(&ProviderCapability::ChatCompletionStream));
}

#[test]
fn test_deepinfra_provider_get_supported_openai_params() {
    let config = DeepInfraConfig {
        api_key: Some("test".to_string()),
        ..Default::default()
    };
    let provider = DeepInfraProvider::new(config).unwrap();
    let params = provider.get_supported_openai_params("any-model");

    assert!(params.contains(&"temperature"));
    assert!(params.contains(&"max_tokens"));
    assert!(params.contains(&"top_p"));
    assert!(params.contains(&"stream"));
}

#[tokio::test]
async fn test_deepinfra_provider_calculate_cost() {
    let config = DeepInfraConfig {
        api_key: Some("test".to_string()),
        ..Default::default()
    };
    let provider = DeepInfraProvider::new(config).unwrap();

    let cost = provider
        .calculate_cost("meta-llama/Llama-2-70b-chat-hf", 1000, 1000)
        .await
        .unwrap();
    assert!(cost > 0.0);

    let cost_unknown = provider
        .calculate_cost("unknown-model", 1000, 1000)
        .await
        .unwrap();
    assert_eq!(cost_unknown, 0.0);
}

#[tokio::test]
async fn test_deepinfra_provider_health_check_with_key() {
    let config = DeepInfraConfig {
        api_key: Some("test".to_string()),
        ..Default::default()
    };
    let provider = DeepInfraProvider::new(config).unwrap();
    let status = provider.health_check().await;
    assert_eq!(status, HealthStatus::Healthy);
}

#[tokio::test]
async fn test_deepinfra_provider_health_check_without_key() {
    let config = DeepInfraConfig {
        api_key: None,
        ..Default::default()
    };
    let provider = DeepInfraProvider::new(config).unwrap();
    let status = provider.health_check().await;
    assert_eq!(status, HealthStatus::Unhealthy);
}
