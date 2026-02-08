//! Main Hosted vLLM Provider Implementation
//!
//! Implements the LLMProvider trait for hosted vLLM inference servers.
//! vLLM provides an OpenAI-compatible API for serving various open-source models.

use async_trait::async_trait;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

use super::config::HostedVLLMConfig;
use super::models::{HostedVLLMModelInfo, get_or_create_model_info};
use crate::core::providers::base::{GlobalPoolManager, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
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
    responses::{ChatChunk, ChatResponse, EmbeddingResponse},
};

/// Provider name constant for error messages
const PROVIDER_NAME: &str = "hosted_vllm";

/// Static capabilities for hosted vLLM provider
const HOSTED_VLLM_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
];

/// Hosted vLLM provider implementation
#[derive(Debug, Clone)]
pub struct HostedVLLMProvider {
    config: HostedVLLMConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
    served_model: Option<HostedVLLMModelInfo>,
}

impl HostedVLLMProvider {
    /// Create a new hosted vLLM provider instance
    pub async fn new(config: HostedVLLMConfig) -> Result<Self, ProviderError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration(PROVIDER_NAME, e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration(
                PROVIDER_NAME,
                format!("Failed to create pool manager: {}", e),
            )
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
            served_model,
        })
    }

    /// Create provider with API base URL only
    pub async fn with_api_base(api_base: impl Into<String>) -> Result<Self, ProviderError> {
        let config = HostedVLLMConfig::new(api_base);
        Self::new(config).await
    }

    /// Create provider with API base and optional API key
    pub async fn with_credentials(
        api_base: impl Into<String>,
        api_key: Option<String>,
    ) -> Result<Self, ProviderError> {
        let config = HostedVLLMConfig::with_credentials(api_base, api_key);
        Self::new(config).await
    }

    /// Create provider from environment variables
    pub async fn from_env() -> Result<Self, ProviderError> {
        let config = HostedVLLMConfig::from_env();
        Self::new(config).await
    }

    /// Get the API base URL
    fn get_api_base(&self) -> Result<String, ProviderError> {
        self.config
            .get_api_base()
            .ok_or_else(|| ProviderError::configuration(PROVIDER_NAME, "API base URL is required"))
    }

    /// Build headers for requests
    fn build_headers(
        &self,
    ) -> Vec<(
        std::borrow::Cow<'static, str>,
        std::borrow::Cow<'static, str>,
    )> {
        let mut headers = Vec::with_capacity(4 + self.config.custom_headers.len());

        // Add auth headers if API key is provided
        if let Some(api_key) = &self.config.get_api_key() {
            headers.push(header("x-api-key", api_key.clone()));
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        // Standard headers
        headers.push(header("Content-Type", "application/json".to_string()));

        // Custom headers
        for (key, value) in &self.config.custom_headers {
            headers.push((
                std::borrow::Cow::Owned(key.clone()),
                std::borrow::Cow::Owned(value.clone()),
            ));
        }

        headers
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError> {
        let api_base = self.get_api_base()?;
        let url = format!("{}{}", api_base, endpoint);
        let headers = self.build_headers();

        if self.config.debug {
            debug!("Hosted vLLM request to {}", url);
        }

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| ProviderError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network(PROVIDER_NAME, e.to_string()))?;

        if !status.is_success() {
            let body_str = String::from_utf8_lossy(&response_bytes);
            return Err(self.map_http_error(status.as_u16(), &body_str));
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::response_parsing(
                PROVIDER_NAME,
                format!("Failed to parse response: {}", e),
            )
        })
    }

    /// Map HTTP status codes to ProviderError
    fn map_http_error(&self, status: u16, body: &str) -> ProviderError {
        match status {
            400 => ProviderError::invalid_request(PROVIDER_NAME, body),
            401 | 403 => ProviderError::authentication(
                PROVIDER_NAME,
                "Invalid API key or authentication failed",
            ),
            404 => ProviderError::model_not_found(PROVIDER_NAME, body),
            429 => {
                // Try to parse retry-after from response
                let retry_after = Self::parse_retry_after(body);
                ProviderError::rate_limit(PROVIDER_NAME, retry_after)
            }
            500 => ProviderError::api_error(PROVIDER_NAME, 500, "Internal server error"),
            502 | 503 => ProviderError::provider_unavailable(PROVIDER_NAME, "Service unavailable"),
            504 => ProviderError::timeout(PROVIDER_NAME, "Gateway timeout"),
            _ => ProviderError::api_error(PROVIDER_NAME, status, body),
        }
    }

    /// Try to parse retry_after from error response body
    fn parse_retry_after(body: &str) -> Option<u64> {
        // Try to parse JSON and extract retry_after
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
            if let Some(retry) = json.get("retry_after").and_then(|v| v.as_u64()) {
                return Some(retry);
            }
            if let Some(error) = json.get("error") {
                if let Some(retry) = error.get("retry_after").and_then(|v| v.as_u64()) {
                    return Some(retry);
                }
            }
        }
        None
    }

    /// Fetch available models from vLLM server
    pub async fn list_available_models(&self) -> Result<Vec<String>, ProviderError> {
        let api_base = self.get_api_base()?;
        let url = format!("{}/models", api_base);
        let headers = self.build_headers();

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::GET, headers, None::<serde_json::Value>)
            .await
            .map_err(|e| ProviderError::network(PROVIDER_NAME, e.to_string()))?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network(PROVIDER_NAME, e.to_string()))?;

        let json: serde_json::Value = serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::response_parsing(
                PROVIDER_NAME,
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
    ) -> Result<Vec<ChatResponse>, ProviderError> {
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

/// Convert HostedVLLMModelInfo to gateway ModelInfo
fn model_info_to_gateway_model(info: &HostedVLLMModelInfo) -> ModelInfo {
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
        provider: PROVIDER_NAME.to_string(),
        max_context_length: info.context_length,
        max_output_length: Some(info.max_output_tokens),
        supports_streaming: true,
        supports_tools: info.supports_tools,
        supports_multimodal: info.supports_vision,
        input_cost_per_1k_tokens: None,  // Self-hosted, no API costs
        output_cost_per_1k_tokens: None, // Self-hosted, no API costs
        currency: "USD".to_string(),
        capabilities,
        created_at: None,
        updated_at: None,
        metadata: HashMap::new(),
    }
}

#[async_trait]
impl LLMProvider for HostedVLLMProvider {
    type Config = HostedVLLMConfig;
    type Error = ProviderError;
    type ErrorMapper = crate::core::traits::error_mapper::DefaultErrorMapper;

    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        HOSTED_VLLM_CAPABILITIES
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
            .map_err(|e| ProviderError::serialization(PROVIDER_NAME, e.to_string()))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        serde_json::from_slice(raw_response).map_err(|e| {
            ProviderError::response_parsing(
                PROVIDER_NAME,
                format!("Failed to parse response: {}", e),
            )
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
        debug!("Hosted vLLM chat request: model={}", request.model);

        let request_json = serde_json::to_value(&request)
            .map_err(|e| ProviderError::serialization(PROVIDER_NAME, e.to_string()))?;

        let response = self
            .execute_request("/chat/completions", request_json)
            .await?;

        serde_json::from_value(response).map_err(|e| {
            ProviderError::response_parsing(
                PROVIDER_NAME,
                format!("Failed to parse chat response: {}", e),
            )
        })
    }

    async fn chat_completion_stream(
        &self,
        mut request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("Hosted vLLM streaming request: model={}", request.model);

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

        // Add custom headers
        for (key, value) in &self.config.custom_headers {
            req = req.header(key, value);
        }

        let response = req
            .send()
            .await
            .map_err(|e| ProviderError::network(PROVIDER_NAME, e.to_string()))?;

        // Check status
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.ok().unwrap_or_default();
            return Err(self.map_http_error(status, &body));
        }

        // Create SSE stream
        let stream = super::streaming::HostedVLLMStream::new(response.bytes_stream());
        Ok(Box::pin(stream))
    }

    async fn embeddings(
        &self,
        request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        debug!("Hosted vLLM embeddings request: model={}", request.model);

        let request_json = serde_json::to_value(&request)
            .map_err(|e| ProviderError::serialization(PROVIDER_NAME, e.to_string()))?;

        let response = self.execute_request("/embeddings", request_json).await?;

        serde_json::from_value(response).map_err(|e| {
            ProviderError::response_parsing(
                PROVIDER_NAME,
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
        // Hosted vLLM is self-hosted, so there are no API costs
        // Infrastructure costs would need to be calculated separately
        Ok(0.0)
    }
}
