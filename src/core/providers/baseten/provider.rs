//! Main Baseten Provider Implementation
//!
//! Implements the LLMProvider trait for Baseten's serverless ML inference.

use std::collections::HashMap;
use std::sync::Arc;

use super::config::BasetenConfig;
use super::error::BasetenError;
use crate::core::providers::base::{GlobalPoolManager, HttpMethod, header};
use crate::core::traits::{
    provider::ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{model::ModelInfo, model::ProviderCapability};

/// Static capabilities for Baseten provider
const BASETEN_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
];

/// Supported OpenAI parameters for Baseten
const SUPPORTED_OPENAI_PARAMS: &[&str] = &[
    "max_tokens",
    "max_completion_tokens",
    "response_format",
    "seed",
    "stop",
    "stream",
    "temperature",
    "top_p",
    "tool_choice",
    "tools",
    "user",
    "presence_penalty",
    "frequency_penalty",
    "stream_options",
];

/// Baseten provider implementation
#[derive(Debug, Clone)]
pub struct BasetenProvider {
    config: BasetenConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl BasetenProvider {
    /// Create a new Baseten provider instance
    pub async fn new(config: BasetenConfig) -> Result<Self, BasetenError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| BasetenError::configuration("baseten", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            BasetenError::configuration("baseten", format!("Failed to create pool manager: {}", e))
        })?);

        // Build default model list - Baseten supports custom deployments
        // so we provide a minimal default list
        let models = vec![ModelInfo {
            id: "baseten-custom".to_string(),
            name: "Baseten Custom Model".to_string(),
            provider: "baseten".to_string(),
            max_context_length: 128000,
            max_output_length: Some(4096),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,  // Depends on deployment
            output_cost_per_1k_tokens: None, // Depends on deployment
            currency: "USD".to_string(),
            capabilities: vec![
                ProviderCapability::ChatCompletion,
                ProviderCapability::ChatCompletionStream,
                ProviderCapability::ToolCalling,
            ],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        }];

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    /// Create provider with API key only
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, BasetenError> {
        let config = BasetenConfig {
            api_key: Some(api_key.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Get the appropriate API base for a model
    fn get_api_base_for_request(&self, model: &str) -> String {
        // Check if custom api_base is set
        if let Some(custom_base) = &self.config.api_base {
            return custom_base.clone();
        }

        // Otherwise use model-based API base selection
        BasetenConfig::get_api_base_for_model(model)
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, BasetenError> {
        let mut headers = Vec::with_capacity(2);
        if let Some(api_key) = &self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));

        let response = self
            .pool_manager
            .execute_request(url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| BasetenError::network("baseten", e.to_string()))?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| BasetenError::network("baseten", e.to_string()))?;

        serde_json::from_slice(&response_bytes).map_err(|e| {
            BasetenError::api_error("baseten", 500, format!("Failed to parse response: {}", e))
        })
    }
}
