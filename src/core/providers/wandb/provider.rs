//! Weights & Biases (W&B) Provider Integration
//!
//! This module provides a W&B integration for LiteLLM-RS that can be used
//! as a callback/wrapper around LLM providers to log all calls to W&B.
//!
//! Unlike traditional LLM providers, W&B is primarily an observability/logging
//! integration rather than a model provider.

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

use super::config::{PROVIDER_NAME, WandbConfig};
use super::logger::{LLMCallLog, WandbLogger};
use crate::core::providers::base_provider::HttpErrorMapper;
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

/// Static capabilities - W&B doesn't directly provide LLM capabilities
/// but can log any capability through wrapped providers
const WANDB_CAPABILITIES: &[ProviderCapability] = &[];

/// W&B error type (using unified ProviderError)
pub type WandbError = ProviderError;

/// W&B error mapper
pub struct WandbErrorMapper;

impl ErrorMapper<WandbError> for WandbErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> WandbError {
        HttpErrorMapper::map_status_code(PROVIDER_NAME, status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> WandbError {
        HttpErrorMapper::parse_json_error(PROVIDER_NAME, error_response)
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> WandbError {
        ProviderError::network(PROVIDER_NAME, error.to_string())
    }

    fn map_parsing_error(&self, error: &dyn std::error::Error) -> WandbError {
        ProviderError::response_parsing(PROVIDER_NAME, error.to_string())
    }

    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> WandbError {
        ProviderError::timeout(
            PROVIDER_NAME,
            format!("Request timed out after {:?}", timeout_duration),
        )
    }
}

/// W&B Provider - Wrapper for logging LLM calls to Weights & Biases
///
/// This provider wraps another LLM provider and logs all calls to W&B.
/// It doesn't provide LLM capabilities itself but acts as a transparent
/// logging layer.
///
/// # Example
/// ```rust,no_run
/// use litellm_rs::core::providers::wandb::{WandbProvider, WandbConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a W&B provider with an inner provider
/// let config = WandbConfig::new("your-wandb-api-key")
///     .with_project("my-llm-project")
///     .with_entity("my-team");
///
/// let wandb = WandbProvider::new(config).await?;
///
/// // Initialize a run
/// wandb.init_run().await?;
///
/// // Log LLM calls manually or use as a wrapper
/// wandb.log_call(
///     "openai",
///     "gpt-4",
///     Some(100),
///     Some(50),
///     Some(0.01),
///     200,
///     true,
///     None,
/// ).await?;
///
/// // Finish the run when done
/// wandb.finish().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct WandbProvider {
    config: WandbConfig,
    logger: Arc<WandbLogger>,
}

impl WandbProvider {
    /// Create a new W&B provider
    pub async fn new(config: WandbConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration(PROVIDER_NAME, e))?;

        let logger = WandbLogger::new(config.clone())?;

        Ok(Self {
            config,
            logger: Arc::new(logger),
        })
    }

    /// Create provider from environment variables
    pub async fn from_env() -> Result<Self, ProviderError> {
        let config = WandbConfig::from_env()?;
        Self::new(config).await
    }

    /// Create provider with just API key
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = WandbConfig::new(api_key);
        Self::new(config).await
    }

    /// Initialize a W&B run
    ///
    /// This should be called before logging any calls.
    pub async fn init_run(&self) -> Result<(), ProviderError> {
        self.logger.init_run().await?;
        Ok(())
    }

    /// Finish the current W&B run
    ///
    /// This flushes all pending logs and marks the run as complete.
    pub async fn finish(&self) -> Result<(), ProviderError> {
        self.logger.finish().await
    }

    /// Log an LLM call to W&B
    ///
    /// This is the main method for manually logging LLM calls.
    #[allow(clippy::too_many_arguments)]
    pub async fn log_call(
        &self,
        provider: &str,
        model: &str,
        input_tokens: Option<u32>,
        output_tokens: Option<u32>,
        cost_usd: Option<f64>,
        latency_ms: u64,
        success: bool,
        error: Option<&str>,
    ) -> Result<(), ProviderError> {
        let mut log = LLMCallLog::new(provider, model).with_latency(latency_ms);

        if let (Some(input), Some(output)) = (input_tokens, output_tokens) {
            log = log.with_token_usage(input, output, input + output);
        }

        if let Some(cost) = cost_usd {
            log = log.with_cost(cost);
        }

        if !success {
            log = log.with_error(error.unwrap_or("Unknown error"));
        }

        self.logger.log(log).await
    }

    /// Log a chat completion request and response
    pub async fn log_chat_completion(
        &self,
        provider: &str,
        request: &ChatRequest,
        response: Option<&ChatResponse>,
        latency_ms: u64,
        error: Option<&str>,
    ) -> Result<(), ProviderError> {
        let log = super::logger::create_chat_log(
            provider,
            &request.model,
            request,
            response,
            latency_ms,
            error,
        );
        self.logger.log(log).await
    }

    /// Get the run summary
    pub async fn get_summary(&self) -> super::logger::RunSummary {
        self.logger.get_summary().await
    }

    /// Get the current run info
    pub async fn get_run(&self) -> Option<super::logger::WandbRun> {
        self.logger.get_run().await
    }

    /// Flush pending logs
    pub async fn flush(&self) -> Result<(), ProviderError> {
        self.logger.flush().await
    }

    /// Check if logging is enabled
    pub fn is_enabled(&self) -> bool {
        self.logger.is_enabled()
    }

    /// Get the underlying logger
    pub fn logger(&self) -> Arc<WandbLogger> {
        self.logger.clone()
    }

    /// Wrap another provider's chat completion with logging
    pub async fn wrap_chat_completion<P: LLMProvider>(
        &self,
        provider: &P,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, P::Error>
    where
        P::Error: From<ProviderError>,
    {
        let start = Instant::now();
        let provider_name = provider.name();
        let model = request.model.clone();

        let result = provider.chat_completion(request.clone(), context).await;

        let latency_ms = start.elapsed().as_millis() as u64;

        // Log the result
        match &result {
            Ok(response) => {
                debug!("Logging successful chat completion to W&B");
                let _ = self
                    .log_chat_completion(provider_name, &request, Some(response), latency_ms, None)
                    .await;
            }
            Err(e) => {
                debug!("Logging failed chat completion to W&B");
                let _ = self
                    .log_chat_completion(
                        provider_name,
                        &request,
                        None,
                        latency_ms,
                        Some(&format!("{:?}", e)),
                    )
                    .await;
            }
        }

        // Log to W&B (fire and forget)
        if let Err(e) = self
            .logger
            .log(LLMCallLog::new(provider_name, &model).with_latency(latency_ms))
            .await
        {
            debug!("Failed to log to W&B: {:?}", e);
        }

        result
    }
}

impl Clone for WandbProvider {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            logger: self.logger.clone(),
        }
    }
}

#[async_trait]
impl LLMProvider for WandbProvider {
    type Config = WandbConfig;
    type Error = WandbError;
    type ErrorMapper = WandbErrorMapper;

    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        WANDB_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        // W&B doesn't provide models directly
        &[]
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        // W&B doesn't process OpenAI params
        &[]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // Pass through params unchanged
        Ok(params)
    }

    async fn transform_request(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        Err(ProviderError::not_supported(
            PROVIDER_NAME,
            "W&B is a logging integration, not an LLM provider. Use wrap_chat_completion() instead.",
        ))
    }

    async fn transform_response(
        &self,
        _raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        Err(ProviderError::not_supported(
            PROVIDER_NAME,
            "W&B is a logging integration, not an LLM provider.",
        ))
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        WandbErrorMapper
    }

    async fn chat_completion(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        Err(ProviderError::not_supported(
            PROVIDER_NAME,
            "W&B is a logging integration, not an LLM provider. Use wrap_chat_completion() to wrap another provider.",
        ))
    }

    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        Err(ProviderError::not_supported(
            PROVIDER_NAME,
            "W&B is a logging integration, not an LLM provider.",
        ))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        Err(ProviderError::not_supported(
            PROVIDER_NAME,
            "W&B is a logging integration, not an LLM provider.",
        ))
    }

    async fn health_check(&self) -> HealthStatus {
        // Check if we can reach W&B
        if !self.config.enabled {
            return HealthStatus::Degraded;
        }

        if self.config.get_effective_api_key().is_some() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }

    async fn calculate_cost(
        &self,
        _model: &str,
        _input_tokens: u32,
        _output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // W&B itself doesn't have model costs
        Ok(0.0)
    }
}


#[cfg(test)]
#[path = "provider_tests.rs"]
mod tests;
