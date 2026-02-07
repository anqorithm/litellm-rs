//! Maritalk Provider Implementation

crate::define_openai_compatible_provider!(
    provider: super::PROVIDER_NAME,
    struct_name: MaritalkProvider,
    config: super::config::MaritalkConfig,
    error_mapper: super::error_mapper::MaritalkErrorMapper,
    model_info: super::model_info::get_supported_models,
    default_base_url: super::DEFAULT_BASE_URL,
    auth_header: "Authorization",
    auth_prefix: "Key ",
    supported_params: ["temperature", "max_tokens", "top_p", "stream", "stop"],
);

#[cfg(test)]
mod tests {
    use super::super::config::MaritalkConfig;
    use super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::ProviderCapability;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = MaritalkConfig::new("test-key");
        let provider = MaritalkProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_capabilities() {
        let config = MaritalkConfig::new("test-key");
        let provider = MaritalkProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ChatCompletion));
    }

    #[test]
    fn test_provider_models() {
        let config = MaritalkConfig::new("test-key");
        let provider = MaritalkProvider::new(config).unwrap();

        let models = provider.models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "sabia-2-medium"));
        assert!(models.iter().any(|m| m.id == "sabia-2-small"));
    }

    #[test]
    fn test_build_headers() {
        let config = MaritalkConfig::new("test-api-key");
        let provider = MaritalkProvider::new(config).unwrap();

        let headers = provider.build_headers();
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Key test-api-key".to_string())
        );
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[tokio::test]
    async fn test_calculate_cost() {
        let config = MaritalkConfig::new("test-key");
        let provider = MaritalkProvider::new(config).unwrap();

        // Test sabia-2-medium cost calculation
        let cost = provider.calculate_cost("sabia-2-medium", 1000, 1000).await;
        assert!(cost.is_ok());

        // Expected: (0.00002 * 1000/1000) + (0.00004 * 1000/1000) = 0.00006
        let cost_value = cost.unwrap();
        assert!((cost_value - 0.00006).abs() < 0.000001);
    }

    #[tokio::test]
    async fn test_calculate_cost_unknown_model() {
        let config = MaritalkConfig::new("test-key");
        let provider = MaritalkProvider::new(config).unwrap();

        let cost = provider.calculate_cost("unknown-model", 1000, 1000).await;
        assert!(cost.is_err());
    }
}
