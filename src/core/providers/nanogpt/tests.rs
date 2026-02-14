//! Tests for NanoGPT provider

#[cfg(test)]
use super::*;

#[tokio::test]
async fn test_provider_creation() {
    let config = NanoGPTConfig {
        api_key: Some("test-key".to_string()),
        ..Default::default()
    };

    let provider = NanoGPTProvider::new(config).await;
    assert!(provider.is_ok());
}

#[tokio::test]
async fn test_provider_with_api_key() {
    let provider = NanoGPTProvider::with_api_key("test-key").await;
    assert!(provider.is_ok());
}
