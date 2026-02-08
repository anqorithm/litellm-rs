//! Main Lambda Labs AI Provider Implementation
//!
//! Implements the LLMProvider trait for Lambda Labs GPU-accelerated inference.

use async_trait::async_trait;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

use super::config::LambdaAIConfig;
use super::error::LambdaAIError;
use super::model_info::{get_available_models, get_model_info};
use crate::core::providers::base::{GlobalPoolManager, HttpMethod, header, streaming_client};
use crate::core::traits::{
    ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    ChatRequest, EmbeddingRequest,
    context::RequestContext,
    health::HealthStatus,
    model::ModelInfo,
    model::ProviderCapability,
    responses::{ChatChunk, ChatResponse, EmbeddingResponse},
};

/// Provider name constant
pub const PROVIDER_NAME: &str = "lambda_ai";

/// Static capabilities for Lambda Labs AI provider
const LAMBDA_AI_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
];

/// Lambda Labs AI provider implementation
#[derive(Debug, Clone)]
pub struct LambdaAIProvider {
    config: LambdaAIConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl LambdaAIProvider {
    /// Create a new Lambda Labs AI provider instance
    pub async fn new(config: LambdaAIConfig) -> Result<Self, LambdaAIError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| LambdaAIError::configuration(PROVIDER_NAME, e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            LambdaAIError::configuration(
                PROVIDER_NAME,
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        // Build model list from static configuration
        let models = get_available_models()
            .iter()
            .filter_map(|id| get_model_info(id))
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
                    provider: PROVIDER_NAME.to_string(),
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
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, LambdaAIError> {
        let config = LambdaAIConfig::new(api_key);
        Self::new(config).await
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, LambdaAIError> {
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
            .map_err(|e| LambdaAIError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| LambdaAIError::network(PROVIDER_NAME, e.to_string()))?;

        // Check for error responses
        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&response_bytes);
            return Err(match status.as_u16() {
                400 => LambdaAIError::invalid_request(PROVIDER_NAME, error_text.to_string()),
                401 => LambdaAIError::authentication(PROVIDER_NAME, "Invalid API key"),
                404 => LambdaAIError::model_not_found(
                    PROVIDER_NAME,
                    format!("Model not found: {}", error_text),
                ),
                429 => LambdaAIError::rate_limit(PROVIDER_NAME, None),
                500..=599 => {
                    LambdaAIError::provider_unavailable(PROVIDER_NAME, error_text.to_string())
                }
                _ => {
                    LambdaAIError::api_error(PROVIDER_NAME, status.as_u16(), error_text.to_string())
                }
            });
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            LambdaAIError::api_error(
                PROVIDER_NAME,
                500,
                format!("Failed to parse response: {}", e),
            )
        })
    }
}

#[async_trait]
impl LLMProvider for LambdaAIProvider {
    type Config = LambdaAIConfig;
    type Error = LambdaAIError;
    type ErrorMapper = crate::core::traits::error_mapper::DefaultErrorMapper;

    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        LAMBDA_AI_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn get_supported_openai_params(&self, model: &str) -> &'static [&'static str] {
        let is_reasoning = super::model_info::is_reasoning_model(model);

        if is_reasoning {
            &[
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
                "reasoning_effort",
            ]
        } else {
            &[
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
            ]
        }
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<HashMap<String, serde_json::Value>, Self::Error> {
        // Lambda Labs uses the same parameters as OpenAI
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<serde_json::Value, Self::Error> {
        // Convert to JSON value - Lambda Labs uses OpenAI-compatible format
        serde_json::to_value(&request)
            .map_err(|e| LambdaAIError::invalid_request(PROVIDER_NAME, e.to_string()))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        // Parse response - Lambda Labs uses OpenAI-compatible format
        let chat_response: ChatResponse = serde_json::from_slice(raw_response).map_err(|e| {
            LambdaAIError::api_error(
                PROVIDER_NAME,
                500,
                format!("Failed to parse response: {}", e),
            )
        })?;

        Ok(chat_response)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        crate::core::traits::error_mapper::DefaultErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("Lambda AI chat request: model={}", request.model);

        // Transform and execute
        let request_json = serde_json::to_value(&request)
            .map_err(|e| LambdaAIError::invalid_request(PROVIDER_NAME, e.to_string()))?;

        let response = self
            .execute_request("/chat/completions", request_json)
            .await?;

        serde_json::from_value(response).map_err(|e| {
            LambdaAIError::api_error(
                PROVIDER_NAME,
                500,
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
        debug!("Lambda AI streaming request: model={}", request.model);

        // Ensure streaming is enabled
        request.stream = true;

        // Get API configuration
        let api_key = self
            .config
            .get_api_key()
            .ok_or_else(|| LambdaAIError::authentication(PROVIDER_NAME, "API key is required"))?;

        // Execute streaming request using the global connection pool
        let url = format!("{}/chat/completions", self.config.get_api_base());
        let client = streaming_client();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LambdaAIError::network(PROVIDER_NAME, e.to_string()))?;

        // Check status
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.ok();
            return Err(match status {
                400 => LambdaAIError::invalid_request(
                    PROVIDER_NAME,
                    body.unwrap_or_else(|| "Bad request".to_string()),
                ),
                401 => LambdaAIError::authentication(PROVIDER_NAME, "Invalid API key"),
                429 => LambdaAIError::rate_limit(PROVIDER_NAME, None),
                _ => LambdaAIError::api_error(
                    PROVIDER_NAME,
                    status,
                    format!("Stream request failed: {}", status),
                ),
            });
        }

        // Create SSE stream
        let stream = super::streaming::LambdaAIStream::new(response.bytes_stream());
        Ok(Box::pin(stream))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        Err(LambdaAIError::not_supported(
            PROVIDER_NAME,
            "Lambda Labs does not currently support embeddings. Use text generation models instead.",
        ))
    }

    async fn health_check(&self) -> HealthStatus {
        // Simple health check - try to get models list
        let url = format!("{}/models", self.config.get_api_base());
        let mut headers = Vec::with_capacity(1);
        if let Some(api_key) = &self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        match self
            .pool_manager
            .execute_request(&url, HttpMethod::GET, headers, None::<serde_json::Value>)
            .await
        {
            Ok(_) => HealthStatus::Healthy,
            Err(_) => HealthStatus::Unhealthy,
        }
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        let model_info = get_model_info(model).ok_or_else(|| {
            LambdaAIError::model_not_found(PROVIDER_NAME, format!("Unknown model: {}", model))
        })?;

        let input_cost = (input_tokens as f64) * (model_info.input_cost_per_million / 1_000_000.0);
        let output_cost =
            (output_tokens as f64) * (model_info.output_cost_per_million / 1_000_000.0);
        Ok(input_cost + output_cost)
    }
}
