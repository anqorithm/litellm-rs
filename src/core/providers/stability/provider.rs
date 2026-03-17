//! Stability AI Provider Implementation
//!
//! Main provider implementation for Stability AI image generation.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::providers::base::GlobalPoolManager;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    provider::ProviderConfig, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    image::ImageGenerationRequest,
    model::ModelInfo,
    responses::{ImageData, ImageGenerationResponse},
};

use super::{StabilityConfig, get_stability_registry};

/// Stability AI image generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityImageRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_preset: Option<String>,
}

/// Stability AI image generation response
#[derive(Debug, Clone, Deserialize)]
pub struct StabilityImageResponse {
    pub image: Option<String>,
    pub finish_reason: Option<String>,
    pub seed: Option<u64>,
    #[serde(default)]
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct StabilityProvider {
    config: StabilityConfig,
    supported_models: Vec<ModelInfo>,
}

impl StabilityProvider {
    /// Create a new Stability AI provider
    pub fn new(config: StabilityConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("stability", e))?;

        let _pool_manager = Arc::new(
            GlobalPoolManager::new()
                .map_err(|e| ProviderError::configuration("stability", e.to_string()))?,
        );
        let supported_models = get_stability_registry().models().to_vec();

        Ok(Self {
            config,
            supported_models,
        })
    }

    /// Create provider from environment variables
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = StabilityConfig::from_env();
        Self::new(config)
    }

    /// Create provider with API key
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = StabilityConfig::with_api_key(api_key);
        Self::new(config)
    }

    /// Transform OpenAI-style image request to Stability request
    fn transform_image_request(&self, request: &ImageGenerationRequest) -> StabilityImageRequest {
        let registry = get_stability_registry();

        // Map size to aspect ratio if provided
        let aspect_ratio = request
            .size
            .as_ref()
            .and_then(|size| registry.size_to_aspect_ratio(size).map(|s| s.to_string()));

        StabilityImageRequest {
            prompt: request.prompt.clone(),
            negative_prompt: None,
            aspect_ratio,
            seed: None,
            output_format: Some("png".to_string()),
            model: request.model.clone(),
            mode: None,
            strength: None,
            style_preset: None,
        }
    }

    /// Transform Stability response to OpenAI-compatible response
    fn transform_image_response(
        &self,
        response: StabilityImageResponse,
    ) -> Result<ImageGenerationResponse, ProviderError> {
        // Check for errors
        if let Some(errors) = &response.errors
            && !errors.is_empty()
        {
            return Err(ProviderError::api_error(
                "stability",
                400,
                errors.join(", "),
            ));
        }

        // Check finish reason
        if let Some(ref reason) = response.finish_reason
            && reason == "CONTENT_FILTERED"
        {
            return Err(ProviderError::content_filtered(
                "stability",
                "Content was filtered by Stability AI safety systems",
                None,
                Some(false),
            ));
        }

        let mut data = Vec::new();

        if let Some(image_b64) = response.image {
            data.push(ImageData {
                url: None,
                b64_json: Some(image_b64),
                revised_prompt: None,
            });
        }

        Ok(ImageGenerationResponse {
            created: chrono::Utc::now().timestamp() as u64,
            data,
        })
    }

    /// Get the API endpoint for a model
    fn get_endpoint(&self, model: Option<&str>) -> String {
        let registry = get_stability_registry();
        let model_name = model.unwrap_or("sd3");
        format!(
            "{}{}",
            self.config
                .base
                .api_base
                .as_deref()
                .unwrap_or("https://api.stability.ai"),
            registry.get_endpoint(model_name)
        )
    }
}
