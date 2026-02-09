//! Sambanova AI Provider
//!
//! High-performance inference provider using Sambanova's custom AI chips.
//! This implementation is OpenAI-compatible with minimal transformation needed.
//!
//! Reference: <https://docs.sambanova.ai/cloud/api-reference/>

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
    provider::ProviderConfig, error_mapper::trait_def::ErrorMapper,
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

// Static capabilities for Sambanova
const SAMBANOVA_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
    ProviderCapability::Embeddings,
];

/// Sambanova provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SambanovaConfig {
    /// API key for authentication
    pub api_key: String,
    /// API base URL (defaults to <https://api.sambanova.ai/v1>)
    pub api_base: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
}

impl Default for SambanovaConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: "https://api.sambanova.ai/v1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

impl ProviderConfig for SambanovaConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("Sambanova API key is required".to_string());
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

/// Sambanova error type (using unified ProviderError)
pub type SambanovaError = ProviderError;

/// Sambanova error mapper
pub struct SambanovaErrorMapper;

impl ErrorMapper<SambanovaError> for SambanovaErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> SambanovaError {
        HttpErrorMapper::map_status_code("sambanova", status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> SambanovaError {
        HttpErrorMapper::parse_json_error("sambanova", error_response)
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> SambanovaError {
        ProviderError::network("sambanova", error.to_string())
    }

    fn map_parsing_error(&self, error: &dyn std::error::Error) -> SambanovaError {
        ProviderError::response_parsing("sambanova", error.to_string())
    }

    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> SambanovaError {
        ProviderError::timeout(
            "sambanova",
            format!("Request timed out after {:?}", timeout_duration),
        )
    }
}

/// Sambanova provider implementation
///
/// Sambanova uses custom AI chips (RDU) for high-performance inference.
/// The API is OpenAI-compatible, so minimal transformation is needed.
#[derive(Debug, Clone)]
pub struct SambanovaProvider {
    config: SambanovaConfig,
    base_client: BaseHttpClient,
    models: Vec<ModelInfo>,
}

impl SambanovaProvider {
    /// Create a new Sambanova provider instance
    pub async fn new(config: SambanovaConfig) -> Result<Self, SambanovaError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("sambanova", e))?;

        // Create base HTTP client
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

        // Define supported models with pricing
        // Note: Sambanova pricing varies - these are placeholder values
        let models = vec![
            ModelInfo {
                id: "Meta-Llama-3.1-8B-Instruct".to_string(),
                name: "Llama 3.1 8B Instruct".to_string(),
                provider: "sambanova".to_string(),
                max_context_length: 8192,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0001),
                output_cost_per_1k_tokens: Some(0.0002),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "Meta-Llama-3.1-70B-Instruct".to_string(),
                name: "Llama 3.1 70B Instruct".to_string(),
                provider: "sambanova".to_string(),
                max_context_length: 8192,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0005),
                output_cost_per_1k_tokens: Some(0.001),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "Meta-Llama-3.1-405B-Instruct".to_string(),
                name: "Llama 3.1 405B Instruct".to_string(),
                provider: "sambanova".to_string(),
                max_context_length: 8192,
                max_output_length: None,
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
                id: "sambanova-embed".to_string(),
                name: "Sambanova Embed".to_string(),
                provider: "sambanova".to_string(),
                max_context_length: 8192,
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
        ];

        Ok(Self {
            config,
            base_client,
            models,
        })
    }

    /// Create provider with just API key using default configuration
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, SambanovaError> {
        let config = SambanovaConfig {
            api_key: api_key.into(),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Check if model is an embedding model
    fn is_embedding_model(&self, model: &str) -> bool {
        model.contains("embed")
    }

    /// Check if model supports function calling
    fn supports_function_calling(&self, model: &str) -> bool {
        // Most Llama models on Sambanova support function calling
        model.contains("Instruct") || model.contains("Chat")
    }
}

#[async_trait]
impl LLMProvider for SambanovaProvider {
    type Config = SambanovaConfig;
    type Error = SambanovaError;
    type ErrorMapper = SambanovaErrorMapper;

    fn name(&self) -> &'static str {
        "sambanova"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        SAMBANOVA_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn get_supported_openai_params(&self, model: &str) -> &'static [&'static str] {
        // Base parameters supported by all Sambanova models
        let base_params: &[&str] = &[
            "max_completion_tokens",
            "max_tokens",
            "response_format",
            "stop",
            "stream",
            "stream_options",
            "temperature",
            "top_p",
            "top_k",
        ];

        // Add tool-related params if model supports function calling
        if self.supports_function_calling(model) {
            &[
                "max_completion_tokens",
                "max_tokens",
                "response_format",
                "stop",
                "stream",
                "stream_options",
                "temperature",
                "top_p",
                "top_k",
                "tools",
                "tool_choice",
                "parallel_tool_calls",
            ]
        } else {
            base_params
        }
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        let mut mapped = HashMap::new();

        for (key, value) in params {
            match key.as_str() {
                // Map max_completion_tokens to max_tokens for Sambanova
                "max_completion_tokens" => {
                    mapped.insert("max_tokens".to_string(), value);
                }
                // Direct pass-through for standard parameters
                "temperature"
                | "top_p"
                | "top_k"
                | "max_tokens"
                | "stream"
                | "stop"
                | "stream_options"
                | "tools"
                | "tool_choice"
                | "response_format"
                | "parallel_tool_calls" => {
                    mapped.insert(key, value);
                }
                // Skip unsupported parameters
                _ => {}
            }
        }

        Ok(mapped)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        // Use OpenAI transformer since Sambanova is OpenAI-compatible
        let body = OpenAIRequestTransformer::transform_chat_request(&request);
        Ok(body)
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        // Parse response (OpenAI-compatible format)
        serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing("sambanova", e.to_string()))
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        SambanovaErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("Sambanova chat request: model={}", request.model);

        // Check if it's an embedding model
        if self.is_embedding_model(&request.model) {
            return Err(ProviderError::invalid_request(
                "sambanova",
                "Use embeddings endpoint for embedding models".to_string(),
            ));
        }

        // Transform request
        let body = self.transform_request(request, context).await?;

        // Build URL and headers
        let url = UrlBuilder::new(&self.config.api_base)
            .with_path("/chat/completions")
            .build();

        let headers = HeaderBuilder::new()
            .with_bearer_token(&self.config.api_key)
            .with_content_type("application/json")
            .build_reqwest()
            .map_err(|e| ProviderError::invalid_request("sambanova", e.to_string()))?;

        // Execute request
        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("sambanova", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::api_error("sambanova", status, body));
        }

        response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("sambanova", e.to_string()))
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("Sambanova streaming chat request: model={}", request.model);

        // Transform request and enable streaming
        let mut body = self.transform_request(request, context).await?;
        body["stream"] = serde_json::json!(true);

        // Build URL and headers
        let url = UrlBuilder::new(&self.config.api_base)
            .with_path("/chat/completions")
            .build();

        let headers = HeaderBuilder::new()
            .with_bearer_token(&self.config.api_key)
            .with_content_type("application/json")
            .build_reqwest()
            .map_err(|e| ProviderError::invalid_request("sambanova", e.to_string()))?;

        // Execute request
        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("sambanova", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::api_error("sambanova", status, body));
        }

        // Parse SSE stream using shared infrastructure
        use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEParser};
        use futures::StreamExt;

        let transformer = OpenAICompatibleTransformer::new("sambanova");
        let parser = UnifiedSSEParser::new(transformer);

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
                    Err(e) => Some(Err(ProviderError::network("sambanova", e.to_string()))),
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
        request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        debug!("Sambanova embedding request: model={}", request.model);

        let body = serde_json::json!({
            "model": request.model,
            "input": request.input,
        });

        // Build URL and headers
        let url = UrlBuilder::new(&self.config.api_base)
            .with_path("/embeddings")
            .build();

        let headers = HeaderBuilder::new()
            .with_bearer_token(&self.config.api_key)
            .with_content_type("application/json")
            .build_reqwest()
            .map_err(|e| ProviderError::invalid_request("sambanova", e.to_string()))?;

        // Execute request
        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("sambanova", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::api_error("sambanova", status, body));
        }

        response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("sambanova", e.to_string()))
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
                        debug!(
                            "Sambanova health check failed: status={}",
                            response.status()
                        );
                        HealthStatus::Unhealthy
                    }
                    Err(e) => {
                        debug!("Sambanova health check error: {}", e);
                        HealthStatus::Unhealthy
                    }
                }
            }
            Err(_) => HealthStatus::Unhealthy,
        }
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
            .ok_or_else(|| ProviderError::model_not_found("sambanova", model.to_string()))?;

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
