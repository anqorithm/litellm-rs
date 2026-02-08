//! Tests for LLM client

#[cfg(test)]
mod tests {
    use super::super::llm_client::LLMClient;
    use crate::sdk::config::{ConfigBuilder, ProviderType};
    use crate::sdk::types::{ChatOptions, ChatRequest};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_llm_client_creation() {
        let config = ConfigBuilder::new()
            .add_provider(crate::sdk::config::ProviderConfig {
                id: "test".to_string(),
                provider_type: ProviderType::OpenAI,
                name: "Test Provider".to_string(),
                api_key: "test-key".to_string(),
                base_url: None,
                models: vec!["gpt-3.5-turbo".to_string()],
                enabled: true,
                weight: 1.0,
                rate_limit_rpm: Some(1000),
                rate_limit_tpm: Some(10000),
                settings: HashMap::new(),
            })
            .build();

        let client = LLMClient::new(config).unwrap();
        assert_eq!(client.list_providers().len(), 1);
    }

    #[tokio::test]
    async fn test_provider_selection() {
        let config = ConfigBuilder::new()
            .add_provider(crate::sdk::config::ProviderConfig {
                id: "anthropic".to_string(),
                provider_type: ProviderType::Anthropic,
                name: "Anthropic".to_string(),
                api_key: "test-key".to_string(),
                base_url: None,
                models: vec!["claude-3-sonnet-20240229".to_string()],
                enabled: true,
                weight: 1.0,
                rate_limit_rpm: Some(1000),
                rate_limit_tpm: Some(10000),
                settings: HashMap::new(),
            })
            .build();

        let client = LLMClient::new(config).unwrap();

        let request = ChatRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![],
            options: ChatOptions::default(),
        };

        let provider = client.select_provider(&request).await.unwrap();
        assert_eq!(provider.id, "anthropic");
    }
}
