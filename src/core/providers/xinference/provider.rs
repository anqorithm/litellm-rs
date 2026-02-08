//! Xinference Provider Implementation

use async_trait::async_trait;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

use super::config::XinferenceConfig;
use super::error::{XinferenceError, XinferenceErrorMapper};
use super::model_info::{get_available_models, get_model_info};
use crate::core::providers::base::{GlobalPoolManager, HeaderPair, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
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

const XINFERENCE_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::Embeddings,
];

/// Xinference provider implementation
#[derive(Debug, Clone)]
pub struct XinferenceProvider {
    config: XinferenceConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl XinferenceProvider {
    /// Create a new Xinference provider
    pub async fn new(config: XinferenceConfig) -> Result<Self, XinferenceError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("xinference", e))?;

        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration(
                "xinference",
                format!("Failed to create pool manager: {}", e),
            )
        })?);

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
                    provider: "xinference".to_string(),
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

    /// Create provider with API base URL
    pub async fn with_api_base(api_base: impl Into<String>) -> Result<Self, XinferenceError> {
        let config = XinferenceConfig {
            api_base: Some(api_base.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Build headers
    fn build_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::new();
        if let Some(api_key) = self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));
        headers
    }

    /// Execute request
    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, XinferenceError> {
        let url = format!("{}{}", self.config.get_api_base(), endpoint);
        let headers = self.build_headers();

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| ProviderError::network("xinference", e.to_string()))?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("xinference", e.to_string()))?;

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::api_error(
                "xinference",
                500,
                format!("Failed to parse response: {}", e),
            )
        })
    }
}

#[async_trait]
impl LLMProvider for XinferenceProvider {
    type Config = XinferenceConfig;
    type Error = XinferenceError;
    type ErrorMapper = XinferenceErrorMapper;

    fn name(&self) -> &'static str {
        "xinference"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        XINFERENCE_CAPABILITIES
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
            "frequency_penalty",
            "presence_penalty",
            "n",
            "tools",
            "tool_choice",
        ]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<HashMap<String, serde_json::Value>, Self::Error> {
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<serde_json::Value, Self::Error> {
        serde_json::to_value(&request)
            .map_err(|e| ProviderError::invalid_request("xinference", e.to_string()))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        serde_json::from_slice(raw_response).map_err(|e| {
            ProviderError::api_error(
                "xinference",
                500,
                format!("Failed to parse response: {}", e),
            )
        })
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        XinferenceErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("Xinference chat request: model={}", request.model);

        let request_json = serde_json::to_value(&request)
            .map_err(|e| ProviderError::invalid_request("xinference", e.to_string()))?;

        let response = self
            .execute_request("/chat/completions", request_json)
            .await?;

        serde_json::from_value(response).map_err(|e| {
            ProviderError::api_error(
                "xinference",
                500,
                format!("Failed to parse response: {}", e),
            )
        })
    }

    async fn chat_completion_stream(
        &self,
        mut request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("Xinference streaming request: model={}", request.model);

        request.stream = true;

        let url = format!("{}/chat/completions", self.config.get_api_base());
        let client = reqwest::Client::new();

        let mut req_builder = client.post(&url);
        if let Some(api_key) = self.config.get_api_key() {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }
        req_builder = req_builder.header("Content-Type", "application/json");

        let response = req_builder
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::network("xinference", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.ok();
            return Err(ProviderError::api_error(
                "xinference",
                status,
                format!("Stream request failed: {} - {:?}", status, body),
            ));
        }

        let stream = crate::core::providers::openai::streaming::create_openai_stream(
            response.bytes_stream(),
        );

        use futures::StreamExt;
        let mapped_stream = stream.map(|result| {
            result.map_err(|e| ProviderError::api_error("xinference", 500, e.to_string()))
        });

        Ok(Box::pin(mapped_stream))
    }

    async fn embeddings(
        &self,
        request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        debug!("Xinference embeddings request: model={}", request.model);

        let request_json = serde_json::to_value(&request)
            .map_err(|e| ProviderError::invalid_request("xinference", e.to_string()))?;

        let response = self.execute_request("/embeddings", request_json).await?;

        serde_json::from_value(response).map_err(|e| {
            ProviderError::api_error(
                "xinference",
                500,
                format!("Failed to parse response: {}", e),
            )
        })
    }

    async fn health_check(&self) -> HealthStatus {
        let url = format!("{}/models", self.config.get_api_base());
        let headers = self.build_headers();

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
        _model: &str,
        _input_tokens: u32,
        _output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Local inference is typically free
        Ok(0.0)
    }
}
