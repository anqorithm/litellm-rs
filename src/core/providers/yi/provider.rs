//! Yi Provider Implementation

crate::define_openai_compatible_provider!(
    provider: super::PROVIDER_NAME,
    struct_name: YiProvider,
    config: super::config::YiConfig,
    error_mapper: super::error_mapper::YiErrorMapper,
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
    use super::super::config::YiConfig;
    use super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::model::ProviderCapability;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = YiConfig::new("yi").with_api_key("test-key");
        let provider = YiProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_capabilities() {
        let config = YiConfig::new("yi").with_api_key("test-key");
        let provider = YiProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ChatCompletion));
    }

    #[test]
    fn test_provider_models() {
        let config = YiConfig::new("yi").with_api_key("test-key");
        let provider = YiProvider::new(config).unwrap();

        let models = provider.models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "yi-large"));
        assert!(models.iter().any(|m| m.id == "yi-vision"));
    }
}
