//! Main Clarifai Provider Implementation
//!
//! Implements the LLMProvider trait for Clarifai's AI platform.

use std::collections::HashMap;
use std::sync::Arc;

use super::config::ClarifaiConfig;
use super::error::ClarifaiError;
use crate::core::providers::base::{GlobalPoolManager, HttpMethod, header};
use crate::core::traits::{
    provider::ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{model::ModelInfo, model::ProviderCapability};

/// Static capabilities for Clarifai provider
const CLARIFAI_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
];

/// Supported OpenAI parameters for Clarifai
const SUPPORTED_OPENAI_PARAMS: &[&str] = &[
    "max_tokens",
    "max_completion_tokens",
    "response_format",
    "stream",
    "temperature",
    "top_p",
    "tool_choice",
    "tools",
    "presence_penalty",
    "frequency_penalty",
    "stream_options",
];

/// Clarifai provider implementation
#[derive(Debug, Clone)]
pub struct ClarifaiProvider {
    config: ClarifaiConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl ClarifaiProvider {
    /// Create a new Clarifai provider instance
    pub async fn new(config: ClarifaiConfig) -> Result<Self, ClarifaiError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ClarifaiError::configuration("clarifai", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ClarifaiError::configuration(
                "clarifai",
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        // Build default model list - Clarifai hosts various models
        let models = vec![ModelInfo {
            id: "clarifai-custom".to_string(),
            name: "Clarifai Custom Model".to_string(),
            provider: "clarifai".to_string(),
            max_context_length: 128000,
            max_output_length: Some(4096),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
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
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ClarifaiError> {
        let config = ClarifaiConfig {
            api_key: Some(api_key.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Transform model name to Clarifai URL format if needed
    fn transform_model(&self, model: &str) -> String {
        // If model is in user.app.model format, convert to URL
        if let Some(url) = ClarifaiConfig::get_model_url(model) {
            url
        } else {
            // Otherwise use as-is
            model.to_string()
        }
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, ClarifaiError> {
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
            .map_err(|e| ClarifaiError::network("clarifai", e.to_string()))?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ClarifaiError::network("clarifai", e.to_string()))?;

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ClarifaiError::api_error("clarifai", 500, format!("Failed to parse response: {}", e))
        })
    }
}
