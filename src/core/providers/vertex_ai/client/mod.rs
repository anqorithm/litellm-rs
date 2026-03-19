//! Vertex AI Client Implementation
//!
//! Split into submodules:
//! - `operations` - Chat completion, embedding, and token counting
//! - `llm_provider` - LLMProvider trait implementation

mod llm_provider;
mod operations;
#[cfg(test)]
mod tests;

use reqwest::{Client, Response};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

use crate::core::providers::base::HttpErrorMapper;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::utils::net::http::create_custom_client;

use super::{
    VertexAIProviderConfig,
    auth::VertexAuth,
    error::VertexAIError,
    models::VertexAIModel,
    transformers::{GeminiTransformer, PartnerModelTransformer},
};
use crate::ProviderError;

// Cost calculation removed - integrated in provider implementation

/// VertexAI-specific error mapper implementation
#[derive(Debug)]
pub struct VertexAIErrorMapper;

impl ErrorMapper<VertexAIError> for VertexAIErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> VertexAIError {
        match status_code {
            400 => ProviderError::response_parsing(
                "vertex_ai",
                format!("Bad request: {}", response_body),
            ),
            401 => ProviderError::authentication("vertex_ai", "Invalid credentials or API key"),
            403 => ProviderError::configuration(
                "vertex_ai",
                "Access forbidden: insufficient permissions",
            ),
            404 => ProviderError::model_not_found("vertex_ai", "Model not found"),
            429 => ProviderError::rate_limit("vertex_ai", None),
            500 => ProviderError::network("vertex_ai", "Internal server error"),
            502 => ProviderError::network("vertex_ai", "Bad gateway"),
            503 => ProviderError::network("vertex_ai", "Service unavailable"),
            _ => ProviderError::network(
                "vertex_ai",
                format!("HTTP error {}: {}", status_code, response_body),
            ),
        }
    }

    fn map_json_error(&self, error_response: &Value) -> VertexAIError {
        if let Some(error) = error_response.get("error") {
            let error_code = error.get("code").and_then(|c| c.as_u64()).unwrap_or(0);
            let error_message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            let status = error
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("UNKNOWN");

            match status {
                "INVALID_ARGUMENT" => ProviderError::response_parsing("vertex_ai", error_message),
                "UNAUTHENTICATED" => {
                    ProviderError::authentication("vertex_ai", "Authentication failed")
                }
                "PERMISSION_DENIED" => {
                    ProviderError::configuration("vertex_ai", "Permission denied")
                }
                "NOT_FOUND" => ProviderError::model_not_found("vertex_ai", error_message),
                "RESOURCE_EXHAUSTED" => ProviderError::rate_limit("vertex_ai", None),
                "INTERNAL" | "UNAVAILABLE" => ProviderError::network("vertex_ai", error_message),
                _ => ProviderError::network(
                    "vertex_ai",
                    format!("API Error ({}): {}", error_code, error_message),
                ),
            }
        } else {
            ProviderError::response_parsing("vertex_ai", "Unknown error response format")
        }
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> VertexAIError {
        ProviderError::network("vertex_ai", format!("Network error: {}", error))
    }
}

/// Vertex AI Provider implementation
#[derive(Debug, Clone)]
pub struct VertexAIProvider {
    pub(crate) config: VertexAIProviderConfig,
    auth: Arc<VertexAuth>,
    http_client: Client,
    // Cost calculation integrated internally
    pub(crate) gemini_transformer: GeminiTransformer,
    pub(crate) partner_transformer: PartnerModelTransformer,
}

impl VertexAIProvider {
    /// Create a new Vertex AI provider
    pub async fn new(config: VertexAIProviderConfig) -> Result<Self, VertexAIError> {
        let auth = Arc::new(VertexAuth::new(config.credentials.clone()));

        let http_client = create_custom_client(Duration::from_secs(config.timeout_seconds))
            .map_err(|e| ProviderError::configuration("vertex_ai", e.to_string()))?;

        Ok(Self {
            config,
            auth,
            http_client,
            gemini_transformer: GeminiTransformer::new(),
            partner_transformer: PartnerModelTransformer::new(),
        })
    }

    /// Build the API URL for a given model and endpoint
    pub(crate) fn build_url(&self, model: &VertexAIModel, endpoint: &str, stream: bool) -> String {
        let model_id = model.model_id();
        let location = &self.config.location;
        let project_id = &self.config.project_id;
        let api_version = &self.config.api_version;

        // Handle custom API base
        if let Some(ref api_base) = self.config.api_base {
            return format!("{}/{}:{}", api_base, model_id, endpoint);
        }

        // Special handling for global models
        let use_global = location == "global" || model_id.contains("imagen");

        let base_url = if use_global {
            format!(
                "https://aiplatform.googleapis.com/{}/projects/{}/locations/global",
                api_version, project_id
            )
        } else {
            format!(
                "https://{}-aiplatform.googleapis.com/{}/projects/{}/locations/{}",
                location, api_version, project_id, location
            )
        };

        // Build full URL based on model type
        let url = if model.is_gemini() {
            format!(
                "{}/publishers/google/models/{}:{}",
                base_url, model_id, endpoint
            )
        } else if model.is_partner_model() {
            // Partner models have different URL structure
            let publisher = self.get_publisher_for_model(&model_id);
            format!(
                "{}/publishers/{}/models/{}:{}",
                base_url, publisher, model_id, endpoint
            )
        } else {
            // Custom models
            format!("{}/endpoints/{}:{}", base_url, model_id, endpoint)
        };

        // Add streaming parameter if needed
        if stream {
            format!("{}?alt=sse", url)
        } else {
            url
        }
    }

    /// Get publisher for partner models
    fn get_publisher_for_model(&self, model_id: &str) -> &str {
        if model_id.contains("claude") {
            "anthropic"
        } else if model_id.contains("llama") {
            "meta"
        } else if model_id.contains("jamba") {
            "ai21"
        } else {
            "google"
        }
    }

    /// Make an authenticated request
    pub(crate) async fn make_request(
        &self,
        url: &str,
        body: Value,
    ) -> Result<Response, VertexAIError> {
        let token = self
            .auth
            .get_access_token()
            .await
            .map_err(|e| ProviderError::authentication("vertex_ai", e.to_string()))?;

        debug!("Making request to Vertex AI: {}", url);

        let response = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("vertex_ai", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return Err(HttpErrorMapper::map_status_code(
                "vertex_ai",
                status.as_u16(),
                &error_text,
            ));
        }

        Ok(response)
    }
}
