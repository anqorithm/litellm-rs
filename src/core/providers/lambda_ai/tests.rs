//! Tests for Lambda Labs AI Provider
//!
//! Unit tests for configuration, model info, and provider functionality.

use super::config::LambdaAIConfig;
use super::model_info::{get_available_models, get_model_info, is_reasoning_model};
use super::*;

#[test]
fn test_provider_name() {
    // Verify the provider name constant
    assert_eq!(provider::PROVIDER_NAME, "lambda_ai");
}

#[test]
fn test_default_config() {
    let config = LambdaAIConfig::default();
    assert!(config.api_key.is_none());
    assert!(config.api_base.is_none());
    assert_eq!(config.timeout, 120);
    assert_eq!(config.max_retries, 3);
}

#[test]
fn test_config_with_api_key() {
    let config = LambdaAIConfig::new("test-api-key");
    assert_eq!(config.api_key, Some("test-api-key".to_string()));
    assert_eq!(config.get_api_key(), Some("test-api-key".to_string()));
}

#[test]
fn test_config_get_api_base() {
    let config = LambdaAIConfig::default();
    assert_eq!(config.get_api_base(), "https://api.lambdalabs.com/v1");

    let config_custom = LambdaAIConfig::default().with_api_base("https://custom.lambda.com/v1");
    assert_eq!(config_custom.get_api_base(), "https://custom.lambda.com/v1");
}

#[test]
fn test_model_info_hermes() {
    let info = get_model_info("hermes-3-llama-3.1-405b-fp8");
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.model_id, "hermes-3-llama-3.1-405b-fp8");
    assert_eq!(info.display_name, "Hermes 3 Llama 3.1 405B FP8");
    assert!(info.supports_tools);
    assert!(!info.is_reasoning);
}

#[test]
fn test_model_info_llama() {
    let info = get_model_info("llama-3.3-70b-instruct-fp8");
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.model_id, "llama-3.3-70b-instruct-fp8");
    assert_eq!(info.max_context_length, 128000);
    assert!(info.supports_tools);
}

#[test]
fn test_model_info_deepseek() {
    let info = get_model_info("deepseek-r1");
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.model_id, "deepseek-r1");
    assert!(info.is_reasoning);
    assert!(info.supports_tools);
}

#[test]
fn test_reasoning_model_detection() {
    assert!(is_reasoning_model("deepseek-r1"));
    assert!(is_reasoning_model("deepseek-r1-671b"));
    assert!(!is_reasoning_model("hermes-3-llama-3.1-405b-fp8"));
    assert!(!is_reasoning_model("llama-3.3-70b-instruct-fp8"));
}

#[test]
fn test_available_models() {
    let models = get_available_models();
    assert!(!models.is_empty());

    // Check for expected models
    assert!(models.contains(&"hermes-3-llama-3.1-405b-fp8"));
    assert!(models.contains(&"llama-3.3-70b-instruct-fp8"));
    assert!(models.contains(&"deepseek-r1"));
    assert!(models.contains(&"qwen2.5-72b-instruct"));
}

#[test]
fn test_model_costs() {
    let info = get_model_info("llama-3.1-8b-instruct").unwrap();
    assert!(info.input_cost_per_million > 0.0);
    assert!(info.output_cost_per_million > 0.0);

    // Verify cost structure makes sense
    let hermes_info = get_model_info("hermes-3-llama-3.1-405b-fp8").unwrap();
    let llama_8b_info = get_model_info("llama-3.1-8b-instruct").unwrap();

    // Larger models should cost more
    assert!(hermes_info.input_cost_per_million > llama_8b_info.input_cost_per_million);
}

#[test]
fn test_config_builder_pattern() {
    let config = LambdaAIConfig::new("api-key")
        .with_api_base("https://custom.api.com")
        .with_timeout(60)
        .with_max_retries(5)
        .with_debug(true);

    assert_eq!(config.api_key, Some("api-key".to_string()));
    assert_eq!(config.api_base, Some("https://custom.api.com".to_string()));
    assert_eq!(config.timeout, 60);
    assert_eq!(config.max_retries, 5);
    assert!(config.debug);
}

#[test]
fn test_error_types() {
    use super::error::LambdaAIError;

    let auth_error = LambdaAIError::authentication("lambda_ai", "Invalid API key");
    assert_eq!(auth_error.provider(), "lambda_ai");
    assert!(!auth_error.is_retryable());

    let rate_error = LambdaAIError::rate_limit("lambda_ai", Some(60));
    assert!(rate_error.is_retryable());
    assert_eq!(rate_error.retry_delay(), Some(60));

    let network_error = LambdaAIError::network("lambda_ai", "Connection failed");
    assert!(network_error.is_retryable());
}

#[tokio::test]
async fn test_provider_creation_without_key() {
    // Clear any existing env var
    unsafe { std::env::remove_var("LAMBDA_API_KEY") };

    let config = LambdaAIConfig::default();
    let result = provider::LambdaAIProvider::new(config).await;

    // Should fail due to missing API key
    assert!(result.is_err());
}

#[tokio::test]
async fn test_provider_creation_with_key() {
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

    let config = LambdaAIConfig::new("test-api-key");
    let result = provider::LambdaAIProvider::new(config).await;

    // Should succeed with API key
    assert!(result.is_ok());

    let provider = result.unwrap();
    assert_eq!(provider.name(), "lambda_ai");
    assert!(!provider.models().is_empty());
}

#[tokio::test]
async fn test_provider_capabilities() {
    let config = LambdaAIConfig::new("test-api-key");
    let provider = provider::LambdaAIProvider::new(config).await.unwrap();

    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::ProviderCapability;

    let capabilities = provider.capabilities();
    assert!(capabilities.contains(&ProviderCapability::ChatCompletion));
    assert!(capabilities.contains(&ProviderCapability::ChatCompletionStream));
    assert!(capabilities.contains(&ProviderCapability::ToolCalling));
}

#[tokio::test]
async fn test_cost_calculation() {
    let config = LambdaAIConfig::new("test-api-key");
    let provider = provider::LambdaAIProvider::new(config).await.unwrap();

    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

    // Test cost calculation for a known model
    let cost = provider
        .calculate_cost("llama-3.1-8b-instruct", 1000, 500)
        .await;
    assert!(cost.is_ok());
    let cost_value = cost.unwrap();
    assert!(cost_value > 0.0);

    // Test with unknown model
    let unknown_cost = provider.calculate_cost("unknown-model", 1000, 500).await;
    assert!(unknown_cost.is_err());
}

#[test]
fn test_config_serialization() {
    let config = LambdaAIConfig::new("test-key")
        .with_api_base("https://custom.api.com")
        .with_timeout(90)
        .with_max_retries(2);

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("test-key"));
    assert!(json.contains("https://custom.api.com"));

    let deserialized: LambdaAIConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.api_key, config.api_key);
    assert_eq!(deserialized.api_base, config.api_base);
    assert_eq!(deserialized.timeout, config.timeout);
}

#[test]
fn test_128k_context_model() {
    let info = get_model_info("hermes-3-llama-3.1-405b-fp8-128k");
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.max_context_length, 128000);
}

#[test]
fn test_qwen_models() {
    let qwen = get_model_info("qwen2.5-72b-instruct");
    assert!(qwen.is_some());
    let qwen = qwen.unwrap();
    assert!(qwen.supports_tools);

    let coder = get_model_info("qwen2.5-coder-32b-instruct");
    assert!(coder.is_some());
}
