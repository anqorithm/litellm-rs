//! Tests for NLP Cloud provider

#[cfg(test)]
use super::*;

#[tokio::test]
async fn test_nlp_cloud_provider_creation() {
    let config = NlpCloudConfig {
        api_key: Some("test-key".to_string()),
        ..Default::default()
    };

    let result = NlpCloudProvider::new(config).await;
    assert!(result.is_ok());
}
