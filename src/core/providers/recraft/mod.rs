//! Recraft Provider
//!
//! Recraft AI image generation platform integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::providers::base::{BaseConfig, BaseHttpClient, HttpErrorMapper};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{error_mapper::trait_def::ErrorMapper, provider::ProviderConfig};

/// Recraft configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecraftConfig {
    /// API key for Recraft
    pub api_key: Option<String>,
    /// API base URL (default: <https://api.recraft.ai>)
    pub api_base: Option<String>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Max retries
    pub max_retries: u32,
}

impl Default for RecraftConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: Some("https://api.recraft.ai".to_string()),
            timeout: 60,
            max_retries: 3,
        }
    }
}

impl RecraftConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, RecraftError> {
        let api_key = std::env::var("RECRAFT_API_KEY").ok();

        let api_base = std::env::var("RECRAFT_API_BASE")
            .unwrap_or_else(|_| "https://api.recraft.ai".to_string());

        Ok(Self {
            api_key,
            api_base: Some(api_base),
            timeout: 60,
            max_retries: 3,
        })
    }

    /// Get effective API base URL
    pub fn get_effective_api_base(&self) -> &str {
        self.api_base.as_deref().unwrap_or("https://api.recraft.ai")
    }
}

/// Recraft error type (alias to unified ProviderError)
pub type RecraftError = ProviderError;

/// Recraft provider
#[derive(Debug, Clone)]
pub struct RecraftProvider {
    config: RecraftConfig,
}

impl RecraftProvider {
    /// Create new Recraft provider
    pub fn new(config: RecraftConfig) -> Result<Self, RecraftError> {
        let base_config = BaseConfig {
            api_key: config.api_key.clone(),
            api_base: config.api_base.clone(),
            timeout: config.timeout,
            max_retries: config.max_retries,
            headers: HashMap::new(),
            organization: None,
            api_version: None,
        };

        let _base_client = BaseHttpClient::new(base_config)
            .map_err(|e| ProviderError::configuration("recraft", e.to_string()))?;

        Ok(Self { config })
    }
}

/// Recraft error mapper
#[derive(Debug)]
pub struct RecraftErrorMapper;

impl ErrorMapper<RecraftError> for RecraftErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> RecraftError {
        HttpErrorMapper::map_status_code("recraft", status_code, response_body)
    }
}

impl ProviderConfig for RecraftConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_none() {
            return Err("Recraft API key is required".to_string());
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
