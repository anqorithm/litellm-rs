//! Minimax AI Provider
//!
//! Minimax provides an OpenAI-compatible API with support for their MiniMax-M2 series models.
//! - International: <https://api.minimax.io/v1>
//! - China: <https://api.minimaxi.com/v1>

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use tracing::debug;

use crate::core::providers::base_provider::{
    BaseHttpClient, BaseProviderConfig, CostCalculator, HeaderBuilder, HttpErrorMapper,
    OpenAIRequestTransformer, UrlBuilder,
};
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
    model::ModelInfo,
    model::ProviderCapability,
    responses::{ChatChunk, ChatResponse, EmbeddingResponse},
};

// Re-export submodules
pub mod chat;

// Static capabilities
const MINIMAX_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::FunctionCalling,
];

/// Minimax provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimaxConfig {
    /// API key for authentication
    pub api_key: String,
    /// API base URL (defaults to <https://api.minimax.io/v1>)
    pub api_base: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
}

impl Default for MinimaxConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: "https://api.minimax.io/v1".to_string(),
            timeout_seconds: 60,
            max_retries: 3,
        }
    }
}

impl ProviderConfig for MinimaxConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("Minimax API key is required".to_string());
        }
        if self.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }
        if self.max_retries > 10 {
            return Err("Max retries should not exceed 10".to_string());
        }
        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        Some(&self.api_key)
    }

    fn api_base(&self) -> Option<&str> {
        Some(&self.api_base)
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

/// Minimax error type (simplified using ProviderError)
pub type MinimaxError = ProviderError;

/// Minimax error mapper
pub struct MinimaxErrorMapper;

impl ErrorMapper<MinimaxError> for MinimaxErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> MinimaxError {
        HttpErrorMapper::map_status_code("minimax", status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> MinimaxError {
        HttpErrorMapper::parse_json_error("minimax", error_response)
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> MinimaxError {
        ProviderError::network("minimax", error.to_string())
    }

    fn map_parsing_error(&self, error: &dyn std::error::Error) -> MinimaxError {
        ProviderError::response_parsing("minimax", error.to_string())
    }

    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> MinimaxError {
        ProviderError::timeout(
            "minimax",
            format!("Request timed out after {:?}", timeout_duration),
        )
    }
}

/// Minimax provider implementation
#[derive(Debug, Clone)]
pub struct MinimaxProvider {
    config: MinimaxConfig,
    base_client: BaseHttpClient,
    models: Vec<ModelInfo>,
}

impl MinimaxProvider {
    /// Create a new Minimax provider instance
    pub async fn new(config: MinimaxConfig) -> Result<Self, MinimaxError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("minimax", e))?;

        // Create base HTTP client using our infrastructure
        let base_config = BaseProviderConfig {
            api_key: Some(config.api_key.clone()),
            api_base: Some(config.api_base.clone()),
            timeout: Some(config.timeout_seconds),
            max_retries: Some(config.max_retries),
            headers: None,
            organization: None,
            api_version: None,
        };

        let base_client = BaseHttpClient::new(base_config)?;

        // Define supported models with pricing (USD per 1k tokens)
        let models = vec![
            ModelInfo {
                id: "MiniMax-M2.1".to_string(),
                name: "MiniMax M2.1".to_string(),
                provider: "minimax".to_string(),
                max_context_length: 1000000,
                max_output_length: Some(16384),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.001),
                output_cost_per_1k_tokens: Some(0.004),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "MiniMax-M2.1-lightning".to_string(),
                name: "MiniMax M2.1 Lightning".to_string(),
                provider: "minimax".to_string(),
                max_context_length: 1000000,
                max_output_length: Some(16384),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.0005),
                output_cost_per_1k_tokens: Some(0.002),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "MiniMax-M2".to_string(),
                name: "MiniMax M2".to_string(),
                provider: "minimax".to_string(),
                max_context_length: 256000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0008),
                output_cost_per_1k_tokens: Some(0.003),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
        ];

        Ok(Self {
            config,
            base_client,
            models,
        })
    }

    /// Build the complete URL for the chat completions endpoint
    fn build_chat_url(&self) -> String {
        let base = &self.config.api_base;
        if base.ends_with("/chat/completions") {
            base.clone()
        } else if base.ends_with("/v1") {
            format!("{}/chat/completions", base)
        } else if base.ends_with('/') {
            format!("{}v1/chat/completions", base)
        } else {
            format!("{}/v1/chat/completions", base)
        }
    }
}

#[async_trait]
impl LLMProvider for MinimaxProvider {
    type Config = MinimaxConfig;
    type Error = MinimaxError;
    type ErrorMapper = MinimaxErrorMapper;

    fn name(&self) -> &'static str {
        "minimax"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        MINIMAX_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("Minimax chat request: model={}", request.model);

        // Transform request
        let body = self.transform_request(request, context).await?;

        // Build URL based on config
        let url = self.build_chat_url();

        let headers = HeaderBuilder::new()
            .with_bearer_token(&self.config.api_key)
            .with_content_type("application/json")
            .build_reqwest()
            .map_err(|e| ProviderError::invalid_request("minimax", e.to_string()))?;

        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("minimax", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::api_error("minimax", status, body));
        }

        response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("minimax", e.to_string()))
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("Minimax streaming chat request: model={}", request.model);

        // Transform request
        let mut body = self.transform_request(request, context).await?;
        body["stream"] = serde_json::json!(true);

        // Build URL based on config
        let url = self.build_chat_url();

        let headers = HeaderBuilder::new()
            .with_bearer_token(&self.config.api_key)
            .with_content_type("application/json")
            .build_reqwest()
            .map_err(|e| ProviderError::invalid_request("minimax", e.to_string()))?;

        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("minimax", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::api_error("minimax", status, body));
        }

        // Parse SSE stream using shared infrastructure
        use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEParser};
        use futures::StreamExt;

        let transformer = OpenAICompatibleTransformer::new("minimax");
        let parser = UnifiedSSEParser::new(transformer);

        // Convert response bytes to stream of ChatChunks
        let byte_stream = response.bytes_stream();
        let stream = byte_stream
            .scan((parser, Vec::new()), |(parser, buffer), bytes_result| {
                futures::future::ready(match bytes_result {
                    Ok(bytes) => match parser.process_bytes(&bytes) {
                        Ok(chunks) => {
                            *buffer = chunks;
                            Some(Ok(buffer.clone()))
                        }
                        Err(e) => Some(Err(e)),
                    },
                    Err(e) => Some(Err(ProviderError::network("minimax", e.to_string()))),
                })
            })
            .map(|result| match result {
                Ok(chunks) => chunks.into_iter().map(Ok).collect::<Vec<_>>(),
                Err(e) => vec![Err(e)],
            })
            .flat_map(futures::stream::iter);

        Ok(Box::pin(stream))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        Err(ProviderError::not_supported("minimax", "embeddings"))
    }

    async fn health_check(&self) -> HealthStatus {
        // Try a simple models endpoint request
        let url = UrlBuilder::new(&self.config.api_base)
            .with_path("/models")
            .build();

        let headers = HeaderBuilder::new()
            .with_bearer_token(&self.config.api_key)
            .build_reqwest();

        match headers {
            Ok(headers) => {
                match self
                    .base_client
                    .inner()
                    .get(&url)
                    .headers(headers)
                    .send()
                    .await
                {
                    Ok(response) if response.status().is_success() => HealthStatus::Healthy,
                    Ok(response) => {
                        debug!("Minimax health check failed: status={}", response.status());
                        HealthStatus::Unhealthy
                    }
                    Err(e) => {
                        debug!("Minimax health check error: {}", e);
                        HealthStatus::Unhealthy
                    }
                }
            }
            Err(_) => HealthStatus::Unhealthy,
        }
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[
            "temperature",
            "top_p",
            "max_tokens",
            "stream",
            "stop",
            "presence_penalty",
            "frequency_penalty",
            "n",
            "user",
            "tools",
            "tool_choice",
            "reasoning_split", // Minimax-specific parameter
        ]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // Minimax is OpenAI-compatible, pass-through most parameters
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        // Use the OpenAI transformer from base_provider
        Ok(OpenAIRequestTransformer::transform_chat_request(&request))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        // Parse Minimax response (OpenAI-compatible format)
        serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing("minimax", e.to_string()))
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        MinimaxErrorMapper
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Find model pricing
        let model_info = self
            .models
            .iter()
            .find(|m| m.id == model)
            .ok_or_else(|| ProviderError::model_not_found("minimax", model.to_string()))?;

        let input_cost_per_1k = model_info.input_cost_per_1k_tokens.unwrap_or(0.0);
        let output_cost_per_1k = model_info.output_cost_per_1k_tokens.unwrap_or(0.0);

        Ok(CostCalculator::calculate(
            input_tokens,
            output_tokens,
            input_cost_per_1k,
            output_cost_per_1k,
        ))
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
