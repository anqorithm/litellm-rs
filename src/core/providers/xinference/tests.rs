//! Tests for Xinference provider

use super::model_info::get_model_info;
use super::*;
use crate::core::traits::provider::ProviderConfig;
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

#[test]
fn test_config_default() {
    let config = XinferenceConfig::default();
    assert!(config.api_key.is_none());
    assert!(config.api_base.is_none());
    assert_eq!(config.timeout, 120);
}

#[test]
fn test_get_api_base_default() {
    let config = XinferenceConfig::default();
    assert_eq!(config.get_api_base(), "http://localhost:9997/v1");
}

#[test]
fn test_config_validation() {
    let config = XinferenceConfig::default();
    assert!(config.validate().is_ok());

    let invalid = XinferenceConfig {
        timeout: 0,
        ..Default::default()
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn test_error_conversion() {
    use crate::core::providers::unified_provider::ProviderError;

    let err = ProviderError::authentication("xinference", "bad key");
    assert!(matches!(err, ProviderError::Authentication { .. }));
}

#[test]
fn test_model_info() {
    let info = get_model_info("llama-3-8b-instruct");
    assert!(info.is_some());
    assert!(info.unwrap().supports_tools);
}

#[tokio::test]
async fn test_provider_creation() {
    let provider = XinferenceProvider::new(XinferenceConfig::default()).await;
    assert!(provider.is_ok());
    assert_eq!(provider.unwrap().name(), "xinference");
}
