//! Fal AI Provider Implementation
//!
//! Main provider implementation for Fal AI image generation

use serde_json::Value;
use std::sync::Arc;
use std::time::SystemTime;

use crate::core::providers::base::{GlobalPoolManager, HeaderPair, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    provider::ProviderConfig, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    model::ModelInfo,
    model::ProviderCapability,
    responses::{ImageData, ImageGenerationResponse},
};

use super::{FalAIConfig, FalAIModelRegistry};

/// Fal AI Provider for image generation
#[derive(Debug, Clone)]
pub struct FalAIProvider {
    config: FalAIConfig,
    pool_manager: Arc<GlobalPoolManager>,
    model_registry: FalAIModelRegistry,
    supported_models: Vec<ModelInfo>,
}

impl FalAIProvider {
    /// Create a new Fal AI provider with configuration
    pub fn new(config: FalAIConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("fal_ai", e))?;

        let pool_manager = Arc::new(
            GlobalPoolManager::new()
                .map_err(|e| ProviderError::configuration("fal_ai", e.to_string()))?,
        );

        let model_registry = FalAIModelRegistry::new();
        let supported_models = Self::build_model_info(&model_registry);

        Ok(Self {
            config,
            pool_manager,
            model_registry,
            supported_models,
        })
    }

    /// Create provider from environment variables
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = FalAIConfig::from_env();
        Self::new(config)
    }

    /// Create provider with API key
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = FalAIConfig::with_api_key(api_key);
        Self::new(config)
    }

    /// Build ModelInfo list from registry
    fn build_model_info(registry: &FalAIModelRegistry) -> Vec<ModelInfo> {
        registry
            .list_models()
            .iter()
            .map(|m| ModelInfo {
                id: m.id.clone(),
                name: m.name.clone(),
                provider: "fal_ai".to_string(),
                max_context_length: 0,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: true,
                input_cost_per_1k_tokens: None,
                output_cost_per_1k_tokens: None,
                currency: "USD".to_string(),
                capabilities: vec![ProviderCapability::ImageGeneration],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            })
            .collect()
    }

    /// Generate request headers for Fal AI API
    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(2);

        if let Some(api_key) = self.config.get_api_key() {
            headers.push(header("Authorization", format!("Key {}", api_key)));
        }

        headers.push(header("Content-Type", "application/json".to_string()));
        headers
    }

    /// Get the endpoint URL for a model
    fn get_model_endpoint(&self, model: &str) -> String {
        let base = self.config.get_api_base().trim_end_matches('/');
        format!("{}/{}", base, model)
    }

    /// Transform Fal AI response to ImageGenerationResponse
    fn transform_image_response(
        &self,
        response_data: Value,
    ) -> Result<ImageGenerationResponse, ProviderError> {
        let images = response_data
            .get("images")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ProviderError::response_parsing("fal_ai", "Missing 'images' field in response")
            })?;

        let data: Vec<ImageData> = images
            .iter()
            .filter_map(|img| {
                if let Some(url) = img.get("url").and_then(|v| v.as_str()) {
                    Some(ImageData {
                        url: Some(url.to_string()),
                        b64_json: img
                            .get("b64_json")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        revised_prompt: None,
                    })
                } else {
                    img.as_str().map(|url| ImageData {
                        url: Some(url.to_string()),
                        b64_json: None,
                        revised_prompt: None,
                    })
                }
            })
            .collect();

        let created = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(ImageGenerationResponse { created, data })
    }
}
