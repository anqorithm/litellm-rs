//! Main Together AI Provider Implementation
//!
//! Implements the LLMProvider trait for Together AI's high-performance inference.
//! Together AI is OpenAI-compatible and supports chat completions, embeddings, and rerank.

use std::collections::HashMap;
use std::sync::Arc;

use super::config::TogetherConfig;
use super::error::TogetherError;
use super::model_info::{get_available_models, get_model_info, is_function_calling_model};
use super::rerank::{RerankRequest, RerankResponse};
use crate::core::providers::base::{GlobalPoolManager, HttpErrorMapper, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    provider::ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    chat::ChatRequest, message::MessageRole, model::ModelInfo, model::ProviderCapability,
};

/// Static capabilities for Together AI provider
const TOGETHER_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
    ProviderCapability::Embeddings,
];

/// Together AI provider implementation
#[derive(Debug, Clone)]
pub struct TogetherProvider {
    config: TogetherConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl TogetherProvider {
    /// Create a new Together AI provider instance
    pub async fn new(config: TogetherConfig) -> Result<Self, TogetherError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("together", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration(
                "together",
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        // Build model list from static configuration
        let models = get_available_models()
            .iter()
            .filter_map(|id| get_model_info(id))
            .filter(|info| !info.is_embedding && !info.is_rerank) // Only chat models
            .map(|info| {
                let mut capabilities = vec![
                    ProviderCapability::ChatCompletion,
                    ProviderCapability::ChatCompletionStream,
                ];
                if info.supports_tools {
                    capabilities.push(ProviderCapability::ToolCalling);
                }

                ModelInfo {
                    id: info.model_id.to_string(),
                    name: info.display_name.to_string(),
                    provider: "together".to_string(),
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

    /// Create provider with API key only
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, TogetherError> {
        let config = TogetherConfig::from_env().with_api_key(api_key);
        Self::new(config).await
    }

    /// Check if response format requires special handling
    pub(crate) fn should_handle_response_format(&self, request: &ChatRequest) -> bool {
        // Together AI supports response_format only for certain models with function calling
        if let Some(ref format) = request.response_format
            && format.format_type == "json_object"
        {
            return !is_function_calling_model(&request.model);
        }
        false
    }

    /// Transform messages for Together AI API
    fn transform_messages(&self, request: &mut ChatRequest) {
        // Remove null function_call from assistant messages
        for message in request.messages.iter_mut() {
            if message.role == MessageRole::Assistant {
                // Function call handling would go here if needed
            }
        }
    }

    /// Handle response_format - remove for models that don't support it
    fn handle_response_format(&self, request: &mut ChatRequest) {
        // Check if model supports function calling / response_format
        if let Some(ref format) = request.response_format
            && format.format_type == "text"
        {
            // Remove text format as it's the default
            request.response_format = None;
        }
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, TogetherError> {
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
            .map_err(|e| ProviderError::network("together", e.to_string()))?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("together", e.to_string()))?;

        if !status.is_success() {
            let error_body = String::from_utf8_lossy(&response_bytes);
            return Err(match status.as_u16() {
                400 => ProviderError::invalid_request("together", error_body.to_string()),
                401 => ProviderError::authentication("together", "Invalid API key"),
                404 => ProviderError::model_not_found("together", "Model not found"),
                429 => ProviderError::rate_limit("together", None),
                _ => HttpErrorMapper::map_status_code("together", status.as_u16(), &error_body),
            });
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::api_error("together", 500, format!("Failed to parse response: {}", e))
        })
    }

    /// Execute a rerank request
    pub async fn rerank(&self, request: RerankRequest) -> Result<RerankResponse, TogetherError> {
        let api_key = self.config.get_api_key().ok_or_else(|| {
            ProviderError::authentication("together", "API key is required".to_string())
        })?;

        let url = format!("{}/rerank", self.config.get_api_base());

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::network("together", e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                400 => ProviderError::invalid_request(
                    "together",
                    format!("Bad request: {}", error_body),
                ),
                401 => ProviderError::authentication("together", "Invalid API key"),
                429 => ProviderError::rate_limit("together", None),
                _ => HttpErrorMapper::map_status_code("together", status.as_u16(), &error_body),
            });
        }

        let response_text = response.text().await.map_err(|e| {
            ProviderError::api_error("together", 500, format!("Failed to read response: {}", e))
        })?;

        serde_json::from_str(&response_text).map_err(|e| {
            ProviderError::api_error("together", 500, format!("Failed to parse response: {}", e))
        })
    }
}
