//! Model Configuration for Bedrock Models
//!
//! Defines model families, capabilities, and routing configuration
//! for all supported Bedrock models.

use crate::core::providers::unified_provider::ProviderError;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Bedrock model families
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BedrockModelFamily {
    Claude,
    TitanText,
    TitanEmbedding,
    TitanImage,
    Nova,
    Llama,
    Mistral,
    AI21,
    Cohere,
    DeepSeek,
    StabilityAI,
}

/// Bedrock API types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BedrockApiType {
    Invoke,
    Converse,
    InvokeStream,
    ConverseStream,
}

/// Model configuration for routing and capabilities
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub family: BedrockModelFamily,
    pub api_type: BedrockApiType,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub supports_multimodal: bool,
    pub max_context_length: u32,
    pub max_output_length: Option<u32>,
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
}

/// Model configuration database
mod model_config_catalog;

use model_config_catalog::build_model_configs;

/// Model configuration database
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelConfig>> =
    LazyLock::new(build_model_configs);

/// Get model configuration for a specific model ID
pub fn get_model_config(model_id: &str) -> Result<&'static ModelConfig, ProviderError> {
    MODEL_CONFIGS.get(model_id).ok_or_else(|| {
        ProviderError::model_not_found("bedrock", format!("Model {} not supported", model_id))
    })
}

/// Check if a model supports a specific capability
pub fn model_supports_capability(model_id: &str, capability: &str) -> bool {
    if let Ok(config) = get_model_config(model_id) {
        match capability {
            "streaming" => config.supports_streaming,
            "function_calling" => config.supports_function_calling,
            "multimodal" => config.supports_multimodal,
            _ => false,
        }
    } else {
        false
    }
}

/// Get all supported model IDs
pub fn get_all_model_ids() -> Vec<&'static str> {
    MODEL_CONFIGS.keys().copied().collect()
}

#[cfg(test)]
#[path = "model_config_tests.rs"]
mod tests;
