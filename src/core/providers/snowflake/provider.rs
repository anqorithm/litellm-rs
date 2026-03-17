//! Snowflake Cortex AI Provider Implementation
//!
//! Implements the LLMProvider trait for Snowflake Cortex AI.

use std::collections::HashMap;
use std::sync::Arc;

use super::config::SnowflakeConfig;
use super::error::SnowflakeError;
use super::model_info::get_available_models;
use crate::core::providers::base::GlobalPoolManager;
use crate::core::traits::provider::ProviderConfig as _;
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::{model::ModelInfo, model::ProviderCapability};

/// Static capabilities for Snowflake provider
const SNOWFLAKE_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
];

/// Snowflake Cortex AI provider implementation
#[derive(Debug, Clone)]
pub struct SnowflakeProvider {
    config: SnowflakeConfig,
    #[allow(dead_code)]
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl SnowflakeProvider {
    /// Create a new Snowflake provider instance
    pub async fn new(config: SnowflakeConfig) -> Result<Self, SnowflakeError> {
        config
            .validate()
            .map_err(|e| SnowflakeError::configuration("snowflake", e))?;

        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            SnowflakeError::configuration(
                "snowflake",
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        // Build model list from static configuration
        let models = get_available_models()
            .iter()
            .map(|info| {
                let mut capabilities = vec![
                    ProviderCapability::ChatCompletion,
                    ProviderCapability::ChatCompletionStream,
                ];
                if info.supports_tools {
                    capabilities.push(ProviderCapability::ToolCalling);
                }

                ModelInfo {
                    id: info.model_id.to_string(),
                    name: info.display_name.to_string(),
                    provider: "snowflake".to_string(),
                    max_context_length: info.max_context_length as u32,
                    max_output_length: Some(info.max_output_length as u32),
                    supports_streaming: true,
                    supports_tools: info.supports_tools,
                    supports_multimodal: false,
                    input_cost_per_1k_tokens: None, // Snowflake pricing varies by region
                    output_cost_per_1k_tokens: None,
                    currency: "USD".to_string(),
                    capabilities,
                    created_at: None,
                    updated_at: None,
                    metadata: HashMap::new(),
                }
            })
            .collect();

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    /// Create provider with API key and account ID
    pub async fn with_api_key(
        api_key: impl Into<String>,
        account_id: impl Into<String>,
    ) -> Result<Self, SnowflakeError> {
        let config = SnowflakeConfig {
            api_key: Some(api_key.into()),
            account_id: Some(account_id.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Get the API base URL
    fn get_api_base(&self) -> String {
        if let Some(base) = &self.config.api_base {
            base.clone()
        } else if let Some(account_id) = &self.config.account_id {
            format!("https://{}.snowflakecomputing.com/api/v2", account_id)
        } else {
            std::env::var("SNOWFLAKE_ACCOUNT_ID")
                .map(|id| format!("https://{}.snowflakecomputing.com/api/v2", id))
                .unwrap_or_else(|_| "https://snowflakecomputing.com/api/v2".to_string())
        }
    }

    /// Get the API key (JWT or PAT)
    fn get_api_key(&self) -> Option<String> {
        self.config
            .api_key
            .clone()
            .or_else(|| std::env::var("SNOWFLAKE_JWT").ok())
    }
}
