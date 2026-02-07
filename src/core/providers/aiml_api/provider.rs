//! AIML API Provider Implementation

crate::define_openai_compatible_provider!(
    provider: super::PROVIDER_NAME,
    struct_name: AimlProvider,
    config: super::config::AimlConfig,
    error_mapper: super::error_mapper::AimlErrorMapper,
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
    use super::super::config::AimlConfig;
    use super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::ProviderCapability;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = AimlConfig::new("test-key");
        let provider = AimlProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_capabilities() {
        let config = AimlConfig::new("test-key");
        let provider = AimlProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ChatCompletion));
    }

    #[test]
    fn test_provider_models() {
        let config = AimlConfig::new("test-key");
        let provider = AimlProvider::new(config).unwrap();

        let models = provider.models();
        assert!(!models.is_empty());
    }
}
