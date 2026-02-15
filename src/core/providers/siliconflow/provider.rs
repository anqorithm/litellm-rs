//! SiliconFlow Provider Implementation

crate::define_openai_compatible_provider!(
    provider: super::PROVIDER_NAME,
    struct_name: SiliconFlowProvider,
    config: super::config::SiliconFlowConfig,
    error_mapper: super::error_mapper::SiliconFlowErrorMapper,
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
    use super::super::config::SiliconFlowConfig;
    use super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::model::ProviderCapability;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = SiliconFlowConfig::new("siliconflow").with_api_key("test-key");
        let provider = SiliconFlowProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_capabilities() {
        let config = SiliconFlowConfig::new("siliconflow").with_api_key("test-key");
        let provider = SiliconFlowProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ChatCompletion));
    }

    #[test]
    fn test_provider_models() {
        let config = SiliconFlowConfig::new("siliconflow").with_api_key("test-key");
        let provider = SiliconFlowProvider::new(config).unwrap();

        let models = provider.models();
        assert!(!models.is_empty());
        assert_eq!(models.len(), 3);

        // Verify specific models
        assert!(models.iter().any(|m| m.id == "deepseek-ai/DeepSeek-V2.5"));
        assert!(models.iter().any(|m| m.id == "Qwen/Qwen2.5-72B-Instruct"));
        assert!(
            models
                .iter()
                .any(|m| m.id == "Pro/Qwen/Qwen2.5-Coder-32B-Instruct")
        );
    }
}
