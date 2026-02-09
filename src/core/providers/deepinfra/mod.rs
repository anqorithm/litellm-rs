//! DeepInfra Provider
//!
//! DeepInfra model hosting platform integration

pub mod chat;
pub mod chat_handler;
pub mod rerank;

// Re-export main components
pub use chat::DeepInfraChatTransformation;
pub use rerank::DeepInfraRerankTransformation;

use async_trait::async_trait;
use futures::Stream;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::core::providers::base_provider::{BaseHttpClient, BaseProviderConfig};
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

/// DeepInfra configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepInfraConfig {
    /// API key for DeepInfra
    pub api_key: Option<String>,
    /// API base URL (default: <https://api.deepinfra.com>)
    pub api_base: Option<String>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Max retries
    pub max_retries: u32,
}

impl Default for DeepInfraConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: Some("https://api.deepinfra.com".to_string()),
            timeout: 60,
            max_retries: 3,
        }
    }
}

impl DeepInfraConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, DeepInfraError> {
        let api_key = std::env::var("DEEPINFRA_API_KEY").ok();

        let api_base = std::env::var("DEEPINFRA_API_BASE")
            .unwrap_or_else(|_| "https://api.deepinfra.com".to_string());

        Ok(Self {
            api_key,
            api_base: Some(api_base),
            timeout: 60,
            max_retries: 3,
        })
    }

    /// Get effective API key
    pub fn get_effective_api_key(&self) -> Option<&String> {
        self.api_key.as_ref()
    }

    /// Get effective API base URL
    pub fn get_effective_api_base(&self) -> &str {
        self.api_base
            .as_deref()
            .unwrap_or("https://api.deepinfra.com")
    }
}

/// DeepInfra error type (alias to unified ProviderError)
pub type DeepInfraError = ProviderError;

/// DeepInfra provider
#[derive(Debug, Clone)]
pub struct DeepInfraProvider {
    config: DeepInfraConfig,
    base_client: BaseHttpClient,
}

impl DeepInfraProvider {
    /// Create new DeepInfra provider
    pub fn new(config: DeepInfraConfig) -> Result<Self, DeepInfraError> {
        // Create base provider config
        let base_config = BaseProviderConfig {
            api_key: config.api_key.clone(),
            api_base: config.api_base.clone(),
            timeout: Some(config.timeout),
            max_retries: Some(config.max_retries),
            headers: None,
            organization: None,
            api_version: None,
        };

        // Create base HTTP client
        let base_client = BaseHttpClient::new(base_config)
            .map_err(|e| ProviderError::configuration("deepinfra", e.to_string()))?;

        Ok(Self {
            config,
            base_client,
        })
    }

    /// Build request headers
    fn build_headers(&self) -> Result<HeaderMap, DeepInfraError> {
        let mut headers = HeaderMap::new();

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(api_key) = &self.config.api_key {
            let auth_value =
                HeaderValue::from_str(&format!("Bearer {}", api_key)).map_err(|e| {
                    ProviderError::configuration("deepinfra", format!("Invalid API key: {}", e))
                })?;
            headers.insert(AUTHORIZATION, auth_value);
        }

        Ok(headers)
    }

    /// Make a chat completion request
    async fn send_chat_request(
        &self,
        request: ChatRequest,
        stream: bool,
    ) -> Result<reqwest::Response, DeepInfraError> {
        let url = format!(
            "{}/v1/chat/completions",
            self.config.get_effective_api_base()
        );
        let headers = self.build_headers()?;

        // Transform request to DeepInfra format
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "stream": stream,
        });

        if let Some(temperature) = request.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        if let Some(top_p) = request.top_p {
            body["top_p"] = serde_json::json!(top_p);
        }

        // Send request using base HTTP client
        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("deepinfra", format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ProviderError::api_error("deepinfra", status, error_body));
        }

        Ok(response)
    }
}

/// DeepInfra error mapper
#[derive(Debug)]
pub struct DeepInfraErrorMapper;

impl ErrorMapper<DeepInfraError> for DeepInfraErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> DeepInfraError {
        match status_code {
            401 => ProviderError::authentication(
                "deepinfra",
                format!("Invalid API key: {}", response_body),
            ),
            403 => ProviderError::authentication(
                "deepinfra",
                format!("Permission denied: {}", response_body),
            ),
            404 => ProviderError::model_not_found(
                "deepinfra",
                format!("Model not found: {}", response_body),
            ),
            429 => ProviderError::rate_limit("deepinfra", None),
            500..=599 => ProviderError::api_error(
                "deepinfra",
                status_code,
                format!("Server error: {}", response_body),
            ),
            _ => ProviderError::api_error(
                "deepinfra",
                status_code,
                format!("HTTP {}: {}", status_code, response_body),
            ),
        }
    }
}

#[async_trait]
impl LLMProvider for DeepInfraProvider {
    type Config = DeepInfraConfig;
    type Error = DeepInfraError;
    type ErrorMapper = DeepInfraErrorMapper;

    fn name(&self) -> &'static str {
        "deepinfra"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        static CAPABILITIES: &[ProviderCapability] = &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
        ];
        CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        // Return empty slice for now to avoid ModelInfo structure errors
        &[]
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &["temperature", "max_tokens", "top_p", "stream"]
    }

    async fn map_openai_params(
        &self,
        params: std::collections::HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>, Self::Error> {
        // DeepInfra API is largely OpenAI-compatible
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<serde_json::Value, Self::Error> {
        use serde_json::json;

        let mut body = json!({
            "model": request.model,
            "messages": request.messages,
        });

        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }

        if request.stream {
            body["stream"] = json!(true);
        }

        Ok(body)
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response_text = std::str::from_utf8(raw_response).map_err(|e| {
            ProviderError::serialization("deepinfra", format!("Invalid UTF-8: {}", e))
        })?;

        let _response_json: serde_json::Value =
            serde_json::from_str(response_text).map_err(|e| {
                ProviderError::serialization("deepinfra", format!("Invalid JSON: {}", e))
            })?;

        // Basic response structure - would need full implementation
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let chat_response = ChatResponse {
            id: request_id.to_string(),
            object: "chat.completion".to_string(),
            created: timestamp as i64,
            model: model.to_string(),
            choices: vec![], // TODO: Parse actual choices
            usage: None,     // TODO: Parse usage
            system_fingerprint: None,
        };

        Ok(chat_response)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        DeepInfraErrorMapper
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Basic cost calculation for DeepInfra models
        let cost = match model {
            "meta-llama/Llama-2-70b-chat-hf" => {
                (input_tokens as f64 * 0.0007 + output_tokens as f64 * 0.0009) / 1000.0
            }
            _ => 0.0,
        };
        Ok(cost)
    }

    fn supports_model(&self, model: &str) -> bool {
        // DeepInfra supports many open source models
        model.contains("llama")
            || model.contains("mistral")
            || model.contains("mixtral")
            || model.contains("falcon")
    }

    async fn health_check(&self) -> HealthStatus {
        if self.config.api_key.is_some() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        let response = self.send_chat_request(request.clone(), false).await?;

        // Parse response
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            ProviderError::serialization("deepinfra", format!("Failed to parse response: {}", e))
        })?;

        // Transform to ChatResponse
        let chat_response: ChatResponse = serde_json::from_value(response_json).map_err(|e| {
            ProviderError::serialization(
                "deepinfra",
                format!("Failed to parse ChatResponse: {}", e),
            )
        })?;

        Ok(chat_response)
    }

    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        // For now, return not implemented since streaming requires SSE parsing
        // which would need to be implemented separately
        Err(ProviderError::not_implemented(
            "deepinfra",
            "Streaming not yet implemented",
        ))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        Err(ProviderError::not_supported("deepinfra", "Embeddings"))
    }

    async fn image_generation(
        &self,
        _request: ImageGenerationRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, Self::Error> {
        Err(ProviderError::not_supported(
            "deepinfra",
            "Image generation",
        ))
    }
}

impl ProviderConfig for DeepInfraConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_none() {
            return Err("DeepInfra API key is required".to_string());
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

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
