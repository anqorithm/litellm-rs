//! Main NLP Cloud Provider Implementation

use async_trait::async_trait;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

use super::config::NlpCloudConfig;
use super::model_info::{get_available_models, get_model_info};
use crate::core::providers::base::{GlobalPoolManager, HttpErrorMapper, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
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

const PROVIDER_NAME: &str = "nlp_cloud";

const NLP_CLOUD_CAPABILITIES: &[ProviderCapability] = &[ProviderCapability::ChatCompletion];

#[derive(Debug, Clone)]
pub struct NlpCloudProvider {
    config: NlpCloudConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl NlpCloudProvider {
    pub async fn new(config: NlpCloudConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration(PROVIDER_NAME, e))?;

        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration(
                PROVIDER_NAME,
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        let models = get_available_models()
            .iter()
            .filter_map(|id| get_model_info(id))
            .map(|info| ModelInfo {
                id: info.model_id.to_string(),
                name: info.display_name.to_string(),
                provider: PROVIDER_NAME.to_string(),
                max_context_length: info.max_context_length,
                max_output_length: Some(info.max_output_length),
                supports_streaming: false,
                supports_tools: info.supports_tools,
                supports_multimodal: info.supports_multimodal,
                input_cost_per_1k_tokens: Some(info.input_cost_per_million / 1000.0),
                output_cost_per_1k_tokens: Some(info.output_cost_per_million / 1000.0),
                currency: "USD".to_string(),
                capabilities: vec![ProviderCapability::ChatCompletion],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            })
            .collect();

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = NlpCloudConfig {
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
            headers.push(header("Authorization", format!("Token {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));

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
            let error_text = String::from_utf8_lossy(&response_bytes);
            return Err(Self::map_http_error(status.as_u16(), &error_text));
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::api_error(
                PROVIDER_NAME,
                500,
                format!("Failed to parse response: {}", e),
            )
        })
    }

    fn map_http_error(status: u16, body: &str) -> ProviderError {
        match status {
            400 => ProviderError::invalid_request(PROVIDER_NAME, body.to_string()),
            401 => ProviderError::authentication(PROVIDER_NAME, "Invalid API token"),
            403 => ProviderError::authentication(PROVIDER_NAME, "Access forbidden"),
            404 => ProviderError::model_not_found(PROVIDER_NAME, "Model not found"),
            429 => ProviderError::rate_limit(PROVIDER_NAME, None),
            500 => HttpErrorMapper::map_status_code(PROVIDER_NAME, 500, "Internal server error"),
            503 => ProviderError::provider_unavailable(PROVIDER_NAME, "Service unavailable"),
            _ => HttpErrorMapper::map_status_code(PROVIDER_NAME, status, body),
        }
    }
}

#[async_trait]
impl LLMProvider for NlpCloudProvider {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        NLP_CLOUD_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &["temperature", "max_tokens", "top_p", "stop"]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<HashMap<String, serde_json::Value>, ProviderError> {
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<serde_json::Value, ProviderError> {
        serde_json::to_value(&request)
            .map_err(|e| ProviderError::invalid_request(PROVIDER_NAME, e.to_string()))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, ProviderError> {
        serde_json::from_slice(raw_response).map_err(|e| {
            ProviderError::api_error(
                PROVIDER_NAME,
                500,
                format!("Failed to parse response: {}", e),
            )
        })
    }

    fn get_error_mapper(&self) -> Box<dyn ErrorMapper<ProviderError>> {
        Box::new(crate::core::traits::error_mapper::DefaultErrorMapper)
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, ProviderError> {
        debug!("NLP Cloud chat request: model={}", request.model);

        let request_json = serde_json::to_value(&request)
            .map_err(|e| ProviderError::invalid_request(PROVIDER_NAME, e.to_string()))?;

        let response = self
            .execute_request(&format!("/{}/chatbot", request.model), request_json)
            .await?;

        serde_json::from_value(response).map_err(|e| {
            ProviderError::api_error(
                PROVIDER_NAME,
                500,
                format!("Failed to parse chat response: {}", e),
            )
        })
    }

    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError>
    {
        Err(ProviderError::not_supported(PROVIDER_NAME, "Streaming"))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, ProviderError> {
        Err(ProviderError::not_supported(PROVIDER_NAME, "Embeddings"))
    }

    async fn health_check(&self) -> HealthStatus {
        if self.config.get_api_key().is_some() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, ProviderError> {
        let model_info = get_model_info(model).ok_or_else(|| {
            ProviderError::model_not_found(PROVIDER_NAME, format!("Unknown model: {}", model))
        })?;

        let input_cost = (input_tokens as f64) * (model_info.input_cost_per_million / 1_000_000.0);
        let output_cost =
            (output_tokens as f64) * (model_info.output_cost_per_million / 1_000_000.0);
        Ok(input_cost + output_cost)
    }
}
