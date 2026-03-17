//! Tavily Provider
//!
//! Tavily AI search API integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::providers::base::{BaseConfig, BaseHttpClient, HttpErrorMapper};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{error_mapper::trait_def::ErrorMapper, provider::ProviderConfig};

/// Tavily configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TavilyConfig {
    /// API key for Tavily
    pub api_key: Option<String>,
    /// API base URL (default: <https://api.tavily.com>)
    pub api_base: Option<String>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Max retries
    pub max_retries: u32,
}

impl Default for TavilyConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: Some("https://api.tavily.com".to_string()),
            timeout: 30,
            max_retries: 3,
        }
    }
}

impl TavilyConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, TavilyError> {
        let api_key = std::env::var("TAVILY_API_KEY").ok();

        let api_base = std::env::var("TAVILY_API_BASE")
            .unwrap_or_else(|_| "https://api.tavily.com".to_string());

        Ok(Self {
            api_key,
            api_base: Some(api_base),
            timeout: 30,
            max_retries: 3,
        })
    }

    /// Get effective API base URL
    pub fn get_effective_api_base(&self) -> &str {
        self.api_base.as_deref().unwrap_or("https://api.tavily.com")
    }
}

/// Tavily error type (alias to unified ProviderError)
pub type TavilyError = ProviderError;

/// Tavily provider
#[derive(Debug, Clone)]
pub struct TavilyProvider {
    config: TavilyConfig,
}

impl TavilyProvider {
    /// Create new Tavily provider
    pub fn new(config: TavilyConfig) -> Result<Self, TavilyError> {
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
            .map_err(|e| ProviderError::configuration("tavily", e.to_string()))?;

        Ok(Self { config })
    }
}

/// Tavily error mapper
#[derive(Debug)]
pub struct TavilyErrorMapper;

impl ErrorMapper<TavilyError> for TavilyErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> TavilyError {
        HttpErrorMapper::map_status_code("tavily", status_code, response_body)
    }
}

impl ProviderConfig for TavilyConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_none() {
            return Err("Tavily API key is required".to_string());
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
