//! Cerebras Model Registry
//!
//! Model registry system for Cerebras AI models with fast inference capabilities

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::providers::base::get_pricing_db;
use crate::core::types::model::ModelInfo;

/// Model feature flags
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelFeature {
    /// Function calling support
    FunctionCalling,
    /// Streaming support
    StreamingSupport,
    /// System message support
    SystemMessages,
    /// Tool calling support
    ToolCalling,
    /// Fast inference (Cerebras specialty)
    FastInference,
}

/// Model specification with features
#[derive(Debug, Clone)]
pub struct ModelSpec {
    /// Model information
    pub model_info: ModelInfo,
    /// Supported features
    pub features: Vec<ModelFeature>,
    /// Model-specific configuration
    pub config: ModelConfig,
}

/// Model-specific configuration
#[derive(Debug, Clone, Default)]
pub struct ModelConfig {
    /// Maximum concurrent requests
    pub max_concurrent_requests: Option<u32>,
    /// Custom parameter mapping
    pub custom_params: HashMap<String, String>,
}

/// Cerebras Model Registry
pub struct CerebrasModelRegistry {
    models: HashMap<String, ModelSpec>,
}

impl Default for CerebrasModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CerebrasModelRegistry {
    /// Create new registry
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };
        registry.load_models();
        registry
    }

    /// Load models from pricing database or defaults
    fn load_models(&mut self) {
        let pricing_db = get_pricing_db();
        let model_ids = pricing_db.get_provider_models("cerebras");

        for model_id in &model_ids {
            if let Some(model_info) = pricing_db.to_model_info(model_id, "cerebras") {
                let features = self.detect_features(&model_info);
                let config = self.create_config(&model_info);

                self.models.insert(
                    model_id.clone(),
                    ModelSpec {
                        model_info,
                        features,
                        config,
                    },
                );
            }
        }

        // Add default models if none loaded
        if self.models.is_empty() {
            self.add_default_models();
        }
    }

    /// Detect features based on model info
    fn detect_features(&self, model_info: &ModelInfo) -> Vec<ModelFeature> {
        let mut features = vec![
            ModelFeature::SystemMessages,
            ModelFeature::StreamingSupport,
            ModelFeature::FastInference, // All Cerebras models have fast inference
        ];

        if model_info.supports_tools {
            features.push(ModelFeature::FunctionCalling);
            features.push(ModelFeature::ToolCalling);
        }

        features
    }

    /// Create model configuration
    fn create_config(&self, model_info: &ModelInfo) -> ModelConfig {
        ModelConfig {
            // Cerebras excels at high throughput, allow more concurrent requests
            max_concurrent_requests: Some(match model_info.id.as_str() {
                "llama3.1-70b" => 10,
                "llama3.1-8b" => 20,
                _ => 15,
            }),
            ..Default::default()
        }
    }

    /// Add default Cerebras models
    fn add_default_models(&mut self) {
        // Cerebras models (as of 2025) - known for fast inference
        let default_models = vec![
            // Llama 3.1 70B - large model with fast inference
            (
                "llama3.1-70b",
                "Llama 3.1 70B",
                128_000,
                Some(8_192),
                0.0006, // $0.60/1M input tokens
                0.0006, // $0.60/1M output tokens
                true,   // supports tools
            ),
            // Llama 3.1 8B - smaller, faster model
            (
                "llama3.1-8b",
                "Llama 3.1 8B",
                128_000,
                Some(8_192),
                0.0001, // $0.10/1M input tokens
                0.0001, // $0.10/1M output tokens
                true,   // supports tools
            ),
            // Llama 3.3 70B - latest version
            (
                "llama-3.3-70b",
                "Llama 3.3 70B",
                128_000,
                Some(8_192),
                0.00085, // $0.85/1M input tokens
                0.00085, // $0.85/1M output tokens
                true,
            ),
        ];

        for (id, name, context_len, output_len, input_cost, output_cost, supports_tools) in
            default_models
        {
            let model_info = ModelInfo {
                id: id.to_string(),
                name: name.to_string(),
                provider: "cerebras".to_string(),
                max_context_length: context_len,
                max_output_length: output_len,
                supports_streaming: true,
                supports_tools,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(input_cost),
                output_cost_per_1k_tokens: Some(output_cost),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("fast_inference".to_string(), serde_json::Value::Bool(true));
                    m
                },
            };

            let features = self.detect_features(&model_info);
            let config = self.create_config(&model_info);

            self.models.insert(
                id.to_string(),
                ModelSpec {
                    model_info,
                    features,
                    config,
                },
            );
        }
    }

    /// Get all models
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        self.models
            .values()
            .map(|spec| spec.model_info.clone())
            .collect()
    }

    /// Get model specification
    pub fn get_model_spec(&self, model_id: &str) -> Option<&ModelSpec> {
        self.models.get(model_id)
    }

    /// Check if model supports a feature
    pub fn supports_feature(&self, model_id: &str, feature: &ModelFeature) -> bool {
        self.models
            .get(model_id)
            .map(|spec| spec.features.contains(feature))
            .unwrap_or(false)
    }

    /// Get models with a specific feature
    pub fn get_models_with_feature(&self, feature: &ModelFeature) -> Vec<String> {
        self.models
            .iter()
            .filter_map(|(id, spec)| {
                if spec.features.contains(feature) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Global registry instance
static CEREBRAS_REGISTRY: OnceLock<CerebrasModelRegistry> = OnceLock::new();

/// Get global Cerebras model registry
pub fn get_cerebras_registry() -> &'static CerebrasModelRegistry {
    CEREBRAS_REGISTRY.get_or_init(CerebrasModelRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_creation() {
        let registry = CerebrasModelRegistry::new();
        assert!(!registry.get_all_models().is_empty());
    }

    #[test]
    fn test_feature_detection() {
        let registry = get_cerebras_registry();
        let models = registry.get_all_models();

        assert!(!models.is_empty());

        for model in &models {
            assert!(registry.supports_feature(&model.id, &ModelFeature::SystemMessages));
            assert!(registry.supports_feature(&model.id, &ModelFeature::StreamingSupport));
            assert!(registry.supports_feature(&model.id, &ModelFeature::FastInference));
        }
    }

    #[test]
    fn test_models_with_feature() {
        let registry = get_cerebras_registry();
        let tool_models = registry.get_models_with_feature(&ModelFeature::ToolCalling);
        assert!(!tool_models.is_empty());
    }

    #[test]
    fn test_fast_inference_feature() {
        let registry = get_cerebras_registry();
        let fast_models = registry.get_models_with_feature(&ModelFeature::FastInference);

        // All Cerebras models should have fast inference
        let all_models = registry.get_all_models();
        assert_eq!(fast_models.len(), all_models.len());
    }

    #[test]
    fn test_default_impl() {
        let registry = CerebrasModelRegistry::default();
        assert!(!registry.get_all_models().is_empty());
    }

    #[test]
    fn test_get_model_spec() {
        let registry = get_cerebras_registry();
        let spec = registry.get_model_spec("llama3.1-70b");
        assert!(spec.is_some());
        let spec = spec.unwrap();
        assert_eq!(spec.model_info.provider, "cerebras");
    }

    #[test]
    fn test_get_model_spec_nonexistent() {
        let registry = get_cerebras_registry();
        let spec = registry.get_model_spec("nonexistent-model");
        assert!(spec.is_none());
    }

    #[test]
    fn test_model_config() {
        let registry = get_cerebras_registry();

        if let Some(spec) = registry.get_model_spec("llama3.1-70b") {
            assert_eq!(spec.config.max_concurrent_requests, Some(10));
        }

        if let Some(spec) = registry.get_model_spec("llama3.1-8b") {
            assert_eq!(spec.config.max_concurrent_requests, Some(20));
        }
    }

    #[test]
    fn test_model_feature_equality() {
        assert_eq!(ModelFeature::FunctionCalling, ModelFeature::FunctionCalling);
        assert_ne!(ModelFeature::FunctionCalling, ModelFeature::FastInference);
    }

    #[test]
    fn test_model_info_properties() {
        let registry = get_cerebras_registry();
        let models = registry.get_all_models();

        for model in models {
            assert!(!model.id.is_empty());
            assert!(!model.name.is_empty());
            assert_eq!(model.provider, "cerebras");
            assert!(model.max_context_length > 0);
            assert_eq!(model.currency, "USD");
            assert!(model.input_cost_per_1k_tokens.is_some());
            assert!(model.output_cost_per_1k_tokens.is_some());
        }
    }

    #[test]
    fn test_global_registry() {
        let registry1 = get_cerebras_registry();
        let registry2 = get_cerebras_registry();

        assert_eq!(
            registry1.get_all_models().len(),
            registry2.get_all_models().len()
        );
    }

    #[test]
    fn test_llama_70b_model() {
        let registry = get_cerebras_registry();
        let spec = registry.get_model_spec("llama3.1-70b").unwrap();

        assert_eq!(spec.model_info.max_context_length, 128_000);
        assert!(registry.supports_feature("llama3.1-70b", &ModelFeature::ToolCalling));
        assert!(registry.supports_feature("llama3.1-70b", &ModelFeature::FunctionCalling));
        assert!(registry.supports_feature("llama3.1-70b", &ModelFeature::FastInference));
    }

    #[test]
    fn test_supports_feature_nonexistent() {
        let registry = get_cerebras_registry();
        assert!(!registry.supports_feature("nonexistent", &ModelFeature::FunctionCalling));
        assert!(!registry.supports_feature("nonexistent", &ModelFeature::ToolCalling));
    }

    #[test]
    fn test_model_metadata_fast_inference() {
        let registry = get_cerebras_registry();
        let models = registry.get_all_models();

        for model in models {
            // All models should have fast_inference metadata
            assert!(model.metadata.contains_key("fast_inference"));
        }
    }
}
