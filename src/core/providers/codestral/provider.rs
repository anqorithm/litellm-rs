//! Codestral Provider Implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

use super::config::CodestralConfig;
use super::error::CodestralError;
use super::model_info::{get_available_models, get_model_info};
use crate::ProviderError;
use crate::core::providers::base::{GlobalPoolManager, HeaderPair, HttpMethod, header};
use crate::core::traits::{
    provider::ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{model::ModelInfo, model::ProviderCapability};

const CODESTRAL_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
];

/// Fill-in-the-middle request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FimRequest {
    pub model: String,
    pub prompt: String,
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

/// Fill-in-the-middle response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FimResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<FimChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FimChoice {
    pub index: i32,
    pub text: String,
    pub finish_reason: Option<String>,
}

/// Codestral provider implementation
#[derive(Debug, Clone)]
pub struct CodestralProvider {
    config: CodestralConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl CodestralProvider {
    pub async fn new(config: CodestralConfig) -> Result<Self, CodestralError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("codestral", e))?;

        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration(
                "codestral",
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        let models = get_available_models()
            .iter()
            .filter_map(|id| get_model_info(id))
            .map(|info| ModelInfo {
                id: info.model_id.to_string(),
                name: info.display_name.to_string(),
                provider: "codestral".to_string(),
                max_context_length: info.max_context_length,
                max_output_length: Some(info.max_output_length),
                supports_streaming: true,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(info.input_cost_per_million / 1000.0),
                output_cost_per_1k_tokens: Some(info.output_cost_per_million / 1000.0),
                currency: "USD".to_string(),
                capabilities: vec![
                    ProviderCapability::ChatCompletion,
                    ProviderCapability::ChatCompletionStream,
                ],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            })
            .collect();

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, CodestralError> {
        let config = CodestralConfig {
            api_key: Some(api_key.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    fn build_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::new();
        if let Some(api_key) = self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));
        headers
    }

    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, CodestralError> {
        let url = format!("{}{}", self.config.get_api_base(), endpoint);
        let headers = self.build_headers();

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| ProviderError::network("codestral", e.to_string()))?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("codestral", e.to_string()))?;

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::api_error("codestral", 500, format!("Failed to parse response: {}", e))
        })
    }

    /// Fill-in-the-middle completion (code infilling)
    pub async fn fim_completion(&self, request: FimRequest) -> Result<FimResponse, CodestralError> {
        debug!("Codestral FIM request: model={}", request.model);

        let request_json = serde_json::to_value(&request)
            .map_err(|e| ProviderError::invalid_request("codestral", e.to_string()))?;

        let response = self
            .execute_request("/fim/completions", request_json)
            .await?;

        serde_json::from_value(response).map_err(|e| {
            ProviderError::api_error(
                "codestral",
                500,
                format!("Failed to parse FIM response: {}", e),
            )
        })
    }
}
