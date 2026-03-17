//! iFlytek Spark Provider Implementation
//!
//! Implementation of LLMProvider for Spark with WebSocket support

use std::collections::HashMap;

use crate::core::providers::base::{BaseConfig, BaseHttpClient, HttpErrorMapper};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{error_mapper::trait_def::ErrorMapper, provider::ProviderConfig};
use crate::core::types::{chat::ChatRequest, model::ModelInfo};

use super::config::SparkConfig;
use super::model_info::{ModelFeature, get_spark_registry};

/// iFlytek Spark provider
#[derive(Debug, Clone)]
pub struct SparkProvider {
    config: SparkConfig,
    supported_models: Vec<ModelInfo>,
}

impl SparkProvider {
    /// Create new Spark provider
    pub fn new(config: SparkConfig) -> Result<Self, ProviderError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("spark", e))?;

        let base_config = BaseConfig {
            api_key: config.api_key.clone(),
            api_base: Some(config.api_base.clone()),
            timeout: config.request_timeout,
            max_retries: config.max_retries,
            headers: HashMap::new(),
            organization: None,
            api_version: None,
        };

        let _base_client = BaseHttpClient::new(base_config)
            .map_err(|e| ProviderError::configuration("spark", e.to_string()))?;

        // Get supported models from registry
        let registry = get_spark_registry();
        let supported_models = registry
            .list_models()
            .into_iter()
            .map(|spec| spec.model_info.clone())
            .collect();

        Ok(Self {
            config,
            supported_models,
        })
    }

    /// Validate request
    fn validate_request(&self, request: &ChatRequest) -> Result<(), ProviderError> {
        let registry = get_spark_registry();

        let model_spec = registry.get_model_spec(&request.model).ok_or_else(|| {
            ProviderError::invalid_request("spark", format!("Unsupported model: {}", request.model))
        })?;

        // Common validation: empty messages + max_tokens
        crate::core::providers::base::validate_chat_request_common(
            "spark",
            request,
            model_spec.limits.max_output_tokens,
        )?;

        // Check function calling support
        if request.tools.is_some() && !model_spec.features.contains(&ModelFeature::FunctionCalling)
        {
            return Err(ProviderError::not_supported(
                "spark",
                format!("Model {} does not support function calling", request.model),
            ));
        }

        Ok(())
    }
}

/// Spark error mapper
#[derive(Debug)]
pub struct SparkErrorMapper;

impl ErrorMapper<ProviderError> for SparkErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        HttpErrorMapper::map_status_code("spark", status_code, response_body)
    }
}

/// Provider builder
pub struct SparkProviderBuilder {
    config: Option<SparkConfig>,
}

impl SparkProviderBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Set configuration
    pub fn with_config(mut self, config: SparkConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set app ID
    pub fn with_app_id(mut self, app_id: impl Into<String>) -> Self {
        if let Some(ref mut config) = self.config {
            config.app_id = Some(app_id.into());
        } else {
            self.config = Some(SparkConfig {
                app_id: Some(app_id.into()),
                ..SparkConfig::default()
            });
        }
        self
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        if let Some(ref mut config) = self.config {
            config.api_key = Some(api_key.into());
        } else {
            self.config = Some(SparkConfig {
                api_key: Some(api_key.into()),
                ..SparkConfig::default()
            });
        }
        self
    }

    /// Set API secret
    pub fn with_api_secret(mut self, api_secret: impl Into<String>) -> Self {
        if let Some(ref mut config) = self.config {
            config.api_secret = Some(api_secret.into());
        } else {
            self.config = Some(SparkConfig {
                api_secret: Some(api_secret.into()),
                ..SparkConfig::default()
            });
        }
        self
    }

    /// Build provider
    pub fn build(self) -> Result<SparkProvider, ProviderError> {
        let config = self
            .config
            .ok_or_else(|| ProviderError::configuration("spark", "Configuration is required"))?;

        SparkProvider::new(config)
    }
}

impl Default for SparkProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}
