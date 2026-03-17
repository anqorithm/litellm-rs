//! Main Petals Provider Implementation

use std::collections::HashMap;
use std::sync::Arc;

use super::config::PetalsConfig;
use super::model_info::{get_available_models, get_model_info};
use crate::core::providers::base::{GlobalPoolManager, HttpErrorMapper, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    provider::ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{model::ModelInfo, model::ProviderCapability};

const PETALS_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
];

#[derive(Debug, Clone)]
pub struct PetalsProvider {
    config: PetalsConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl PetalsProvider {
    pub async fn new(config: PetalsConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("petals", e))?;

        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration("petals", format!("Failed to create pool manager: {}", e))
        })?);

        let models = get_available_models()
            .iter()
            .filter_map(|id| get_model_info(id))
            .map(|info| {
                let capabilities = vec![
                    ProviderCapability::ChatCompletion,
                    ProviderCapability::ChatCompletionStream,
                ];

                ModelInfo {
                    id: info.model_id.to_string(),
                    name: info.display_name.to_string(),
                    provider: "petals".to_string(),
                    max_context_length: info.max_context_length,
                    max_output_length: Some(info.max_output_length),
                    supports_streaming: true,
                    supports_tools: info.supports_tools,
                    supports_multimodal: info.supports_multimodal,
                    input_cost_per_1k_tokens: Some(info.input_cost_per_million / 1000.0),
                    output_cost_per_1k_tokens: Some(info.output_cost_per_million / 1000.0),
                    currency: "USD".to_string(),
                    capabilities,
                    created_at: None,
                    updated_at: None,
                    metadata: HashMap::new(),
                }
            })
            .collect();

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    pub async fn with_api_base(api_base: impl Into<String>) -> Result<Self, ProviderError> {
        let config = PetalsConfig {
            api_base: Some(api_base.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError> {
        let url = format!("{}{}", self.config.get_api_base(), endpoint);

        let mut headers = Vec::with_capacity(2);
        if let Some(api_key) = &self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| ProviderError::network("petals", e.to_string()))?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("petals", e.to_string()))?;

        if !status.is_success() {
            let error_body = String::from_utf8_lossy(&response_bytes);
            return Err(match status.as_u16() {
                400 => ProviderError::invalid_request("petals", error_body.to_string()),
                404 => ProviderError::model_not_found("petals", "Model not found"),
                429 => ProviderError::rate_limit("petals", None),
                503 => ProviderError::provider_unavailable("petals", "Service unavailable"),
                _ => HttpErrorMapper::map_status_code("petals", status.as_u16(), &error_body),
            });
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::api_error("petals", 500, format!("Failed to parse response: {}", e))
        })
    }
}
