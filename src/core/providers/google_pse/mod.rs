//! Google PSE (Programmable Search Engine) Provider
//!
//! Google Programmable Search Engine integration for search-augmented generation.

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
const GOOGLE_PSE_CAPABILITIES: &[ProviderCapability] = &[ProviderCapability::ChatCompletion];

/// Google PSE provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooglePSEConfig {
    /// API key for authentication
    pub api_key: String,
    /// Search Engine ID
    pub search_engine_id: String,
    /// API base URL (defaults to <https://www.googleapis.com/customsearch/v1>)
    pub api_base: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
}

impl Default for GooglePSEConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            search_engine_id: String::new(),
            api_base: "https://www.googleapis.com/customsearch/v1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

impl ProviderConfig for GooglePSEConfig {
    fn validate(&self) -> Result<(), String> {
        self.validate_standard("Google PSE")?;
        if self.search_engine_id.is_empty() {
            return Err("Google PSE Search Engine ID is required".to_string());
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

/// Google PSE error type (using unified ProviderError)
pub type GooglePSEError = ProviderError;

/// Google PSE error mapper
pub struct GooglePSEErrorMapper;

impl ErrorMapper<GooglePSEError> for GooglePSEErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> GooglePSEError {
        HttpErrorMapper::map_status_code("google_pse", status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> GooglePSEError {
        HttpErrorMapper::parse_json_error("google_pse", error_response)
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> GooglePSEError {
        ProviderError::network("google_pse", error.to_string())
    }

    fn map_parsing_error(&self, error: &dyn std::error::Error) -> GooglePSEError {
        ProviderError::response_parsing("google_pse", error.to_string())
    }

    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> GooglePSEError {
        ProviderError::timeout(
            "google_pse",
            format!("Request timed out after {:?}", timeout_duration),
        )
    }
}

/// Google PSE provider implementation
#[derive(Debug, Clone)]
pub struct GooglePSEProvider {
    config: GooglePSEConfig,
    base_client: BaseHttpClient,
    models: Vec<ModelInfo>,
}

impl GooglePSEProvider {
    /// Create a new Google PSE provider instance
    pub async fn new(config: GooglePSEConfig) -> Result<Self, GooglePSEError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("google_pse", e))?;

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

        let models = vec![ModelInfo {
            id: "google-pse-search".to_string(),
            name: "Google PSE Search".to_string(),
            provider: "google_pse".to_string(),
            max_context_length: 1024,
            max_output_length: None,
            supports_streaming: false,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k_tokens: Some(0.005),
            output_cost_per_1k_tokens: Some(0.0),
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        }];

        Ok(Self {
            config,
            base_client,
            models,
        })
    }
}
