//! Provider registry

use crate::core::providers::Provider;
use crate::sdk::{config::ClientConfig, errors::*};
use std::collections::HashMap;
use std::sync::Arc;

/// Provider registry
#[derive(Debug)]
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
    default_provider: Option<String>,
}

impl ProviderRegistry {
    /// Create new provider registry
    pub async fn new(config: &ClientConfig) -> Result<Self> {
        let mut registry = Self {
            providers: HashMap::new(),
            default_provider: config.default_provider.clone(),
        };

        // Register all configured providers
        for provider_config in &config.providers {
            if provider_config.enabled {
                let gateway_config = convert_to_gateway_config(provider_config)?;

                // Create provider instance (needs async handling, simplified for now)
                let provider = crate::core::providers::create_provider(gateway_config)
                    .await
                    .map_err(|e| SDKError::ProviderError(e.to_string()))?;

                registry
                    .providers
                    .insert(provider_config.id.clone(), provider);
            }
        }

        // If no default provider is set, use the first one
        if registry.default_provider.is_none() && !registry.providers.is_empty() {
            registry.default_provider = registry.providers.keys().next().cloned();
        }

        Ok(registry)
    }

    /// Get provider
    pub fn get_provider(&self, provider_id: Option<&str>) -> Result<Arc<dyn Provider>> {
        let id = match provider_id {
            Some(id) => id,
            None => self
                .default_provider
                .as_ref()
                .ok_or(SDKError::NoDefaultProvider)?,
        };

        self.providers
            .get(id)
            .cloned()
            .ok_or_else(|| SDKError::ProviderNotFound(id.to_string()))
    }

    /// List all providers
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

/// Convert SDK config to gateway config
fn convert_to_gateway_config(
    config: &crate::sdk::config::SdkProviderConfig,
) -> Result<crate::config::models::provider::ProviderConfig> {
    Ok(crate::config::models::provider::ProviderConfig {
        name: config.id.clone(),
        provider_type: match &config.provider_type {
            crate::sdk::config::ProviderType::OpenAI => "openai".to_string(),
            crate::sdk::config::ProviderType::Anthropic => "anthropic".to_string(),
            crate::sdk::config::ProviderType::Azure => "azure".to_string(),
            crate::sdk::config::ProviderType::Google => "google".to_string(),
            crate::sdk::config::ProviderType::Cohere => "cohere".to_string(),
            crate::sdk::config::ProviderType::HuggingFace => "huggingface".to_string(),
            crate::sdk::config::ProviderType::Ollama => "ollama".to_string(),
            crate::sdk::config::ProviderType::AwsBedrock => "aws_bedrock".to_string(),
            crate::sdk::config::ProviderType::GoogleVertex => "google_vertex".to_string(),
            crate::sdk::config::ProviderType::Mistral => "mistral".to_string(),
            crate::sdk::config::ProviderType::Custom(name) => name.clone(),
        },
        api_key: config.api_key.clone(),
        base_url: config.base_url.clone(),
        models: config.models.clone(),
        timeout: 30,    // Default value
        max_retries: 3, // Default value
        organization: None,
        api_version: None,
        project: None,
        weight: config.weight,
        rpm: config.rate_limit_rpm.unwrap_or(1000),
        tpm: config.rate_limit_tpm.unwrap_or(50000),
        enabled: config.enabled,
        max_concurrent_requests: 10, // Default value
        retry: crate::config::models::provider::RetryConfig::default(),
        health_check: crate::config::models::provider::ProviderHealthCheckConfig::default(),
        settings: HashMap::new(),
        tags: Vec::new(),
    })
}
