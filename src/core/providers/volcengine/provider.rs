//! Volcengine Provider Implementation
//!
//! Main provider implementation for ByteDance's Volcengine AI platform

use std::sync::Arc;

use crate::core::providers::base::{GlobalPoolManager, HeaderPair, header, header_owned};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::ProviderConfig;
use crate::core::types::model::ModelInfo;

use super::{VolcengineClient, VolcengineConfig};

/// Volcengine provider for ByteDance's cloud AI platform
#[derive(Debug, Clone)]
pub struct VolcengineProvider {
    config: VolcengineConfig,
    pool_manager: Arc<GlobalPoolManager>,
    supported_models: Vec<ModelInfo>,
}

impl VolcengineProvider {
    /// Generate headers for Volcengine API requests
    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(2);

        if let Some(api_key) = &self.config.base.api_key {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        // Add custom headers
        for (key, value) in &self.config.base.headers {
            headers.push(header_owned(key.clone(), value.clone()));
        }

        headers
    }

    /// Create new Volcengine provider
    pub fn new(config: VolcengineConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("volcengine", e))?;

        let pool_manager = Arc::new(
            GlobalPoolManager::new()
                .map_err(|e| ProviderError::configuration("volcengine", e.to_string()))?,
        );
        let supported_models = VolcengineClient::supported_models();

        Ok(Self {
            config,
            pool_manager,
            supported_models,
        })
    }

    /// Create provider from environment variables
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = VolcengineConfig::from_env();
        Self::new(config)
    }
}
