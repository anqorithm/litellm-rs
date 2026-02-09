//! Dashscope (Alibaba Cloud) AI Provider
//!
//! Dashscope provides access to Alibaba's Qwen series models with an OpenAI-compatible API.
//! API Base: <https://dashscope.aliyuncs.com/compatible-mode/v1>

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

// Re-export submodules
pub mod chat;

// Static capabilities
const DASHSCOPE_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::FunctionCalling,
];

/// Dashscope provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashscopeConfig {
    /// API key for authentication
    pub api_key: String,
    /// API base URL (defaults to <https://dashscope.aliyuncs.com/compatible-mode/v1>)
    pub api_base: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
}

impl Default for DashscopeConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            timeout_seconds: 60,
            max_retries: 3,
        }
    }
}

impl ProviderConfig for DashscopeConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("Dashscope API key is required".to_string());
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

/// Dashscope error type (simplified using ProviderError)
pub type DashscopeError = ProviderError;

/// Dashscope error mapper
pub struct DashscopeErrorMapper;

impl ErrorMapper<DashscopeError> for DashscopeErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> DashscopeError {
        HttpErrorMapper::map_status_code("dashscope", status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> DashscopeError {
        HttpErrorMapper::parse_json_error("dashscope", error_response)
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> DashscopeError {
        ProviderError::network("dashscope", error.to_string())
    }

    fn map_parsing_error(&self, error: &dyn std::error::Error) -> DashscopeError {
        ProviderError::response_parsing("dashscope", error.to_string())
    }

    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> DashscopeError {
        ProviderError::timeout(
            "dashscope",
            format!("Request timed out after {:?}", timeout_duration),
        )
    }
}

/// Dashscope provider implementation
#[derive(Debug, Clone)]
pub struct DashscopeProvider {
    config: DashscopeConfig,
    base_client: BaseHttpClient,
    models: Vec<ModelInfo>,
}

impl DashscopeProvider {
    /// Create a new Dashscope provider instance
    pub async fn new(config: DashscopeConfig) -> Result<Self, DashscopeError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("dashscope", e))?;

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

        // Define supported models with pricing (CNY per 1k tokens)
        let models = vec![
            ModelInfo {
                id: "qwen-turbo".to_string(),
                name: "Qwen Turbo".to_string(),
                provider: "dashscope".to_string(),
                max_context_length: 131072,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0008),
                output_cost_per_1k_tokens: Some(0.002),
                currency: "CNY".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "qwen-plus".to_string(),
                name: "Qwen Plus".to_string(),
                provider: "dashscope".to_string(),
                max_context_length: 131072,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.004),
                output_cost_per_1k_tokens: Some(0.012),
                currency: "CNY".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "qwen-max".to_string(),
                name: "Qwen Max".to_string(),
                provider: "dashscope".to_string(),
                max_context_length: 32768,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.02),
                output_cost_per_1k_tokens: Some(0.06),
                currency: "CNY".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "qwen-max-longcontext".to_string(),
                name: "Qwen Max Long Context".to_string(),
                provider: "dashscope".to_string(),
                max_context_length: 1000000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.02),
                output_cost_per_1k_tokens: Some(0.06),
                currency: "CNY".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "qwen-vl-plus".to_string(),
                name: "Qwen VL Plus".to_string(),
                provider: "dashscope".to_string(),
                max_context_length: 32768,
                max_output_length: Some(2048),
                supports_streaming: true,
                supports_tools: false,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.008),
                output_cost_per_1k_tokens: Some(0.008),
                currency: "CNY".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "qwen-vl-max".to_string(),
                name: "Qwen VL Max".to_string(),
                provider: "dashscope".to_string(),
                max_context_length: 32768,
                max_output_length: Some(2048),
                supports_streaming: true,
                supports_tools: false,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.02),
                output_cost_per_1k_tokens: Some(0.02),
                currency: "CNY".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "qwen2.5-72b-instruct".to_string(),
                name: "Qwen 2.5 72B Instruct".to_string(),
                provider: "dashscope".to_string(),
                max_context_length: 131072,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.004),
                output_cost_per_1k_tokens: Some(0.012),
                currency: "CNY".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "qwen2.5-32b-instruct".to_string(),
                name: "Qwen 2.5 32B Instruct".to_string(),
                provider: "dashscope".to_string(),
                max_context_length: 131072,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0035),
                output_cost_per_1k_tokens: Some(0.007),
                currency: "CNY".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "qwen2.5-14b-instruct".to_string(),
                name: "Qwen 2.5 14B Instruct".to_string(),
                provider: "dashscope".to_string(),
                max_context_length: 131072,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.002),
                output_cost_per_1k_tokens: Some(0.006),
                currency: "CNY".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "qwen2.5-7b-instruct".to_string(),
                name: "Qwen 2.5 7B Instruct".to_string(),
                provider: "dashscope".to_string(),
                max_context_length: 131072,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.001),
                output_cost_per_1k_tokens: Some(0.002),
                currency: "CNY".to_string(),
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
        } else if base.ends_with('/') {
            format!("{}chat/completions", base)
        } else {
            format!("{}/chat/completions", base)
        }
    }
}

#[async_trait]
impl LLMProvider for DashscopeProvider {
    type Config = DashscopeConfig;
    type Error = DashscopeError;
    type ErrorMapper = DashscopeErrorMapper;

    fn name(&self) -> &'static str {
        "dashscope"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        DASHSCOPE_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("Dashscope chat request: model={}", request.model);

        // Transform request (Dashscope uses OpenAI-compatible format but needs content list to string conversion)
        let body = self.transform_request(request, context).await?;

        // Build URL based on config
        let url = self.build_chat_url();

        let headers = HeaderBuilder::new()
            .with_bearer_token(&self.config.api_key)
            .with_content_type("application/json")
            .build_reqwest()
            .map_err(|e| ProviderError::invalid_request("dashscope", e.to_string()))?;

        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("dashscope", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::api_error("dashscope", status, body));
        }

        response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("dashscope", e.to_string()))
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("Dashscope streaming chat request: model={}", request.model);

        // Transform request
        let mut body = self.transform_request(request, context).await?;
        body["stream"] = serde_json::json!(true);

        // Build URL based on config
        let url = self.build_chat_url();

        let headers = HeaderBuilder::new()
            .with_bearer_token(&self.config.api_key)
            .with_content_type("application/json")
            .build_reqwest()
            .map_err(|e| ProviderError::invalid_request("dashscope", e.to_string()))?;

        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("dashscope", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::api_error("dashscope", status, body));
        }

        // Parse SSE stream using shared infrastructure
        use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEParser};
        use futures::StreamExt;

        let transformer = OpenAICompatibleTransformer::new("dashscope");
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
                    Err(e) => Some(Err(ProviderError::network("dashscope", e.to_string()))),
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
        // Dashscope does support embeddings via text-embedding-v2/v3
        // For now, return not supported - can be implemented later
        Err(ProviderError::not_supported("dashscope", "embeddings"))
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
                            "Dashscope health check failed: status={}",
                            response.status()
                        );
                        HealthStatus::Unhealthy
                    }
                    Err(e) => {
                        debug!("Dashscope health check error: {}", e);
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
            "seed",
            "top_k", // Qwen-specific
        ]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // Dashscope is OpenAI-compatible, pass-through most parameters
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        // Use the OpenAI transformer from base_provider
        // Note: Dashscope doesn't support content in list format, so we need to convert
        let mut body = OpenAIRequestTransformer::transform_chat_request(&request);

        // Convert content list to string if needed (Dashscope requirement)
        if let Some(messages) = body.get_mut("messages") {
            if let Some(messages_array) = messages.as_array_mut() {
                for message in messages_array {
                    if let Some(content) = message.get("content") {
                        if content.is_array() {
                            // Convert array content to string
                            if let Some(content_array) = content.as_array() {
                                let text_parts: Vec<String> = content_array
                                    .iter()
                                    .filter_map(|part| {
                                        if let Some(text) = part.get("text") {
                                            text.as_str().map(|s| s.to_string())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                message["content"] = serde_json::json!(text_parts.join("\n"));
                            }
                        }
                    }
                }
            }
        }

        Ok(body)
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        // Parse Dashscope response (OpenAI-compatible format)
        serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing("dashscope", e.to_string()))
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        DashscopeErrorMapper
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
            .ok_or_else(|| ProviderError::model_not_found("dashscope", model.to_string()))?;

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
