//! Main vLLM Provider Implementation
//!
//! Implements the LLMProvider trait for vLLM's high-throughput inference engine.
//! vLLM provides an OpenAI-compatible API for serving various open-source models.

use async_trait::async_trait;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

use super::config::VLLMConfig;
use super::error::VLLMError;
use super::model_info::{VLLMModelInfo, get_or_create_model_info};
use crate::core::providers::base::{GlobalPoolManager, HttpMethod, header};
use crate::core::traits::{
    provider::ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
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

/// Static capabilities for vLLM provider
const VLLM_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
];

/// vLLM provider implementation
#[derive(Debug, Clone)]
pub struct VLLMProvider {
    config: VLLMConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl VLLMProvider {
    /// Create a new vLLM provider instance
    pub async fn new(config: VLLMConfig) -> Result<Self, VLLMError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| VLLMError::configuration("vllm", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            VLLMError::configuration("vllm", format!("Failed to create pool manager: {}", e))
        })?);

        // Build model list - vLLM serves a specific model
        let served_model = config.model.as_ref().map(|m| get_or_create_model_info(m));

        let models = if let Some(ref model_info) = served_model {
            vec![model_info_to_gateway_model(model_info)]
        } else {
            // If no model specified, return empty - will be populated on first request
            vec![]
        };

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    /// Create provider with API base URL only
    pub async fn with_api_base(api_base: impl Into<String>) -> Result<Self, VLLMError> {
        let config = VLLMConfig::new(api_base);
        Self::new(config).await
    }

    /// Create provider with API base and optional API key
    pub async fn with_credentials(
        api_base: impl Into<String>,
        api_key: Option<String>,
    ) -> Result<Self, VLLMError> {
        let config = VLLMConfig::with_credentials(api_base, api_key);
        Self::new(config).await
    }

    /// Get the API base URL
    fn get_api_base(&self) -> Result<String, VLLMError> {
        self.config
            .get_api_base()
            .ok_or_else(|| VLLMError::configuration("vllm", "API base URL is required"))
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, VLLMError> {
        let api_base = self.get_api_base()?;
        let url = format!("{}{}", api_base, endpoint);

        let mut headers = Vec::with_capacity(2);
        if let Some(api_key) = &self.config.get_api_key() {
            headers.push(header("x-api-key", api_key.clone()));
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));

        debug!("vLLM request to {}", url);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| VLLMError::network("vllm", e.to_string()))?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| VLLMError::network("vllm", e.to_string()))?;

        if !status.is_success() {
            let body_str = String::from_utf8_lossy(&response_bytes);
            return Err(match status.as_u16() {
                400 => VLLMError::invalid_request("vllm", body_str.to_string()),
                401 | 403 => {
                    VLLMError::authentication("vllm", "Invalid API key or authentication failed")
                }
                404 => VLLMError::model_not_found("vllm", body_str.to_string()),
                429 => VLLMError::rate_limit("vllm", None),
                500 => VLLMError::api_error("vllm", 500, "Internal server error"),
                502 | 503 => VLLMError::api_error("vllm", status.as_u16(), "Service unavailable"),
                _ => VLLMError::api_error("vllm", status.as_u16(), body_str.to_string()),
            });
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            VLLMError::api_error("vllm", 500, format!("Failed to parse response: {}", e))
        })
    }

    /// Fetch available models from vLLM server
    pub async fn list_available_models(&self) -> Result<Vec<String>, VLLMError> {
        let api_base = self.get_api_base()?;
        let url = format!("{}/models", api_base);

        let mut headers = Vec::new();
        if let Some(api_key) = &self.config.get_api_key() {
            headers.push(header("x-api-key", api_key.clone()));
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::GET, headers, None::<serde_json::Value>)
            .await
            .map_err(|e| VLLMError::network("vllm", e.to_string()))?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| VLLMError::network("vllm", e.to_string()))?;

        let json: serde_json::Value = serde_json::from_slice(&response_bytes).map_err(|e| {
            VLLMError::api_error(
                "vllm",
                500,
                format!("Failed to parse models response: {}", e),
            )
        })?;

        let models = json["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["id"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }

    /// Process batch completions for multiple message sets
    pub async fn batch_completions(
        &self,
        model: &str,
        messages_batch: Vec<Vec<crate::core::types::chat::ChatMessage>>,
        optional_params: Option<BatchParams>,
    ) -> Result<Vec<ChatResponse>, VLLMError> {
        let params = optional_params.unwrap_or_default();

        // Process each message set
        let mut results = Vec::with_capacity(messages_batch.len());

        for messages in messages_batch {
            let request = ChatRequest {
                model: model.to_string(),
                messages,
                temperature: params.temperature,
                max_tokens: params.max_tokens,
                top_p: params.top_p,
                stop: params.stop.clone(),
                ..Default::default()
            };

            let context = RequestContext::default();
            let response = self.chat_completion(request, context).await?;
            results.push(response);
        }

        Ok(results)
    }
}

/// Parameters for batch processing
#[derive(Debug, Clone, Default)]
pub struct BatchParams {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
}

/// Convert VLLMModelInfo to gateway ModelInfo
fn model_info_to_gateway_model(info: &VLLMModelInfo) -> ModelInfo {
    let mut capabilities = vec![
        ProviderCapability::ChatCompletion,
        ProviderCapability::ChatCompletionStream,
    ];
    if info.supports_tools {
        capabilities.push(ProviderCapability::ToolCalling);
    }

    ModelInfo {
        id: info.model_id.clone(),
        name: info.display_name.clone(),
        provider: "vllm".to_string(),
        max_context_length: info.max_context_length,
        max_output_length: Some(info.max_output_length),
        supports_streaming: true,
        supports_tools: info.supports_tools,
        supports_multimodal: info.supports_multimodal,
        input_cost_per_1k_tokens: None, // vLLM is self-hosted, no API costs
        output_cost_per_1k_tokens: None, // vLLM is self-hosted, no API costs
        currency: "USD".to_string(),
        capabilities,
        created_at: None,
        updated_at: None,
        metadata: HashMap::new(),
    }
}

#[async_trait]
impl LLMProvider for VLLMProvider {
    type Config = VLLMConfig;
    type Error = VLLMError;
    type ErrorMapper = crate::core::traits::error_mapper::DefaultErrorMapper;

    fn name(&self) -> &'static str {
        "vllm"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        VLLM_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        // vLLM supports most OpenAI parameters
        &[
            "messages",
            "model",
            "temperature",
            "top_p",
            "max_tokens",
            "max_completion_tokens",
            "stream",
            "stop",
            "frequency_penalty",
            "presence_penalty",
            "n",
            "response_format",
            "seed",
            "tools",
            "tool_choice",
            "user",
            "logprobs",
            "top_logprobs",
            "echo",
            "best_of",
            "logit_bias",
            // vLLM specific parameters
            "use_beam_search",
            "top_k",
            "min_p",
            "repetition_penalty",
            "length_penalty",
            "early_stopping",
            "ignore_eos",
            "min_tokens",
            "skip_special_tokens",
            "spaces_between_special_tokens",
        ]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<HashMap<String, serde_json::Value>, Self::Error> {
        // vLLM uses OpenAI-compatible parameters, no mapping needed
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<serde_json::Value, Self::Error> {
        serde_json::to_value(&request)
            .map_err(|e| VLLMError::invalid_request("vllm", e.to_string()))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        serde_json::from_slice(raw_response).map_err(|e| {
            VLLMError::api_error("vllm", 500, format!("Failed to parse response: {}", e))
        })
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        crate::core::traits::error_mapper::DefaultErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("vLLM chat request: model={}", request.model);

        let request_json = serde_json::to_value(&request)
            .map_err(|e| VLLMError::invalid_request("vllm", e.to_string()))?;

        let response = self
            .execute_request("/chat/completions", request_json)
            .await?;

        serde_json::from_value(response).map_err(|e| {
            VLLMError::api_error("vllm", 500, format!("Failed to parse chat response: {}", e))
        })
    }

    async fn chat_completion_stream(
        &self,
        mut request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("vLLM streaming request: model={}", request.model);

        // Ensure streaming is enabled
        request.stream = true;

        // Get API configuration
        let api_base = self.get_api_base()?;
        let url = format!("{}/chat/completions", api_base);

        // Build request
        let client = reqwest::Client::new();
        let mut req = client.post(&url).json(&request);

        // Add authentication headers
        if let Some(api_key) = &self.config.get_api_key() {
            req = req.header("x-api-key", api_key);
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        req = req.header("Content-Type", "application/json");

        let response = req
            .send()
            .await
            .map_err(|e| VLLMError::network("vllm", e.to_string()))?;

        // Check status
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.ok();
            return Err(match status {
                400 => VLLMError::invalid_request(
                    "vllm",
                    body.unwrap_or_else(|| "Bad request".to_string()),
                ),
                401 => VLLMError::authentication("vllm", "Invalid API key"),
                429 => VLLMError::rate_limit("vllm", None),
                503 => VLLMError::api_error("vllm", 503, "vLLM server unavailable"),
                _ => VLLMError::api_error(
                    "vllm",
                    status,
                    format!("Stream request failed: {}", status),
                ),
            });
        }

        // Create SSE stream
        let stream = super::streaming::VLLMStream::new(response.bytes_stream());
        Ok(Box::pin(stream))
    }

    async fn embeddings(
        &self,
        request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        debug!("vLLM embeddings request: model={}", request.model);

        let request_json = serde_json::to_value(&request)
            .map_err(|e| VLLMError::invalid_request("vllm", e.to_string()))?;

        let response = self.execute_request("/embeddings", request_json).await?;

        serde_json::from_value(response).map_err(|e| {
            VLLMError::api_error(
                "vllm",
                500,
                format!("Failed to parse embeddings response: {}", e),
            )
        })
    }

    async fn health_check(&self) -> HealthStatus {
        // Try to get models list as health check
        match self.list_available_models().await {
            Ok(_) => HealthStatus::Healthy,
            Err(_) => HealthStatus::Unhealthy,
        }
    }

    async fn calculate_cost(
        &self,
        _model: &str,
        _input_tokens: u32,
        _output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // vLLM is self-hosted, so there are no API costs
        // Infrastructure costs would need to be calculated separately
        Ok(0.0)
    }
}
