//! SearXNG Provider
//!
//! SearXNG meta search engine integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::providers::base::{BaseConfig, BaseHttpClient, HttpErrorMapper};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{error_mapper::trait_def::ErrorMapper, provider::ProviderConfig};

/// SearXNG configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearXNGConfig {
    /// API key for SearXNG (optional)
    pub api_key: Option<String>,
    /// API base URL (required, no default)
    pub api_base: Option<String>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Max retries
    pub max_retries: u32,
}

impl Default for SearXNGConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: None,
            timeout: 30,
            max_retries: 3,
        }
    }
}

impl SearXNGConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, SearXNGError> {
        let api_key = std::env::var("SEARXNG_API_KEY").ok();
        let api_base = std::env::var("SEARXNG_API_BASE").ok();

        Ok(Self {
            api_key,
            api_base,
            timeout: 30,
            max_retries: 3,
        })
    }

    /// Get effective API base URL
    pub fn get_effective_api_base(&self) -> Result<&str, SearXNGError> {
        self.api_base
            .as_deref()
            .ok_or_else(|| ProviderError::configuration("searxng", "API base URL is required"))
    }
}

/// SearXNG error type (alias to unified ProviderError)
pub type SearXNGError = ProviderError;

/// SearXNG provider
#[derive(Debug, Clone)]
pub struct SearXNGProvider {
    config: SearXNGConfig,
}

impl SearXNGProvider {
    /// Create new SearXNG provider
    pub fn new(config: SearXNGConfig) -> Result<Self, SearXNGError> {
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
            .map_err(|e| ProviderError::configuration("searxng", e.to_string()))?;

        Ok(Self { config })
    }
}

/// SearXNG error mapper
#[derive(Debug)]
pub struct SearXNGErrorMapper;

impl ErrorMapper<SearXNGError> for SearXNGErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> SearXNGError {
        HttpErrorMapper::map_status_code("searxng", status_code, response_body)
    }
}

impl ProviderConfig for SearXNGConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_base.is_none() {
            return Err("SearXNG API base URL is required".to_string());
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
