//! Gemini Provider Implementation
//!
//! Implementation

use std::sync::Arc;

use crate::core::providers::base::GlobalPoolManager;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    provider::ProviderConfig, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{chat::ChatRequest, model::ModelInfo};

use super::client::GeminiClient;
use super::config::GeminiConfig;
use super::error::{gemini_model_error, gemini_validation_error};
use super::models::{ModelFeature, get_gemini_registry};

/// Gemini Provider - Unified implementation
#[derive(Debug)]
pub struct GeminiProvider {
    client: GeminiClient,
    supported_models: Vec<ModelInfo>,
}

impl GeminiProvider {
    /// Create
    pub fn new(config: GeminiConfig) -> Result<Self, ProviderError> {
        // Configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("gemini", e))?;

        // Create
        let client = GeminiClient::new(config.clone())?;

        // Get
        let _pool_manager = Arc::new(GlobalPoolManager::new()?);

        // Get
        let registry = get_gemini_registry();
        let supported_models = registry
            .list_models()
            .into_iter()
            .map(|spec| spec.model_info.clone())
            .collect();

        Ok(Self {
            client,
            supported_models,
        })
    }

    /// Request
    fn validate_request(&self, request: &ChatRequest) -> Result<(), ProviderError> {
        let registry = get_gemini_registry();

        let model_spec = registry
            .get_model_spec(&request.model)
            .ok_or_else(|| gemini_model_error(format!("Unsupported model: {}", request.model)))?;

        // Common validation: empty messages + max_tokens
        crate::core::providers::base::validate_chat_request_common(
            "gemini",
            request,
            model_spec.limits.max_output_tokens,
        )?;

        // Check temperature range
        if let Some(temperature) = request.temperature
            && !(0.0..=2.0).contains(&temperature)
        {
            return Err(gemini_validation_error(
                "temperature must be between 0.0 and 2.0",
            ));
        }

        // Check top_p range
        if let Some(top_p) = request.top_p
            && !(0.0..=1.0).contains(&top_p)
        {
            return Err(gemini_validation_error("top_p must be between 0.0 and 1.0"));
        }

        // Check tool calling support
        if request.tools.is_some() && !model_spec.features.contains(&ModelFeature::ToolCalling) {
            return Err(gemini_validation_error(format!(
                "Model {} does not support tool calling",
                request.model
            )));
        }

        Ok(())
    }

    /// Get
    pub fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Option<f64> {
        super::models::CostCalculator::calculate_cost(model, input_tokens, output_tokens)
    }
}

// GeminiError is a type alias for ProviderError, so we don't need to implement traits for it
// The error mapping is handled by GeminiErrorMapper in error.rs
