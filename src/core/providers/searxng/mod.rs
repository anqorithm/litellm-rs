//! SearXNG Provider
//!
//! SearXNG meta search engine integration

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;

use crate::core::providers::base::{BaseConfig, BaseHttpClient, HttpErrorMapper};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    error_mapper::trait_def::ErrorMapper, provider::ProviderConfig,
    provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    chat::ChatRequest,
    context::RequestContext,
    embedding::EmbeddingRequest,
    health::HealthStatus,
    image::ImageGenerationRequest,
    model::ModelInfo,
    model::ProviderCapability,
    responses::{ChatChunk, ChatResponse, EmbeddingResponse, ImageGenerationResponse},
};

/// SearXNG configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearXNGConfig {
    /// API key for SearXNG (optional)
    pub api_key: Option<String>,
    /// API base URL (required, no default)
    pub api_base: Option<String>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Max retries
    pub max_retries: u32,
}

impl Default for SearXNGConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: None,
            timeout: 30,
            max_retries: 3,
        }
    }
}

impl SearXNGConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, SearXNGError> {
        let api_key = std::env::var("SEARXNG_API_KEY").ok();
        let api_base = std::env::var("SEARXNG_API_BASE").ok();

        Ok(Self {
            api_key,
            api_base,
            timeout: 30,
            max_retries: 3,
        })
    }

    /// Get effective API base URL
    pub fn get_effective_api_base(&self) -> Result<&str, SearXNGError> {
        self.api_base
            .as_deref()
            .ok_or_else(|| ProviderError::configuration("searxng", "API base URL is required"))
    }
}

/// SearXNG error type (alias to unified ProviderError)
pub type SearXNGError = ProviderError;

/// SearXNG provider
#[derive(Debug, Clone)]
pub struct SearXNGProvider {
    config: SearXNGConfig,
}

impl SearXNGProvider {
    /// Create new SearXNG provider
    pub fn new(config: SearXNGConfig) -> Result<Self, SearXNGError> {
        let base_config = BaseConfig {
            api_key: config.api_key.clone(),
            api_base: config.api_base.clone(),
            timeout: config.timeout,
            max_retries: config.max_retries,
            headers: HashMap::new(),
            organization: None,
            api_version: None,
        };

        let _base_client = BaseHttpClient::new(base_config)
            .map_err(|e| ProviderError::configuration("searxng", e.to_string()))?;

        Ok(Self { config })
    }
}

/// SearXNG error mapper
#[derive(Debug)]
pub struct SearXNGErrorMapper;

impl ErrorMapper<SearXNGError> for SearXNGErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> SearXNGError {
        HttpErrorMapper::map_status_code("searxng", status_code, response_body)
    }
}

#[async_trait]
impl LLMProvider for SearXNGProvider {
    fn name(&self) -> &'static str {
        "searxng"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        static CAPABILITIES: &[ProviderCapability] = &[];
        CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &[]
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[]
    }

    async fn map_openai_params(
        &self,
        params: std::collections::HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>, ProviderError> {
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<serde_json::Value, ProviderError> {
        use serde_json::json;
        Ok(json!({ "model": request.model }))
    }

    async fn transform_response(
        &self,
        _raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, ProviderError> {
        Err(ProviderError::not_supported("searxng", "Chat completion"))
    }

    fn get_error_mapper(&self) -> Box<dyn ErrorMapper<ProviderError>> {
        Box::new(SearXNGErrorMapper)
    }

    async fn calculate_cost(
        &self,
        _model: &str,
        _input_tokens: u32,
        _output_tokens: u32,
    ) -> Result<f64, ProviderError> {
        Ok(0.0)
    }

    fn supports_model(&self, _model: &str) -> bool {
        false
    }

    async fn health_check(&self) -> HealthStatus {
        if self.config.api_base.is_some() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }

    async fn chat_completion(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, ProviderError> {
        Err(ProviderError::not_supported("searxng", "Chat completion"))
    }

    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError>
    {
        Err(ProviderError::not_supported("searxng", "Streaming"))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, ProviderError> {
        Err(ProviderError::not_supported("searxng", "Embeddings"))
    }

    async fn image_generation(
        &self,
        _request: ImageGenerationRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, ProviderError> {
        Err(ProviderError::not_supported("searxng", "Image generation"))
    }
}

impl ProviderConfig for SearXNGConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_base.is_none() {
            return Err("SearXNG API base URL is required".to_string());
        }
        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        self.api_base.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}
