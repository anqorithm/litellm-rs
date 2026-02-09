//! Mistral AI Provider (Refactored)
//!
//! Mistral AI model integration using the base infrastructure.
//! This implementation eliminates the need for common_utils.rs.

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use tracing::debug;

// Use base infrastructure instead of common_utils
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

// Re-export submodules (these remain as-is for now)
pub mod chat;
pub mod embedding;

// Static capabilities
const MISTRAL_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
    ProviderCapability::Embeddings,
];

/// Mistral provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistralConfig {
    /// API key for authentication
    pub api_key: String,
    /// API base URL (defaults to <https://api.mistral.ai/v1>)
    pub api_base: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
}

impl Default for MistralConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: "https://api.mistral.ai/v1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

impl ProviderConfig for MistralConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("Mistral API key is required".to_string());
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

/// Mistral error type (simplified using ProviderError)
pub type MistralError = ProviderError;

/// Mistral error mapper
pub struct MistralErrorMapper;

impl ErrorMapper<MistralError> for MistralErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> MistralError {
        HttpErrorMapper::map_status_code("mistral", status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> MistralError {
        HttpErrorMapper::parse_json_error("mistral", error_response)
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> MistralError {
        ProviderError::network("mistral", error.to_string())
    }

    fn map_parsing_error(&self, error: &dyn std::error::Error) -> MistralError {
        ProviderError::response_parsing("mistral", error.to_string())
    }

    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> MistralError {
        ProviderError::timeout(
            "mistral",
            format!("Request timed out after {:?}", timeout_duration),
        )
    }
}

/// Mistral provider implementation (refactored)
#[derive(Debug, Clone)]
pub struct MistralProvider {
    config: MistralConfig,
    base_client: BaseHttpClient,
    models: Vec<ModelInfo>,
}

impl MistralProvider {
    /// Create a new Mistral provider instance
    pub async fn new(config: MistralConfig) -> Result<Self, MistralError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("mistral", e))?;

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

        // Define supported models with pricing
        let models = vec![
            ModelInfo {
                id: "mistral-tiny".to_string(),
                name: "Mistral Tiny".to_string(),
                provider: "mistral".to_string(),
                max_context_length: 32000,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0002),
                output_cost_per_1k_tokens: Some(0.0006),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "mistral-small".to_string(),
                name: "Mistral Small".to_string(),
                provider: "mistral".to_string(),
                max_context_length: 32000,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.001),
                output_cost_per_1k_tokens: Some(0.003),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "mistral-medium".to_string(),
                name: "Mistral Medium".to_string(),
                provider: "mistral".to_string(),
                max_context_length: 32000,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0027),
                output_cost_per_1k_tokens: Some(0.0081),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "mistral-large".to_string(),
                name: "Mistral Large".to_string(),
                provider: "mistral".to_string(),
                max_context_length: 32000,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.008),
                output_cost_per_1k_tokens: Some(0.024),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "mistral-embed".to_string(),
                name: "Mistral Embed".to_string(),
                provider: "mistral".to_string(),
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

    /// Check if model is an embedding model
    fn is_embedding_model(&self, model: &str) -> bool {
        model.contains("embed")
    }
}

#[async_trait]
impl LLMProvider for MistralProvider {
    type Config = MistralConfig;
    type Error = MistralError;
    type ErrorMapper = MistralErrorMapper;

    fn name(&self) -> &'static str {
        "mistral"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        MISTRAL_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[
            "temperature",
            "top_p",
            "max_tokens",
            "stream",
            "stop",
            "random_seed",
            "tools",
            "tool_choice",
            "response_format",
        ]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // Mistral uses OpenAI-compatible parameters, so mostly pass-through
        let mut mapped = HashMap::new();

        for (key, value) in params {
            match key.as_str() {
                // Mistral uses 'random_seed' instead of 'seed'
                "seed" => mapped.insert("random_seed".to_string(), value),
                // Direct pass-through for standard parameters
                "temperature" | "top_p" | "max_tokens" | "stream" | "stop" | "tools"
                | "tool_choice" | "response_format" => mapped.insert(key, value),
                // Skip unsupported parameters
                _ => None,
            };
        }

        Ok(mapped)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        // Use the OpenAI transformer from base_provider
        let mut body = OpenAIRequestTransformer::transform_chat_request(&request);

        // Mistral-specific adjustments
        if let Some(seed) = body.get("seed") {
            let seed_value = seed.clone();
            let body_obj = body.as_object_mut().ok_or_else(|| {
                ProviderError::response_parsing(
                    "mistral",
                    "Transformed request body is not a JSON object".to_string(),
                )
            })?;
            body_obj.remove("seed");
            body["random_seed"] = seed_value;
        }

        Ok(body)
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        // Parse Mistral response (OpenAI-compatible format)
        serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing("mistral", e.to_string()))
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        MistralErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("Mistral chat request: model={}", request.model);

        // Check if it's an embedding model
        if self.is_embedding_model(&request.model) {
            return Err(ProviderError::invalid_request(
                "mistral",
                "Use embeddings endpoint for embedding models".to_string(),
            ));
        }

        // Transform request
        let body = self.transform_request(request, context).await?;

        // Direct HTTP call using BaseHttpClient
        let url = UrlBuilder::new(&self.config.api_base)
            .with_path("/chat/completions")
            .build();

        let headers = HeaderBuilder::new()
            .with_bearer_token(&self.config.api_key)
            .with_content_type("application/json")
            .build_reqwest()
            .map_err(|e| ProviderError::invalid_request("mistral", e.to_string()))?;

        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("mistral", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::api_error("mistral", status, body));
        }

        response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("mistral", e.to_string()))
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("Mistral streaming chat request: model={}", request.model);

        // Transform request
        let mut body = self.transform_request(request, context).await?;
        body["stream"] = serde_json::json!(true);

        // Execute streaming request directly
        let url = UrlBuilder::new(&self.config.api_base)
            .with_path("/chat/completions")
            .build();

        let headers = HeaderBuilder::new()
            .with_bearer_token(&self.config.api_key)
            .with_content_type("application/json")
            .build_reqwest()
            .map_err(|e| ProviderError::invalid_request("mistral", e.to_string()))?;

        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("mistral", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::api_error("mistral", status, body));
        }

        // Parse SSE stream using shared infrastructure
        use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEParser};
        use futures::StreamExt;

        let transformer = OpenAICompatibleTransformer::new("mistral");
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
                    Err(e) => Some(Err(ProviderError::network("mistral", e.to_string()))),
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
        debug!("Mistral embedding request: model={}", request.model);

        let body = serde_json::json!({
            "model": request.model,
            "input": request.input,
            "encoding_format": request.encoding_format,
        });

        // Direct HTTP call for embeddings
        let url = UrlBuilder::new(&self.config.api_base)
            .with_path("/embeddings")
            .build();

        let headers = HeaderBuilder::new()
            .with_bearer_token(&self.config.api_key)
            .with_content_type("application/json")
            .build_reqwest()
            .map_err(|e| ProviderError::invalid_request("mistral", e.to_string()))?;

        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("mistral", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::api_error("mistral", status, body));
        }

        response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("mistral", e.to_string()))
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
                        debug!("Mistral health check failed: status={}", response.status());
                        HealthStatus::Unhealthy
                    }
                    Err(e) => {
                        debug!("Mistral health check error: {}", e);
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
            .ok_or_else(|| ProviderError::model_not_found("mistral", model.to_string()))?;

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
