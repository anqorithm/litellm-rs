//! ExaAi Provider Implementation

use std::sync::Arc;

use crate::core::providers::base::{GlobalPoolManager, HeaderPair, header, header_owned};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::ProviderConfig;
use crate::core::types::model::ModelInfo;

use super::{ExaAiClient, ExaAiConfig};

#[derive(Debug, Clone)]
pub struct ExaAiProvider {
    config: ExaAiConfig,
    pool_manager: Arc<GlobalPoolManager>,
    supported_models: Vec<ModelInfo>,
}

impl ExaAiProvider {
    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(2);

        if let Some(api_key) = &self.config.base.api_key {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        for (key, value) in &self.config.base.headers {
            headers.push(header_owned(key.clone(), value.clone()));
        }

        headers
    }

    pub fn new(config: ExaAiConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("exa_ai", e))?;

        let pool_manager = Arc::new(
            GlobalPoolManager::new()
                .map_err(|e| ProviderError::configuration("exa_ai", e.to_string()))?,
        );
        let supported_models = ExaAiClient::supported_models();

        Ok(Self {
            config,
            pool_manager,
            supported_models,
        })
    }

    pub fn from_env() -> Result<Self, ProviderError> {
        let config = ExaAiConfig::from_env();
        Self::new(config)
    }

    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let mut config = ExaAiConfig::new("exa_ai");
        config.base.api_key = Some(api_key.into());
        Self::new(config)
    }
}
