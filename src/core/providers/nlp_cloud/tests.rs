//! Tests for NLP Cloud provider

#[cfg(test)]
mod unit_tests {
    use super::super::*;

    #[tokio::test]
    async fn test_nlp_cloud_provider_creation() {
        let config = NlpCloudConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let result = NlpCloudProvider::new(config).await;
        assert!(result.is_ok());
    }
}
