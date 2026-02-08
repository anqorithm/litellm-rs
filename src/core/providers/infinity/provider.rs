//! Main Infinity Provider Implementation
//!
//! Implements the LLMProvider trait for Infinity embedding and reranking server.
//! Infinity is primarily used for embeddings and reranking, not chat completion.

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

use super::config::InfinityConfig;
use super::error::InfinityError;
use crate::core::providers::base::{GlobalPoolManager, HttpMethod, header};
use crate::core::traits::{
    ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    chat::ChatRequest,
    context::RequestContext,
    embedding::EmbeddingRequest,
    health::HealthStatus,
    model::ModelInfo,
    model::ProviderCapability,
    responses::{ChatChunk, ChatResponse, EmbeddingData, EmbeddingResponse, Usage},
};

/// Static capabilities for Infinity provider
const INFINITY_CAPABILITIES: &[ProviderCapability] = &[ProviderCapability::Embeddings];

/// Infinity embedding request
#[derive(Debug, Clone, Serialize)]
struct InfinityEmbeddingRequest {
    input: Vec<String>,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_dimension: Option<u32>,
}

/// Infinity embedding response
#[derive(Debug, Clone, Deserialize)]
struct InfinityEmbeddingResponse {
    model: Option<String>,
    data: Vec<InfinityEmbeddingData>,
    object: Option<String>,
    usage: Option<InfinityUsage>,
}

/// Infinity embedding data
#[derive(Debug, Clone, Deserialize)]
struct InfinityEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
    object: Option<String>,
}

/// Infinity usage
#[derive(Debug, Clone, Deserialize)]
pub struct InfinityUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

/// Infinity rerank request
#[derive(Debug, Clone, Serialize)]
pub struct InfinityRerankRequest {
    pub query: String,
    pub documents: Vec<String>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_documents: Option<bool>,
}

/// Infinity rerank result
#[derive(Debug, Clone, Deserialize)]
pub struct InfinityRerankResult {
    pub index: usize,
    pub relevance_score: f64,
    #[serde(default)]
    pub document: Option<String>,
}

/// Infinity rerank response
#[derive(Debug, Clone, Deserialize)]
pub struct InfinityRerankResponse {
    pub id: Option<String>,
    pub results: Vec<InfinityRerankResult>,
    pub usage: Option<InfinityUsage>,
}

/// Infinity provider implementation
#[derive(Debug, Clone)]
pub struct InfinityProvider {
    config: InfinityConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl InfinityProvider {
    /// Create a new Infinity provider instance
    pub async fn new(config: InfinityConfig) -> Result<Self, InfinityError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| InfinityError::configuration("infinity", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            InfinityError::configuration(
                "infinity",
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        // Infinity doesn't have a fixed model list - models are configured on the server
        // We create a placeholder model entry
        let models = vec![ModelInfo {
            id: "infinity-embedding".to_string(),
            name: "Infinity Embedding Model".to_string(),
            provider: "infinity".to_string(),
            max_context_length: 8192,
            max_output_length: None,
            supports_streaming: false,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![ProviderCapability::Embeddings],
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

    /// Create provider with API base URL
    pub async fn with_api_base(api_base: impl Into<String>) -> Result<Self, InfinityError> {
        let config = InfinityConfig {
            api_base: Some(api_base.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Execute an HTTP POST request
    async fn execute_post_request(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, InfinityError> {
        let mut headers = Vec::with_capacity(3);
        if let Some(api_key) = &self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("accept", "application/json".to_string()));
        headers.push(header("Content-Type", "application/json".to_string()));

        let response = self
            .pool_manager
            .execute_request(url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| InfinityError::network("infinity", e.to_string()))?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| InfinityError::network("infinity", e.to_string()))?;

        if !status.is_success() {
            let body_text = String::from_utf8_lossy(&response_bytes);
            return Err(InfinityError::api_error(
                "infinity",
                status.as_u16(),
                format!("HTTP {}: {}", status.as_u16(), body_text),
            ));
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            InfinityError::api_error("infinity", 500, format!("Failed to parse response: {}", e))
        })
    }

    /// Perform reranking
    pub async fn rerank(
        &self,
        request: InfinityRerankRequest,
    ) -> Result<InfinityRerankResponse, InfinityError> {
        let url = self
            .config
            .get_rerank_url()
            .ok_or_else(|| InfinityError::configuration("infinity", "API base not configured"))?;

        debug!("Infinity rerank request: model={}", request.model);

        let request_json = serde_json::to_value(&request)
            .map_err(|e| InfinityError::invalid_request("infinity", e.to_string()))?;

        let response = self.execute_post_request(&url, request_json).await?;

        serde_json::from_value(response).map_err(|e| {
            InfinityError::api_error(
                "infinity",
                500,
                format!("Failed to parse rerank response: {}", e),
            )
        })
    }
}

#[async_trait]
impl LLMProvider for InfinityProvider {
    type Config = InfinityConfig;
    type Error = InfinityError;
    type ErrorMapper = crate::core::traits::error_mapper::DefaultErrorMapper;

    fn name(&self) -> &'static str {
        "infinity"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        INFINITY_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &["encoding_format", "modality", "dimensions"]
    }

    async fn map_openai_params(
        &self,
        mut params: HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<HashMap<String, serde_json::Value>, Self::Error> {
        // Map OpenAI 'dimensions' to Infinity 'output_dimension'
        if let Some(dimensions) = params.remove("dimensions") {
            params.insert("output_dimension".to_string(), dimensions);
        }
        Ok(params)
    }

    async fn transform_request(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<serde_json::Value, Self::Error> {
        // Infinity doesn't support chat completion
        Err(InfinityError::not_supported("infinity", "chat completion"))
    }

    async fn transform_response(
        &self,
        _raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        // Infinity doesn't support chat completion
        Err(InfinityError::not_supported("infinity", "chat completion"))
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        crate::core::traits::error_mapper::DefaultErrorMapper
    }

    async fn chat_completion(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        Err(InfinityError::not_supported(
            "infinity",
            "Infinity is an embedding and reranking server. Chat completion is not supported. Use the embeddings() or rerank() methods instead.",
        ))
    }

    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        Err(InfinityError::not_supported(
            "infinity",
            "Infinity does not support chat completion or streaming.",
        ))
    }

    async fn embeddings(
        &self,
        request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        let url = self
            .config
            .get_embeddings_url()
            .ok_or_else(|| InfinityError::configuration("infinity", "API base not configured"))?;

        debug!("Infinity embeddings request: model={}", request.model);

        // Convert input to Vec<String>
        let input: Vec<String> = match request.input {
            crate::core::types::embedding::EmbeddingInput::Text(s) => vec![s],
            crate::core::types::embedding::EmbeddingInput::Array(v) => v,
        };

        // Build Infinity-specific request
        let infinity_request = InfinityEmbeddingRequest {
            input,
            model: request.model.clone(),
            encoding_format: request.encoding_format,
            modality: None,
            output_dimension: request.dimensions,
        };

        let request_json = serde_json::to_value(&infinity_request)
            .map_err(|e| InfinityError::invalid_request("infinity", e.to_string()))?;

        let response = self.execute_post_request(&url, request_json).await?;

        let infinity_response: InfinityEmbeddingResponse = serde_json::from_value(response)
            .map_err(|e| {
                InfinityError::api_error(
                    "infinity",
                    500,
                    format!("Failed to parse embeddings response: {}", e),
                )
            })?;

        // Convert to standard EmbeddingResponse
        let data: Vec<EmbeddingData> = infinity_response
            .data
            .into_iter()
            .map(|d| EmbeddingData {
                object: d.object.unwrap_or_else(|| "embedding".to_string()),
                embedding: d.embedding,
                index: d.index as u32,
            })
            .collect();

        let usage = infinity_response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: 0,
            total_tokens: u.total_tokens,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        });

        Ok(EmbeddingResponse {
            object: infinity_response
                .object
                .unwrap_or_else(|| "list".to_string()),
            data,
            model: infinity_response.model.unwrap_or(request.model),
            usage,
            embeddings: None,
        })
    }

    async fn health_check(&self) -> HealthStatus {
        // Simple health check - try to access the API base
        if let Some(api_base) = self.config.get_api_base() {
            let url = format!("{}/health", api_base.trim_end_matches('/'));
            let headers = Vec::new();

            match self
                .pool_manager
                .execute_request(&url, HttpMethod::GET, headers, None::<serde_json::Value>)
                .await
            {
                Ok(_) => HealthStatus::Healthy,
                Err(_) => HealthStatus::Unhealthy,
            }
        } else {
            HealthStatus::Unhealthy
        }
    }

    async fn calculate_cost(
        &self,
        _model: &str,
        _input_tokens: u32,
        _output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Infinity is typically self-hosted, so cost calculation doesn't apply
        Ok(0.0)
    }
}
