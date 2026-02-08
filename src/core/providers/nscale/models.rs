//! Nscale Model Registry
//!
//! Model definitions for Nscale AI inference platform

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::providers::base::get_pricing_db;
use crate::core::types::model::ModelInfo;

/// Type alias for model definition tuple: (id, name, context_len, output_len, input_cost, output_cost)
type ModelDefinition<'a> = (&'a str, &'a str, u32, Option<u32>, f64, f64);

/// Nscale model registry
pub struct NscaleModelRegistry {
    models: HashMap<String, ModelInfo>,
}

impl Default for NscaleModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NscaleModelRegistry {
    /// Create new model registry
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
        let model_ids = pricing_db.get_provider_models("nscale");

        for model_id in &model_ids {
            if let Some(model_info) = pricing_db.to_model_info(model_id, "nscale") {
                self.models.insert(model_id.clone(), model_info);
            }
        }

        // Add default models if none loaded from pricing DB
        if self.models.is_empty() {
            self.add_default_models();
        }
    }

    /// Add default Nscale models
    fn add_default_models(&mut self) {
        // Nscale provides fast inference for various models
        let default_models: Vec<ModelDefinition> = vec![
            // Llama 3.1 series
            (
                "meta-llama/Llama-3.1-8B-Instruct",
                "Llama 3.1 8B Instruct",
                128_000,
                Some(4_096),
                0.00005, // Very competitive pricing
                0.00005,
            ),
            (
                "meta-llama/Llama-3.1-70B-Instruct",
                "Llama 3.1 70B Instruct",
                128_000,
                Some(4_096),
                0.00030,
                0.00035,
            ),
            // Llama 3.3
            (
                "meta-llama/Llama-3.3-70B-Instruct",
                "Llama 3.3 70B Instruct",
                128_000,
                Some(4_096),
                0.00030,
                0.00035,
            ),
            // Mistral models
            (
                "mistralai/Mistral-7B-Instruct-v0.3",
                "Mistral 7B Instruct v0.3",
                32_768,
                Some(4_096),
                0.00005,
                0.00005,
            ),
            (
                "mistralai/Mixtral-8x7B-Instruct-v0.1",
                "Mixtral 8x7B Instruct",
                32_768,
                Some(4_096),
                0.00020,
                0.00020,
            ),
            // Qwen models
            (
                "Qwen/Qwen2.5-7B-Instruct",
                "Qwen 2.5 7B Instruct",
                128_000,
                Some(8_192),
                0.00005,
                0.00005,
            ),
            (
                "Qwen/Qwen2.5-72B-Instruct",
                "Qwen 2.5 72B Instruct",
                128_000,
                Some(8_192),
                0.00035,
                0.00035,
            ),
            (
                "Qwen/Qwen2.5-Coder-32B-Instruct",
                "Qwen 2.5 Coder 32B Instruct",
                128_000,
                Some(8_192),
                0.00025,
                0.00025,
            ),
            // DeepSeek models
            (
                "deepseek-ai/DeepSeek-R1-Distill-Llama-70B",
                "DeepSeek R1 Distill Llama 70B",
                128_000,
                Some(32_000),
                0.00040,
                0.00040,
            ),
            (
                "deepseek-ai/DeepSeek-R1-Distill-Qwen-32B",
                "DeepSeek R1 Distill Qwen 32B",
                128_000,
                Some(32_000),
                0.00025,
                0.00025,
            ),
            // Gemma models
            (
                "google/gemma-2-9b-it",
                "Gemma 2 9B Instruct",
                8_192,
                Some(4_096),
                0.00006,
                0.00006,
            ),
            (
                "google/gemma-2-27b-it",
                "Gemma 2 27B Instruct",
                8_192,
                Some(4_096),
                0.00015,
                0.00015,
            ),
            // Phi models
            (
                "microsoft/Phi-3-mini-4k-instruct",
                "Phi 3 Mini 4K Instruct",
                4_096,
                Some(2_048),
                0.00003,
                0.00003,
            ),
            (
                "microsoft/Phi-3-medium-4k-instruct",
                "Phi 3 Medium 4K Instruct",
                4_096,
                Some(2_048),
                0.00008,
                0.00008,
            ),
        ];

        for (id, name, context_len, output_len, input_cost, output_cost) in default_models {
            let model_info = ModelInfo {
                id: id.to_string(),
                name: name.to_string(),
                provider: "nscale".to_string(),
                max_context_length: context_len,
                max_output_length: output_len,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(input_cost),
                output_cost_per_1k_tokens: Some(output_cost),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            };

            self.models.insert(id.to_string(), model_info);
        }
    }

    /// Get all models
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        self.models.values().cloned().collect()
    }

    /// Get model by ID
    pub fn get_model(&self, model_id: &str) -> Option<&ModelInfo> {
        self.models.get(model_id)
    }

    /// Check if model exists
    pub fn has_model(&self, model_id: &str) -> bool {
        self.models.contains_key(model_id)
    }
}

/// Global model registry instance
static NSCALE_REGISTRY: OnceLock<NscaleModelRegistry> = OnceLock::new();

/// Get global model registry
pub fn get_nscale_registry() -> &'static NscaleModelRegistry {
    NSCALE_REGISTRY.get_or_init(NscaleModelRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_creation() {
        let registry = NscaleModelRegistry::new();
        assert!(!registry.get_all_models().is_empty());
    }

    #[test]
    fn test_default_impl() {
        let registry = NscaleModelRegistry::default();
        assert!(!registry.get_all_models().is_empty());
    }

    #[test]
    fn test_get_model() {
        let registry = get_nscale_registry();
        let model = registry.get_model("meta-llama/Llama-3.1-8B-Instruct");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.provider, "nscale");
    }

    #[test]
    fn test_get_model_nonexistent() {
        let registry = get_nscale_registry();
        let model = registry.get_model("nonexistent-model");
        assert!(model.is_none());
    }

    #[test]
    fn test_has_model() {
        let registry = get_nscale_registry();
        assert!(registry.has_model("meta-llama/Llama-3.1-8B-Instruct"));
        assert!(!registry.has_model("nonexistent"));
    }

    #[test]
    fn test_llama_model() {
        let registry = get_nscale_registry();
        let model = registry
            .get_model("meta-llama/Llama-3.1-8B-Instruct")
            .unwrap();

        assert_eq!(model.max_context_length, 128_000);
        assert!(model.supports_streaming);
        assert!(model.supports_tools);
    }

    #[test]
    fn test_model_info_properties() {
        let registry = get_nscale_registry();
        let models = registry.get_all_models();

        for model in models {
            assert!(!model.id.is_empty());
            assert!(!model.name.is_empty());
            assert_eq!(model.provider, "nscale");
            assert!(model.max_context_length > 0);
            assert_eq!(model.currency, "USD");
            assert!(model.input_cost_per_1k_tokens.is_some());
        }
    }

    #[test]
    fn test_global_registry() {
        let registry1 = get_nscale_registry();
        let registry2 = get_nscale_registry();

        assert_eq!(
            registry1.get_all_models().len(),
            registry2.get_all_models().len()
        );
    }
}
