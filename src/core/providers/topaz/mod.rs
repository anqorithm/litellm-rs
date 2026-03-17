//! Topaz Provider
//!
//! Topaz AI platform integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::providers::base::{BaseConfig, BaseHttpClient, HttpErrorMapper};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{error_mapper::trait_def::ErrorMapper, provider::ProviderConfig};

/// Topaz configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopazConfig {
    /// API key for Topaz
    pub api_key: Option<String>,
    /// API base URL (default: <https://api.topaz.com>)
    pub api_base: Option<String>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Max retries
    pub max_retries: u32,
}

impl Default for TopazConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: Some("https://api.topaz.com".to_string()),
            timeout: 60,
            max_retries: 3,
        }
    }
}

impl TopazConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, TopazError> {
        let api_key = std::env::var("TOPAZ_API_KEY").ok();

        let api_base =
            std::env::var("TOPAZ_API_BASE").unwrap_or_else(|_| "https://api.topaz.com".to_string());

        Ok(Self {
            api_key,
            api_base: Some(api_base),
            timeout: 60,
            max_retries: 3,
        })
    }

    /// Get effective API base URL
    pub fn get_effective_api_base(&self) -> &str {
        self.api_base.as_deref().unwrap_or("https://api.topaz.com")
    }
}

/// Topaz error type (alias to unified ProviderError)
pub type TopazError = ProviderError;

/// Topaz provider
#[derive(Debug, Clone)]
pub struct TopazProvider {
    config: TopazConfig,
}

impl TopazProvider {
    /// Create new Topaz provider
    pub fn new(config: TopazConfig) -> Result<Self, TopazError> {
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
            .map_err(|e| ProviderError::configuration("topaz", e.to_string()))?;

        Ok(Self { config })
    }
}

/// Topaz error mapper
#[derive(Debug)]
pub struct TopazErrorMapper;

impl ErrorMapper<TopazError> for TopazErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> TopazError {
        HttpErrorMapper::map_status_code("topaz", status_code, response_body)
    }
}

impl ProviderConfig for TopazConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_none() {
            return Err("Topaz API key is required".to_string());
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
