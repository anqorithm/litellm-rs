//! Azure OpenAI Provider
//!
//! Azure OpenAI integration for LiteLLM using LLMProvider trait

pub mod assistants;
pub mod batches;
pub mod chat;
pub mod client;
pub mod config;
pub mod embed;
pub mod error;
pub mod image;
pub mod responses;
pub mod utils;

// Re-export core utilities
pub use client::{AzureClient, AzureConfigFactory, AzureRateLimitInfo};
pub use config::{AzureConfig, AzureModelInfo};
pub use error::{
    AzureError, AzureErrorMapper, azure_ad_error, azure_api_error, azure_config_error,
    azure_deployment_error, azure_header_error,
};
pub use utils::{AzureEndpointType, AzureUtils};

// Use the new unified cost calculation system
pub use crate::core::cost::providers::azure::{
    AzureCostCalculator, cost_per_token, get_azure_model_pricing,
};

// Re-export assistant functionality
pub use assistants::{AzureAssistantHandler, AzureAssistantUtils};

// Re-export batch functionality
pub use batches::{AzureBatchHandler, AzureBatchUtils};

// Re-export chat functionality
pub use chat::{AzureChatHandler, AzureChatUtils};

// Re-export embedding functionality
pub use embed::{AzureEmbeddingHandler, AzureEmbeddingUtils};

// Re-export image functionality
pub use image::{AzureImageHandler, AzureImageUtils};

// Re-export response processing functionality
pub use responses::{AzureResponseHandler, AzureResponseProcessor, AzureResponseUtils};

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;
use std::pin::Pin;

use crate::core::types::{
    ChatRequest, ImageGenerationRequest,
    context::RequestContext,
    embedding::EmbeddingRequest,
    health::HealthStatus,
    model::ModelInfo,
    model::ProviderCapability,
    responses::{ChatChunk, ChatResponse, EmbeddingResponse, ImageGenerationResponse},
};

use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

/// Main Azure OpenAI provider - complete implementation
#[derive(Debug, Clone)]
pub struct AzureOpenAIProvider {
    config: AzureConfig,
    chat_handler: AzureChatHandler,
    embedding_handler: AzureEmbeddingHandler,
    image_handler: AzureImageHandler,
    cost_calculator: AzureCostCalculator,
}

impl AzureOpenAIProvider {
    /// Create new Azure OpenAI provider
    pub fn new(config: AzureConfig) -> Result<Self, AzureError> {
        let chat_handler = AzureChatHandler::new(config.clone())?;
        let embedding_handler = AzureEmbeddingHandler::new(config.clone())?;
        let image_handler = AzureImageHandler::new(config.clone())?;
        let cost_calculator = AzureCostCalculator::new();

        Ok(Self {
            config,
            chat_handler,
            embedding_handler,
            image_handler,
            cost_calculator,
        })
    }

    /// Create from configuration
    pub fn from_config(config: AzureConfig) -> Result<Self, AzureError> {
        Self::new(config)
    }

    /// Get Azure configuration
    pub fn get_azure_config(&self) -> &AzureConfig {
        &self.config
    }

    /// Get cost calculator
    pub fn get_cost_calculator(&self) -> &AzureCostCalculator {
        &self.cost_calculator
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self, AzureError> {
        let config = AzureConfig::new();
        Self::new(config)
    }

    /// Create with API key
    pub fn with_api_key(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Result<Self, AzureError> {
        let config = AzureConfig::new()
            .with_api_key(api_key.into())
            .with_azure_endpoint(endpoint.into());
        Self::new(config)
    }
}

// Azure error mapper is now re-exported from common_utils

/// Implement the unified LLMProvider trait for AzureOpenAIProvider
#[async_trait]
impl LLMProvider for AzureOpenAIProvider {
    type Config = AzureConfig;
    type Error = AzureError;
    type ErrorMapper = AzureErrorMapper;

    fn name(&self) -> &'static str {
        "azure_openai"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        static CAPABILITIES: &[ProviderCapability] = &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
            ProviderCapability::Embeddings,
            ProviderCapability::ImageGeneration,
            ProviderCapability::FunctionCalling,
            ProviderCapability::ToolCalling,
        ];
        CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        // Return empty for now - Azure uses deployment names
        &[]
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[
            "temperature",
            "max_tokens",
            "top_p",
            "frequency_penalty",
            "presence_penalty",
            "stream",
            "functions",
            "function_call",
            "tools",
            "tool_choice",
        ]
    }

    async fn map_openai_params(
        &self,
        params: std::collections::HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>, Self::Error> {
        // Azure OpenAI API is largely compatible with OpenAI, minimal mapping needed
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        self.chat_handler.transform_request(&request)
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response_json: Value = serde_json::from_slice(raw_response)?;
        self.chat_handler.transform_response(response_json, model)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        AzureErrorMapper
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Basic cost calculation for Azure OpenAI models
        let cost = match model {
            "gpt-35-turbo" => {
                (input_tokens as f64 * 0.0015 + output_tokens as f64 * 0.002) / 1000.0
            }
            "gpt-4" => (input_tokens as f64 * 0.03 + output_tokens as f64 * 0.06) / 1000.0,
            "gpt-4-turbo" => (input_tokens as f64 * 0.01 + output_tokens as f64 * 0.03) / 1000.0,
            _ => 0.0,
        };
        Ok(cost)
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        self.chat_handler
            .create_chat_completion(request, context)
            .await
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        self.chat_handler
            .create_chat_completion_stream(request, context)
            .await
    }

    async fn embeddings(
        &self,
        request: EmbeddingRequest,
        context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        self.embedding_handler
            .create_embeddings(request, context)
            .await
    }

    async fn image_generation(
        &self,
        request: ImageGenerationRequest,
        context: RequestContext,
    ) -> Result<ImageGenerationResponse, Self::Error> {
        self.image_handler.generate_image(request, context).await
    }

    async fn health_check(&self) -> HealthStatus {
        if self.config.api_key.is_some() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }
}

// ProviderConfig implementation is in common_utils.rs

/// Azure provider factory
pub struct AzureProviderFactory;

impl AzureProviderFactory {
    /// Create provider with default configuration
    pub fn create_default() -> Result<AzureOpenAIProvider, AzureError> {
        let config = AzureConfig::new();
        AzureOpenAIProvider::new(config)
    }

    /// Create provider with custom configuration
    pub fn create_with_config(config: AzureConfig) -> Result<AzureOpenAIProvider, AzureError> {
        AzureOpenAIProvider::new(config)
    }

    /// Create provider from environment variables
    pub fn create_from_env() -> Result<AzureOpenAIProvider, AzureError> {
        AzureOpenAIProvider::from_env()
    }
}
