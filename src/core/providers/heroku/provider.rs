//! Heroku Provider Implementation
//!
//! Main provider implementation for Heroku AI Inference API.
//! Heroku provides managed access to various AI models including Claude, Amazon Nova, and more.

use std::sync::Arc;

use crate::core::providers::base::{GlobalPoolManager, HeaderPair, header, header_owned};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::ProviderConfig;
use crate::core::types::model::ModelInfo;

use super::HerokuClient;
use super::config::{DEFAULT_API_BASE, HerokuConfig, PROVIDER_NAME};

/// Heroku AI Inference Provider
///
/// Provides access to AI models through Heroku's managed inference service,
/// which is part of the Salesforce ecosystem.
#[derive(Debug, Clone)]
pub struct HerokuProvider {
    config: HerokuConfig,
    pool_manager: Arc<GlobalPoolManager>,
    supported_models: Vec<ModelInfo>,
}

impl HerokuProvider {
    /// Generate headers for Heroku API requests
    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(3);

        if let Some(api_key) = &self.config.base.api_key {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        headers.push(header("Content-Type", "application/json".to_string()));

        // Add custom headers
        for (key, value) in &self.config.base.headers {
            headers.push(header_owned(key.clone(), value.clone()));
        }

        headers
    }

    /// Get the effective API base URL
    fn get_api_base(&self) -> String {
        self.config
            .base
            .api_base
            .clone()
            .or_else(|| std::env::var("INFERENCE_URL").ok())
            .unwrap_or_else(|| DEFAULT_API_BASE.to_string())
    }

    /// Create a new Heroku provider with the given configuration
    pub fn new(config: HerokuConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration(PROVIDER_NAME, e))?;

        let pool_manager = Arc::new(
            GlobalPoolManager::new()
                .map_err(|e| ProviderError::configuration(PROVIDER_NAME, e.to_string()))?,
        );
        let supported_models = HerokuClient::supported_models();

        Ok(Self {
            config,
            pool_manager,
            supported_models,
        })
    }

    /// Create provider from environment variables
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = HerokuConfig::from_env();
        Self::new(config)
    }

    /// Create provider with API key
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = HerokuConfig::with_api_key(api_key);
        Self::new(config)
    }

    /// Create provider with API key and custom API base
    pub async fn with_api_key_and_base(
        api_key: impl Into<String>,
        api_base: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let config = HerokuConfig::with_api_key(api_key).with_api_base(api_base);
        Self::new(config)
    }
}
