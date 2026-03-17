//! Replicate Provider Implementation
//!
//! Main provider implementation using the unified base infrastructure

use serde_json::Value;
use std::sync::Arc;

use crate::core::providers::base::{GlobalPoolManager, HeaderPair, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::ProviderConfig;
use crate::core::types::{
    context::RequestContext, image::ImageGenerationRequest, model::ModelInfo,
    responses::ImageGenerationResponse,
};

use super::{
    ReplicateClient, ReplicateConfig,
    prediction::{CreatePredictionRequest, PredictionResponse, PredictionStatus},
};

/// Replicate provider implementation
#[derive(Debug, Clone)]
pub struct ReplicateProvider {
    config: ReplicateConfig,
    pool_manager: Arc<GlobalPoolManager>,
    supported_models: Vec<ModelInfo>,
}

impl ReplicateProvider {
    /// Create a new Replicate provider
    pub fn new(config: ReplicateConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("replicate", e))?;

        let pool_manager = Arc::new(
            GlobalPoolManager::new()
                .map_err(|e| ProviderError::configuration("replicate", e.to_string()))?,
        );

        let supported_models = ReplicateClient::supported_models();

        Ok(Self {
            config,
            pool_manager,
            supported_models,
        })
    }

    /// Create provider with API token
    pub async fn with_api_token(api_token: impl Into<String>) -> Result<Self, ProviderError> {
        let config = ReplicateConfig::new(api_token);
        Self::new(config)
    }

    /// Create provider from environment
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = ReplicateConfig::from_env();
        Self::new(config)
    }

    /// Generate headers for Replicate API requests
    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(2);

        if let Some(api_key) = &self.config.base.api_key {
            headers.push(header("Authorization", format!("Token {}", api_key)));
        }

        headers.push(header("Content-Type", "application/json".to_string()));

        headers
    }

    /// Create a prediction and wait for completion
    async fn create_prediction_and_wait(
        &self,
        model: &str,
        input: Value,
        stream: bool,
    ) -> Result<PredictionResponse, ProviderError> {
        // Create prediction request
        let version_hash = ReplicateConfig::extract_version_hash(model);
        let prediction_request =
            ReplicateClient::create_prediction_request(input, version_hash, stream);

        // Submit prediction
        let prediction_url = self.config.get_prediction_url(model);
        let prediction = self
            .submit_prediction(&prediction_url, &prediction_request)
            .await?;

        // Get polling URL
        let polling_url = prediction
            .get_prediction_url()
            .ok_or_else(|| {
                ProviderError::replicate_response_parsing("No polling URL in prediction response")
            })?
            .to_string();

        // Poll until completion
        self.poll_prediction(&polling_url).await
    }

    /// Submit a prediction request
    async fn submit_prediction(
        &self,
        url: &str,
        request: &CreatePredictionRequest,
    ) -> Result<PredictionResponse, ProviderError> {
        let headers = self.get_request_headers();
        let body = serde_json::to_value(request)
            .map_err(|e| ProviderError::serialization("replicate", e.to_string()))?;

        let response = self
            .pool_manager
            .execute_request(url, HttpMethod::POST, headers, Some(body))
            .await?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("replicate", e.to_string()))?;

        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&response_bytes);
            return Err(ProviderError::replicate_api_error(
                status.as_u16(),
                error_text.to_string(),
            ));
        }

        serde_json::from_slice(&response_bytes)
            .map_err(|e| ProviderError::replicate_response_parsing(e.to_string()))
    }

    /// Poll a prediction until completion
    async fn poll_prediction(&self, url: &str) -> Result<PredictionResponse, ProviderError> {
        let headers = self.get_request_headers();
        let polling_delay = std::time::Duration::from_secs(self.config.polling_delay_seconds);

        for _ in 0..self.config.polling_retries {
            tokio::time::sleep(polling_delay).await;

            let response = self
                .pool_manager
                .execute_request(url, HttpMethod::GET, headers.clone(), None)
                .await?;

            let status = response.status();
            let response_bytes = response
                .bytes()
                .await
                .map_err(|e| ProviderError::network("replicate", e.to_string()))?;

            if !status.is_success() {
                // Temporary failure, continue polling
                continue;
            }

            let prediction: PredictionResponse = serde_json::from_slice(&response_bytes)
                .map_err(|e| ProviderError::replicate_response_parsing(e.to_string()))?;

            match prediction.status {
                PredictionStatus::Succeeded => return Ok(prediction),
                PredictionStatus::Failed => {
                    let error = prediction
                        .error
                        .clone()
                        .unwrap_or_else(|| "Prediction failed".to_string());
                    return Err(ProviderError::replicate_prediction_failed(error));
                }
                PredictionStatus::Canceled => {
                    return Err(ProviderError::replicate_prediction_canceled(
                        "Prediction was canceled",
                    ));
                }
                _ => {
                    // Still processing, continue polling
                }
            }
        }

        Err(ProviderError::replicate_prediction_timeout(
            "Maximum retries exceeded waiting for prediction",
        ))
    }

    /// Execute image generation
    async fn execute_image_generation(
        &self,
        request: ImageGenerationRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, ProviderError> {
        let model = request.model.as_deref().unwrap_or("stability-ai/sdxl");

        let input = ReplicateClient::transform_image_request(&request, model);
        let prediction = self.create_prediction_and_wait(model, input, false).await?;

        ReplicateClient::transform_prediction_to_image_response(&prediction)
    }
}
