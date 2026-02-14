//! Tests for Predibase provider

#[cfg(test)]
use super::*;

#[tokio::test]
async fn test_predibase_provider_creation() {
    let config = PredibaseConfig {
        api_key: Some("test-key".to_string()),
        ..Default::default()
    };

    let result = PredibaseProvider::new(config).await;
    assert!(result.is_ok());
}
