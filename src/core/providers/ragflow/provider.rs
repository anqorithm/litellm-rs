//! Main RAGFlow Provider Implementation

use async_trait::async_trait;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

use super::config::RagflowConfig;
use super::model_info::{get_available_models, get_model_info};
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

const RAGFLOW_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
];

#[derive(Debug, Clone)]
pub struct RagflowProvider {
    config: RagflowConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl RagflowProvider {
    pub async fn new(config: RagflowConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("ragflow", e))?;

        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration("ragflow", format!("Failed to create pool manager: {}", e))
        })?);

        let models = get_available_models()
            .iter()
            .filter_map(|id| get_model_info(id))
            .map(|info| {
                let capabilities = vec![
                    ProviderCapability::ChatCompletion,
                    ProviderCapability::ChatCompletionStream,
                ];

                ModelInfo {
                    id: info.model_id.to_string(),
                    name: info.display_name.to_string(),
                    provider: "ragflow".to_string(),
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

    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = RagflowConfig {
            api_key: Some(api_key.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError> {
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
            .map_err(|e| ProviderError::network("ragflow", e.to_string()))?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("ragflow", e.to_string()))?;

        if !status.is_success() {
            let error_body = String::from_utf8_lossy(&response_bytes);
            return Err(match status.as_u16() {
                400 => ProviderError::invalid_request("ragflow", error_body.to_string()),
                401 => ProviderError::authentication("ragflow", "Invalid API key"),
                404 => ProviderError::model_not_found("ragflow", "Model not found"),
                429 => ProviderError::rate_limit("ragflow", None),
                _ => ProviderError::api_error(
                    "ragflow",
                    status.as_u16(),
                    format!("API error {}: {}", status, error_body),
                ),
            });
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::api_error("ragflow", 500, format!("Failed to parse response: {}", e))
        })
    }
}

#[async_trait]
impl LLMProvider for RagflowProvider {
    type Config = RagflowConfig;
    type Error = ProviderError;
    type ErrorMapper = crate::core::traits::error_mapper::DefaultErrorMapper;

    fn name(&self) -> &'static str {
        "ragflow"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        RAGFLOW_CAPABILITIES
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
            "user",
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
            .map_err(|e| ProviderError::invalid_request("ragflow", e.to_string()))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        serde_json::from_slice(raw_response).map_err(|e| {
            ProviderError::api_error("ragflow", 500, format!("Failed to parse response: {}", e))
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
        debug!("RAGFlow chat request: model={}", request.model);

        let request_json = serde_json::to_value(&request)
            .map_err(|e| ProviderError::invalid_request("ragflow", e.to_string()))?;

        let response = self
            .execute_request("/chat/completions", request_json)
            .await?;

        serde_json::from_value(response).map_err(|e| {
            ProviderError::api_error(
                "ragflow",
                500,
                format!("Failed to parse chat response: {}", e),
            )
        })
    }

    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        Err(ProviderError::not_supported(
            "ragflow",
            "Streaming not yet implemented",
        ))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        Err(ProviderError::not_supported(
            "ragflow",
            "Embeddings not supported",
        ))
    }

    async fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        let model_info = get_model_info(model).ok_or_else(|| {
            ProviderError::model_not_found("ragflow", format!("Unknown model: {}", model))
        })?;

        let input_cost = (input_tokens as f64) * (model_info.input_cost_per_million / 1_000_000.0);
        let output_cost =
            (output_tokens as f64) * (model_info.output_cost_per_million / 1_000_000.0);
        Ok(input_cost + output_cost)
    }
}
