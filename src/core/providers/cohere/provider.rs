//! Cohere Provider Implementation
//!
//! Main provider implementation integrating all Cohere capabilities:
//! - Chat completions (Command models)
//! - Embeddings (embed models)
//! - Reranking (rerank models)

use serde_json::Value;
use std::collections::HashMap;
use tracing::debug;

use super::config::CohereConfig;
use super::rerank::{CohereRerankHandler, RerankRequest, RerankResponse};
use crate::core::providers::base::{
    BaseConfig, BaseHttpClient, HttpErrorMapper, apply_headers, header, header_static,
};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    error_mapper::trait_def::ErrorMapper, provider::ProviderConfig,
    provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{model::ModelInfo, model::ProviderCapability};

// Static capabilities
const COHERE_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
    ProviderCapability::Embeddings,
];

/// Cohere error mapper
pub struct CohereErrorMapper;

impl ErrorMapper<ProviderError> for CohereErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        HttpErrorMapper::map_status_code("cohere", status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> ProviderError {
        HttpErrorMapper::parse_json_error("cohere", error_response)
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> ProviderError {
        ProviderError::network("cohere", error.to_string())
    }

    fn map_parsing_error(&self, error: &dyn std::error::Error) -> ProviderError {
        ProviderError::response_parsing("cohere", error.to_string())
    }

    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> ProviderError {
        ProviderError::timeout(
            "cohere",
            format!("Request timed out after {:?}", timeout_duration),
        )
    }
}

/// Cohere provider implementation
#[derive(Debug, Clone)]
pub struct CohereProvider {
    config: CohereConfig,
    client: BaseHttpClient,
    models: Vec<ModelInfo>,
}

impl CohereProvider {
    /// Create a new Cohere provider instance
    pub async fn new(config: CohereConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("cohere", e))?;

        let base_config = BaseConfig {
            api_key: Some(config.api_key.clone()),
            api_base: Some(config.api_base.clone()),
            timeout: config.timeout_seconds,
            max_retries: config.max_retries,
            headers: HashMap::new(),
            organization: None,
            api_version: None,
        };

        let client = BaseHttpClient::new(base_config)?;

        let models = Self::create_model_registry();

        Ok(Self {
            config,
            client,
            models,
        })
    }

    /// Create provider with API key
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = CohereConfig::new(api_key);
        Self::new(config).await
    }

    /// Create the model registry with all supported models
    fn create_model_registry() -> Vec<ModelInfo> {
        vec![
            // Command models (Chat)
            ModelInfo {
                id: "command-r-plus".to_string(),
                name: "Command R+".to_string(),
                provider: "cohere".to_string(),
                max_context_length: 128000,
                max_output_length: Some(4096),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.003),
                output_cost_per_1k_tokens: Some(0.015),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "command-r".to_string(),
                name: "Command R".to_string(),
                provider: "cohere".to_string(),
                max_context_length: 128000,
                max_output_length: Some(4096),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0005),
                output_cost_per_1k_tokens: Some(0.0015),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "command".to_string(),
                name: "Command".to_string(),
                provider: "cohere".to_string(),
                max_context_length: 4096,
                max_output_length: Some(4096),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.001),
                output_cost_per_1k_tokens: Some(0.002),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "command-light".to_string(),
                name: "Command Light".to_string(),
                provider: "cohere".to_string(),
                max_context_length: 4096,
                max_output_length: Some(4096),
                supports_streaming: true,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0003),
                output_cost_per_1k_tokens: Some(0.0006),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            // Embedding models
            ModelInfo {
                id: "embed-english-v3.0".to_string(),
                name: "Embed English v3.0".to_string(),
                provider: "cohere".to_string(),
                max_context_length: 512,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0001),
                output_cost_per_1k_tokens: Some(0.0),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "embed-multilingual-v3.0".to_string(),
                name: "Embed Multilingual v3.0".to_string(),
                provider: "cohere".to_string(),
                max_context_length: 512,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: true, // Supports images
                input_cost_per_1k_tokens: Some(0.0001),
                output_cost_per_1k_tokens: Some(0.0),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "embed-english-light-v3.0".to_string(),
                name: "Embed English Light v3.0".to_string(),
                provider: "cohere".to_string(),
                max_context_length: 512,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0001),
                output_cost_per_1k_tokens: Some(0.0),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "embed-multilingual-light-v3.0".to_string(),
                name: "Embed Multilingual Light v3.0".to_string(),
                provider: "cohere".to_string(),
                max_context_length: 512,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0001),
                output_cost_per_1k_tokens: Some(0.0),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            // Rerank models
            ModelInfo {
                id: "rerank-english-v3.0".to_string(),
                name: "Rerank English v3.0".to_string(),
                provider: "cohere".to_string(),
                max_context_length: 4096,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.002),
                output_cost_per_1k_tokens: Some(0.0),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "rerank-multilingual-v3.0".to_string(),
                name: "Rerank Multilingual v3.0".to_string(),
                provider: "cohere".to_string(),
                max_context_length: 4096,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.002),
                output_cost_per_1k_tokens: Some(0.0),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
        ]
    }

    /// Check if model is an embedding model
    fn is_embedding_model(&self, model: &str) -> bool {
        model.contains("embed")
    }

    /// Check if model is a rerank model
    fn is_rerank_model(&self, model: &str) -> bool {
        model.contains("rerank")
    }

    /// Get config reference
    pub fn config(&self) -> &CohereConfig {
        &self.config
    }

    /// Execute a rerank request
    pub async fn rerank(&self, request: RerankRequest) -> Result<RerankResponse, ProviderError> {
        debug!("Cohere rerank request: model={}", request.model);

        let body = CohereRerankHandler::transform_request(&request)?;

        let url = self.config.rerank_endpoint();

        let headers = vec![
            header("Authorization", format!("Bearer {}", self.config.api_key)),
            header_static("Content-Type", "application/json"),
        ];

        let response = apply_headers(self.client.inner().post(&url), headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("cohere", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(HttpErrorMapper::map_status_code("cohere", status, &body));
        }

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("cohere", e.to_string()))?;

        CohereRerankHandler::transform_response(response_json)
    }
}
