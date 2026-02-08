//! Anyscale Provider Implementation

crate::define_openai_compatible_provider!(
    provider: super::PROVIDER_NAME,
    struct_name: AnyscaleProvider,
    config: super::config::AnyscaleConfig,
    error_mapper: super::error_mapper::AnyscaleErrorMapper,
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
    use super::super::config::AnyscaleConfig;
    use super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::model::ProviderCapability;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = AnyscaleConfig::new("test-key");
        let provider = AnyscaleProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_capabilities() {
        let config = AnyscaleConfig::new("test-key");
        let provider = AnyscaleProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ChatCompletion));
    }

    #[test]
    fn test_provider_models() {
        let config = AnyscaleConfig::new("test-key");
        let provider = AnyscaleProvider::new(config).unwrap();

        let models = provider.models();
        assert!(!models.is_empty());
        assert_eq!(models.len(), 3);

        // Verify specific models
        assert!(
            models
                .iter()
                .any(|m| m.id == "meta-llama/Llama-2-70b-chat-hf")
        );
        assert!(
            models
                .iter()
                .any(|m| m.id == "mistralai/Mistral-7B-Instruct-v0.1")
        );
        assert!(
            models
                .iter()
                .any(|m| m.id == "codellama/CodeLlama-34b-Instruct-hf")
        );
    }
}
