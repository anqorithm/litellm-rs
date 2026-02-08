//! Aleph Alpha Provider Implementation

crate::define_openai_compatible_provider!(
    provider: super::PROVIDER_NAME,
    struct_name: AlephAlphaProvider,
    config: super::config::AlephAlphaConfig,
    error_mapper: super::error_mapper::AlephAlphaErrorMapper,
    model_info: super::model_info::get_supported_models,
    default_base_url: super::DEFAULT_BASE_URL,
    auth_header: "Authorization",
    auth_prefix: "Bearer ",
    supported_params: [
        "temperature",
        "max_tokens",
        "top_p",
        "stream",
        "stop",
        "frequency_penalty",
        "presence_penalty",
    ],
);

#[cfg(test)]
mod tests {
    use super::super::config::AlephAlphaConfig;
    use super::super::model_info;
    use super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::model::ProviderCapability;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = AlephAlphaConfig::new("test-key");
        let provider = AlephAlphaProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_capabilities() {
        let config = AlephAlphaConfig::new("test-key");
        let provider = AlephAlphaProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ChatCompletion));
    }

    #[test]
    fn test_provider_models() {
        let config = AlephAlphaConfig::new("test-key");
        let provider = AlephAlphaProvider::new(config).unwrap();

        let models = provider.models();
        assert!(!models.is_empty());
        assert_eq!(models.len(), 3);

        let model_ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
        assert!(model_ids.contains(&"luminous-supreme"));
        assert!(model_ids.contains(&"luminous-extended"));
        assert!(model_ids.contains(&"luminous-base"));
    }

    #[test]
    fn test_model_support() {
        assert!(model_info::is_model_supported("luminous-supreme"));
        assert!(model_info::is_model_supported("luminous-extended"));
        assert!(model_info::is_model_supported("luminous-base"));
        assert!(!model_info::is_model_supported("unknown-model"));
    }
}
