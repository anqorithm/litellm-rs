//! OpenAI-Like Provider Model Support
//!
//! Dynamic model support - accepts any model name and passes it through

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{model::ModelInfo, model::ProviderCapability};
use std::collections::HashMap;

const XAI_HIGH_CONTEXT_THRESHOLD_TOKENS: u32 = 200_000;
const XAI_GROK_43_INPUT_COST_PER_1K: f64 = 0.00125;
const XAI_GROK_43_OUTPUT_COST_PER_1K: f64 = 0.0025;
const XAI_GROK_43_CONTEXT_LENGTH: u32 = 1_000_000;
const XAI_GROK_BUILD_INPUT_COST_PER_1K: f64 = 0.001;
const XAI_GROK_BUILD_OUTPUT_COST_PER_1K: f64 = 0.002;
const XAI_GROK_BUILD_CONTEXT_LENGTH: u32 = 256_000;

const XAI_GROK_43_MODEL_IDS: &[&str] = &[
    "grok-4.3",
    "grok-4.3-latest",
    "grok-latest",
    "grok-3",
    "grok-3-latest",
    "grok-3-beta",
    "grok-3-fast",
    "grok-3-fast-latest",
    "grok-3-fast-beta",
    "grok-3-mini",
    "grok-3-mini-latest",
    "grok-3-mini-beta",
    "grok-3-mini-fast",
    "grok-3-mini-fast-latest",
    "grok-3-mini-fast-beta",
    "grok-3-mini-high",
    "grok-3-mini-high-beta",
    "grok-3-mini-fast-high",
    "grok-3-mini-fast-high-beta",
    "grok-4-0709",
    "grok-4",
    "grok-4-latest",
    "grok-4-fast-reasoning",
    "grok-4-fast",
    "grok-4-fast-reasoning-latest",
    "grok-4-fast-non-reasoning",
    "grok-4-fast-non-reasoning-latest",
    "grok-4-1-fast-reasoning",
    "grok-4-1-fast",
    "grok-4-1-fast-reasoning-latest",
    "grok-4-1-fast-non-reasoning",
    "grok-4-1-fast-non-reasoning-latest",
];

const XAI_GROK_420_MODEL_IDS: &[&str] = &[
    "grok-4.20-multi-agent-0309",
    "grok-4.20-multi-agent",
    "grok-4.20-multi-agent-latest",
    "grok-4.20-multi-agent-beta-latest",
    "grok-4.20-multi-agent-experimental-beta-0304",
    "grok-4.20-multi-agent-experimental-beta-latest",
    "grok-4.20-multi-agent-beta-0309",
    "grok-4.20-0309-reasoning",
    "grok-4.20-reasoning-latest",
    "grok-4.20",
    "grok-4.20-reasoning",
    "grok-4.20-0309",
    "grok-4.20-beta-0309-reasoning",
    "grok-4.20-beta",
    "grok-4.20-beta-0309",
    "grok-4.20-beta-latest",
    "grok-4.20-beta-latest-reasoning",
    "grok-4.20-beta-reasoning",
    "grok-4.20-experimental-beta-0304-reasoning",
    "grok-4.20-experimental-beta-0304",
    "grok-4.20-experimental-beta-reasoning-latest",
    "grok-4.20-experimental-beta-latest",
    "grok-4.20-reasoning-gv2",
    "grok-4.20-0309-non-reasoning",
    "grok-4.20-non-reasoning",
    "grok-4.20-non-reasoning-latest",
    "grok-4.20-beta-non-reasoning",
    "grok-4.20-beta-latest-non-reasoning",
    "grok-4.20-experimental-beta-0304-non-reasoning",
    "grok-4.20-experimental-beta-non-reasoning-latest",
    "grok-4.20-beta-0309-non-reasoning",
    "grok-4.20-non-reasoning-gv2",
];

const XAI_GROK_BUILD_MODEL_IDS: &[&str] = &[
    "grok-build-0.1",
    "grok-code-fast-1",
    "grok-code-fast",
    "grok-code-fast-1-0825",
];

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
        registry.register_xai_model_family(
            XAI_GROK_43_MODEL_IDS,
            XAI_GROK_43_CONTEXT_LENGTH,
            XAI_GROK_43_INPUT_COST_PER_1K,
            XAI_GROK_43_OUTPUT_COST_PER_1K,
        );
        registry.register_xai_model_family(
            XAI_GROK_420_MODEL_IDS,
            XAI_GROK_43_CONTEXT_LENGTH,
            XAI_GROK_43_INPUT_COST_PER_1K,
            XAI_GROK_43_OUTPUT_COST_PER_1K,
        );
        registry.register_xai_model_family(
            XAI_GROK_BUILD_MODEL_IDS,
            XAI_GROK_BUILD_CONTEXT_LENGTH,
            XAI_GROK_BUILD_INPUT_COST_PER_1K,
            XAI_GROK_BUILD_OUTPUT_COST_PER_1K,
        );
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

    fn register_xai_model_family(
        &mut self,
        model_ids: &[&str],
        context_length: u32,
        input_cost_per_1k: f64,
        output_cost_per_1k: f64,
    ) {
        for model_id in model_ids {
            self.register_model(OpenAILikeModelConfig {
                id: (*model_id).to_string(),
                max_context_length: context_length,
                max_output_length: Some(self.default_output_length),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k: Some(input_cost_per_1k),
                output_cost_per_1k: Some(output_cost_per_1k),
            });
        }
    }

    fn known_config_for_model(&self, model_id: &str) -> Option<&OpenAILikeModelConfig> {
        self.known_models.get(model_id).or_else(|| {
            model_id
                .strip_prefix("xai/")
                .and_then(|stripped| self.known_models.get(stripped))
        })
    }

    /// Get model info for any model name
    ///
    /// If the model is known, returns its specific configuration.
    /// Otherwise, returns default configuration that allows the request to proceed.
    pub fn get_model_info(&self, model_id: &str) -> ModelInfo {
        if let Some(config) = self.known_config_for_model(model_id) {
            ModelInfo {
                id: model_id.to_string(),
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
        self.known_config_for_model(model_id).is_some()
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

pub fn validate_xai_standard_cost_window(
    provider_name: &str,
    model_id: &str,
    input_tokens: u32,
) -> Result<(), ProviderError> {
    if provider_name == "xai"
        && input_tokens > XAI_HIGH_CONTEXT_THRESHOLD_TOKENS
        && is_known_xai_priced_model(model_id)
    {
        return Err(ProviderError::configuration(
            "xai",
            format!(
                "xAI high-context pricing is not represented for {} above {} input tokens",
                model_id, XAI_HIGH_CONTEXT_THRESHOLD_TOKENS
            ),
        ));
    }

    Ok(())
}

fn is_known_xai_priced_model(model_id: &str) -> bool {
    let model_id = model_id.strip_prefix("xai/").unwrap_or(model_id);

    XAI_GROK_43_MODEL_IDS.contains(&model_id)
        || XAI_GROK_420_MODEL_IDS.contains(&model_id)
        || XAI_GROK_BUILD_MODEL_IDS.contains(&model_id)
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

    #[test]
    fn test_static_registry_prices_xai_grok_models() {
        let registry = get_openai_like_registry();

        for model_id in [
            "grok-4.3",
            "xai/grok-4.3",
            "grok-latest",
            "grok-4.20-multi-agent-0309",
            "grok-4.20-0309-reasoning",
            "grok-4.20-0309-non-reasoning",
        ] {
            let info = registry.get_model_info(model_id);

            assert_eq!(info.id, model_id);
            assert_eq!(info.provider, "openai_like");
            assert_eq!(info.max_context_length, 1_000_000);
            assert!(info.supports_tools);
            assert!(info.supports_multimodal);
            assert_eq!(
                info.input_cost_per_1k_tokens,
                Some(XAI_GROK_43_INPUT_COST_PER_1K)
            );
            assert_eq!(
                info.output_cost_per_1k_tokens,
                Some(XAI_GROK_43_OUTPUT_COST_PER_1K)
            );
        }

        assert!(registry.is_known_model("xai/grok-4.3"));
    }

    #[test]
    fn test_static_registry_prices_xai_grok_build_model() {
        let registry = get_openai_like_registry();
        let info = registry.get_model_info("grok-build-0.1");

        assert_eq!(info.max_context_length, 256_000);
        assert_eq!(
            info.input_cost_per_1k_tokens,
            Some(XAI_GROK_BUILD_INPUT_COST_PER_1K)
        );
        assert_eq!(
            info.output_cost_per_1k_tokens,
            Some(XAI_GROK_BUILD_OUTPUT_COST_PER_1K)
        );
    }

    #[test]
    fn test_xai_high_context_cost_validation_rejects_flat_estimate() {
        assert!(validate_xai_standard_cost_window("xai", "grok-4.3", 200_001).is_err());
        assert!(validate_xai_standard_cost_window("xai", "grok-4.3", 200_000).is_ok());
        assert!(validate_xai_standard_cost_window("groq", "grok-4.3", 200_001).is_ok());
        assert!(validate_xai_standard_cost_window("xai", "unknown-grok", 200_001).is_ok());
    }
}
