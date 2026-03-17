//! Main Gradient AI Provider Implementation
//!
//! Implements the LLMProvider trait for Gradient AI's agent and model platform.

use std::collections::HashMap;
use std::sync::Arc;

use super::config::GradientAIConfig;
use crate::core::providers::base::{GlobalPoolManager, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    provider::ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{chat::ChatRequest, model::ModelInfo, model::ProviderCapability};

/// Static capabilities for Gradient AI provider
const GRADIENT_AI_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
];

/// Supported OpenAI parameters for Gradient AI
const SUPPORTED_OPENAI_PARAMS: &[&str] = &[
    "frequency_penalty",
    "max_tokens",
    "max_completion_tokens",
    "presence_penalty",
    "stop",
    "stream",
    "stream_options",
    "temperature",
    "top_p",
    // Gradient AI specific parameters
    "k",
    "kb_filters",
    "filter_kb_content_by_query_metadata",
    "instruction_override",
    "include_functions_info",
    "include_retrieval_info",
    "include_guardrails_info",
    "provide_citations",
    "retrieval_method",
];

/// Gradient AI provider implementation
#[derive(Debug, Clone)]
pub struct GradientAIProvider {
    config: GradientAIConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl GradientAIProvider {
    /// Create a new Gradient AI provider instance
    pub async fn new(config: GradientAIConfig) -> Result<Self, ProviderError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("gradient_ai", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration(
                "gradient_ai",
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        // Build default model list
        let models = vec![ModelInfo {
            id: "gradient-ai-agent".to_string(),
            name: "Gradient AI Agent".to_string(),
            provider: "gradient_ai".to_string(),
            max_context_length: 128000,
            max_output_length: Some(4096),
            supports_streaming: true,
            supports_tools: false, // Gradient AI uses KB-based retrieval instead of tool calling
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![
                ProviderCapability::ChatCompletion,
                ProviderCapability::ChatCompletionStream,
            ],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        }];

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    /// Create provider with API key only
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = GradientAIConfig {
            api_key: Some(api_key.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Build request body with Gradient AI specific parameters
    fn build_request_body(&self, request: &ChatRequest) -> serde_json::Value {
        let mut body = serde_json::to_value(request).unwrap_or_default();

        // Add Gradient AI specific parameters from config
        if let Some(k) = self.config.k {
            body["k"] = serde_json::json!(k);
        }
        if let Some(ref kb_filters) = self.config.kb_filters {
            body["kb_filters"] = serde_json::json!(kb_filters);
        }
        if let Some(filter) = self.config.filter_kb_content_by_query_metadata {
            body["filter_kb_content_by_query_metadata"] = serde_json::json!(filter);
        }
        if let Some(ref instruction) = self.config.instruction_override {
            body["instruction_override"] = serde_json::json!(instruction);
        }
        if let Some(include) = self.config.include_functions_info {
            body["include_functions_info"] = serde_json::json!(include);
        }
        if let Some(include) = self.config.include_retrieval_info {
            body["include_retrieval_info"] = serde_json::json!(include);
        }
        if let Some(include) = self.config.include_guardrails_info {
            body["include_guardrails_info"] = serde_json::json!(include);
        }
        if let Some(provide) = self.config.provide_citations {
            body["provide_citations"] = serde_json::json!(provide);
        }
        if let Some(ref method) = self.config.retrieval_method {
            body["retrieval_method"] = serde_json::json!(method);
        }

        body
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError> {
        let mut headers = Vec::with_capacity(2);
        if let Some(api_key) = &self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));

        let response = self
            .pool_manager
            .execute_request(url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| ProviderError::network("gradient_ai", e.to_string()))?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("gradient_ai", e.to_string()))?;

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::api_error(
                "gradient_ai",
                500,
                format!("Failed to parse response: {}", e),
            )
        })
    }
}
