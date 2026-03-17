//! GigaChat Provider
//!
//! GigaChat (Sber) AI model integration with custom authentication.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::core::providers::base::{BaseConfig, BaseHttpClient, HttpErrorMapper};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    error_mapper::trait_def::ErrorMapper, provider::ProviderConfig,
    provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{model::ModelInfo, model::ProviderCapability};

// Static capabilities
const GIGACHAT_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::Embeddings,
];

/// GigaChat provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GigaChatConfig {
    /// API key (credentials) for authentication
    pub api_key: String,
    /// API base URL (defaults to <https://gigachat.devices.sberbank.ru/api/v1>)
    pub api_base: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
    /// Scope for OAuth (defaults to GIGACHAT_API_PERS)
    pub scope: String,
}

impl Default for GigaChatConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: "https://gigachat.devices.sberbank.ru/api/v1".to_string(),
            timeout_seconds: 60,
            max_retries: 3,
            scope: "GIGACHAT_API_PERS".to_string(),
        }
    }
}

impl ProviderConfig for GigaChatConfig {
    fn validate(&self) -> Result<(), String> {
        self.validate_standard("GigaChat")
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

/// GigaChat error type (using unified ProviderError)
pub type GigaChatError = ProviderError;

/// GigaChat error mapper
pub struct GigaChatErrorMapper;

impl ErrorMapper<GigaChatError> for GigaChatErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> GigaChatError {
        HttpErrorMapper::map_status_code("gigachat", status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> GigaChatError {
        HttpErrorMapper::parse_json_error("gigachat", error_response)
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> GigaChatError {
        ProviderError::network("gigachat", error.to_string())
    }

    fn map_parsing_error(&self, error: &dyn std::error::Error) -> GigaChatError {
        ProviderError::response_parsing("gigachat", error.to_string())
    }

    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> GigaChatError {
        ProviderError::timeout(
            "gigachat",
            format!("Request timed out after {:?}", timeout_duration),
        )
    }
}

/// GigaChat provider implementation
#[derive(Debug, Clone)]
pub struct GigaChatProvider {
    config: GigaChatConfig,
    base_client: BaseHttpClient,
    models: Vec<ModelInfo>,
}

impl GigaChatProvider {
    /// Create a new GigaChat provider instance
    pub async fn new(config: GigaChatConfig) -> Result<Self, GigaChatError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("gigachat", e))?;

        let base_config = BaseConfig {
            api_key: Some(config.api_key.clone()),
            api_base: Some(config.api_base.clone()),
            timeout: config.timeout_seconds,
            max_retries: config.max_retries,
            headers: HashMap::new(),
            organization: None,
            api_version: None,
        };

        let base_client = BaseHttpClient::new(base_config)?;

        let models = vec![
            ModelInfo {
                id: "GigaChat".to_string(),
                name: "GigaChat".to_string(),
                provider: "gigachat".to_string(),
                max_context_length: 8192,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0),
                output_cost_per_1k_tokens: Some(0.0),
                currency: "RUB".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "GigaChat-Pro".to_string(),
                name: "GigaChat Pro".to_string(),
                provider: "gigachat".to_string(),
                max_context_length: 8192,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0),
                output_cost_per_1k_tokens: Some(0.0),
                currency: "RUB".to_string(),
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

    fn is_embedding_model(&self, model: &str) -> bool {
        model.contains("embed") || model.contains("Embeddings")
    }
}
