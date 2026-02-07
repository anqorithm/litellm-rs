//! OpenAI-Like Provider Model Support
//!
//! Dynamic model support - accepts any model name and passes it through

use crate::core::types::{ModelInfo, ProviderCapability};
use std::collections::HashMap;

/// OpenAI-like model registry
///
/// Unlike other providers, this registry accepts ANY model name
/// and creates dynamic model info on the fly.
#[derive(Debug, Clone)]
pub struct OpenAILikeModelRegistry {
    /// Known model configurations (optional, for optimization)
    known_models: HashMap<String, OpenAILikeModelConfig>,
    /// Default context length for unknown models
    default_context_length: u32,
    /// Default output length for unknown models
    default_output_length: u32,
}

/// Configuration for a known model
#[derive(Debug, Clone)]
pub struct OpenAILikeModelConfig {
    /// Model ID
    pub id: String,
    /// Maximum context length
    pub max_context_length: u32,
    /// Maximum output length
    pub max_output_length: Option<u32>,
    /// Whether the model supports streaming
    pub supports_streaming: bool,
    /// Whether the model supports tools/function calling
    pub supports_tools: bool,
    /// Whether the model supports multimodal input
    pub supports_multimodal: bool,
    /// Input cost per 1k tokens (optional)
    pub input_cost_per_1k: Option<f64>,
    /// Output cost per 1k tokens (optional)
    pub output_cost_per_1k: Option<f64>,
}

impl Default for OpenAILikeModelConfig {
    fn default() -> Self {
        Self {
            id: "unknown".to_string(),
            max_context_length: 4096,
            max_output_length: Some(4096),
            supports_streaming: true,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k: None,
            output_cost_per_1k: None,
        }
    }
}

impl Default for OpenAILikeModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAILikeModelRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            known_models: HashMap::new(),
            default_context_length: 4096,
            default_output_length: 4096,
        }
    }

    /// Create a registry with common model defaults
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.default_context_length = 128000; // Most modern models support large contexts
        registry.default_output_length = 4096;
        registry
    }

    /// Set default context length for unknown models
    pub fn with_default_context_length(mut self, length: u32) -> Self {
        self.default_context_length = length;
        self
    }

    /// Set default output length for unknown models
    pub fn with_default_output_length(mut self, length: u32) -> Self {
        self.default_output_length = length;
        self
    }

    /// Register a known model with specific configuration
    pub fn register_model(&mut self, config: OpenAILikeModelConfig) {
        self.known_models.insert(config.id.clone(), config);
    }

    /// Get model info for any model name
    ///
    /// If the model is known, returns its specific configuration.
    /// Otherwise, returns default configuration that allows the request to proceed.
    pub fn get_model_info(&self, model_id: &str) -> ModelInfo {
        if let Some(config) = self.known_models.get(model_id) {
            ModelInfo {
                id: config.id.clone(),
                name: config.id.clone(),
                provider: "openai_like".to_string(),
                max_context_length: config.max_context_length,
                max_output_length: config.max_output_length,
                supports_streaming: config.supports_streaming,
                supports_tools: config.supports_tools,
                supports_multimodal: config.supports_multimodal,
                capabilities: self.build_capabilities(config),
                input_cost_per_1k_tokens: config.input_cost_per_1k,
                output_cost_per_1k_tokens: config.output_cost_per_1k,
                currency: "USD".to_string(),
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            }
        } else {
            // Return default info for unknown models
            // This allows any model name to be passed through to the API
            self.create_default_model_info(model_id)
        }
    }

    /// Create default model info for an unknown model
    fn create_default_model_info(&self, model_id: &str) -> ModelInfo {
        ModelInfo {
            id: model_id.to_string(),
            name: model_id.to_string(),
            provider: "openai_like".to_string(),
            max_context_length: self.default_context_length,
            max_output_length: Some(self.default_output_length),
            supports_streaming: true, // Assume streaming is supported
            supports_tools: true,     // Assume tools are supported
            supports_multimodal: false,
            capabilities: vec![
                ProviderCapability::ChatCompletion,
                ProviderCapability::ChatCompletionStream,
                ProviderCapability::ToolCalling,
            ],
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Build capabilities from model config
    fn build_capabilities(&self, config: &OpenAILikeModelConfig) -> Vec<ProviderCapability> {
        let mut capabilities = vec![ProviderCapability::ChatCompletion];

        if config.supports_streaming {
            capabilities.push(ProviderCapability::ChatCompletionStream);
        }

        if config.supports_tools {
            capabilities.push(ProviderCapability::ToolCalling);
            capabilities.push(ProviderCapability::FunctionCalling);
        }

        capabilities
    }

    /// Check if a model is known (has explicit configuration)
    pub fn is_known_model(&self, model_id: &str) -> bool {
        self.known_models.contains_key(model_id)
    }

    /// Get all known models as ModelInfo list
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        self.known_models
            .keys()
            .map(|id| self.get_model_info(id))
            .collect()
    }

    /// Always returns true - any model name is accepted
    ///
    /// This is the key difference from other providers:
    /// we don't validate models locally, letting the API handle validation.
    pub fn supports_model(&self, _model_id: &str) -> bool {
        true
    }
}

/// Get a static registry instance with defaults
pub fn get_openai_like_registry() -> &'static OpenAILikeModelRegistry {
    static REGISTRY: std::sync::LazyLock<OpenAILikeModelRegistry> =
        std::sync::LazyLock::new(OpenAILikeModelRegistry::with_defaults);
    &REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_model_returns_default_info() {
        let registry = OpenAILikeModelRegistry::new();
        let info = registry.get_model_info("my-custom-model");

        assert_eq!(info.id, "my-custom-model");
        assert_eq!(info.name, "my-custom-model");
        assert_eq!(info.provider, "openai_like");
        assert!(info.supports_streaming);
    }

    #[test]
    fn test_all_models_supported() {
        let registry = OpenAILikeModelRegistry::new();

        assert!(registry.supports_model("any-model-name"));
        assert!(registry.supports_model("gpt-4"));
        assert!(registry.supports_model("llama-2-70b"));
        assert!(registry.supports_model("custom/my-model"));
    }

    #[test]
    fn test_known_model_returns_specific_info() {
        let mut registry = OpenAILikeModelRegistry::new();

        registry.register_model(OpenAILikeModelConfig {
            id: "llama-2-70b".to_string(),
            max_context_length: 4096,
            max_output_length: Some(2048),
            supports_streaming: true,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k: Some(0.0001),
            output_cost_per_1k: Some(0.0002),
        });

        let info = registry.get_model_info("llama-2-70b");
        assert_eq!(info.max_context_length, 4096);
        assert_eq!(info.max_output_length, Some(2048));
        assert!(!info.supports_tools);
    }

    #[test]
    fn test_custom_defaults() {
        let registry = OpenAILikeModelRegistry::new()
            .with_default_context_length(32000)
            .with_default_output_length(8000);

        let info = registry.get_model_info("unknown-model");
        assert_eq!(info.max_context_length, 32000);
        assert_eq!(info.max_output_length, Some(8000));
    }

    #[test]
    fn test_is_known_model() {
        let mut registry = OpenAILikeModelRegistry::new();

        registry.register_model(OpenAILikeModelConfig {
            id: "known-model".to_string(),
            ..Default::default()
        });

        assert!(registry.is_known_model("known-model"));
        assert!(!registry.is_known_model("unknown-model"));
    }

    #[test]
    fn test_static_registry() {
        let registry = get_openai_like_registry();
        assert!(registry.supports_model("any-model"));
    }
}
