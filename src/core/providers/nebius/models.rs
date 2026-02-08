//! Nebius Model Registry
//!
//! Model definitions for Nebius AI cloud platform

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::providers::base::get_pricing_db;
use crate::core::types::model::ModelInfo;

/// Type alias for model definition tuple: (id, name, context_len, output_len, input_cost, output_cost)
type ModelDefinition<'a> = (&'a str, &'a str, u32, Option<u32>, f64, f64);

/// Nebius model registry
pub struct NebiusModelRegistry {
    models: HashMap<String, ModelInfo>,
}

impl Default for NebiusModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NebiusModelRegistry {
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
        let model_ids = pricing_db.get_provider_models("nebius");

        for model_id in &model_ids {
            if let Some(model_info) = pricing_db.to_model_info(model_id, "nebius") {
                self.models.insert(model_id.clone(), model_info);
            }
        }

        // Add default models if none loaded from pricing DB
        if self.models.is_empty() {
            self.add_default_models();
        }
    }

    /// Add default Nebius models
    fn add_default_models(&mut self) {
        // Nebius provides access to various open-source models
        let default_models: Vec<ModelDefinition> = vec![
            // Llama 3.1 series
            (
                "meta-llama/Meta-Llama-3.1-8B-Instruct",
                "Llama 3.1 8B Instruct",
                128_000,
                Some(4_096),
                0.00006, // Competitive pricing
                0.00006,
            ),
            (
                "meta-llama/Meta-Llama-3.1-70B-Instruct",
                "Llama 3.1 70B Instruct",
                128_000,
                Some(4_096),
                0.00035,
                0.00040,
            ),
            (
                "meta-llama/Meta-Llama-3.1-405B-Instruct",
                "Llama 3.1 405B Instruct",
                128_000,
                Some(4_096),
                0.0020,
                0.0020,
            ),
            // Llama 3.3 series
            (
                "meta-llama/Llama-3.3-70B-Instruct",
                "Llama 3.3 70B Instruct",
                128_000,
                Some(4_096),
                0.00035,
                0.00040,
            ),
            // Mistral models
            (
                "mistralai/Mistral-7B-Instruct-v0.3",
                "Mistral 7B Instruct v0.3",
                32_768,
                Some(4_096),
                0.00006,
                0.00006,
            ),
            (
                "mistralai/Mixtral-8x7B-Instruct-v0.1",
                "Mixtral 8x7B Instruct",
                32_768,
                Some(4_096),
                0.00024,
                0.00024,
            ),
            (
                "mistralai/Mixtral-8x22B-Instruct-v0.1",
                "Mixtral 8x22B Instruct",
                64_000,
                Some(4_096),
                0.00065,
                0.00065,
            ),
            // Qwen models
            (
                "Qwen/Qwen2.5-7B-Instruct",
                "Qwen 2.5 7B Instruct",
                128_000,
                Some(8_192),
                0.00006,
                0.00006,
            ),
            (
                "Qwen/Qwen2.5-72B-Instruct",
                "Qwen 2.5 72B Instruct",
                128_000,
                Some(8_192),
                0.00040,
                0.00040,
            ),
            (
                "Qwen/QwQ-32B-Preview",
                "QwQ 32B Preview (Reasoning)",
                32_768,
                Some(8_192),
                0.00030,
                0.00030,
            ),
            // DeepSeek models
            (
                "deepseek-ai/DeepSeek-R1",
                "DeepSeek R1",
                128_000,
                Some(64_000),
                0.00055,
                0.00219,
            ),
            (
                "deepseek-ai/DeepSeek-V3",
                "DeepSeek V3",
                128_000,
                Some(8_192),
                0.00014,
                0.00028,
            ),
            // Embedding models
            (
                "BAAI/bge-en-icl",
                "BGE English ICL Embedding",
                8_192,
                None,
                0.00002,
                0.0,
            ),
            (
                "BAAI/bge-multilingual-gemma2",
                "BGE Multilingual Gemma2 Embedding",
                8_192,
                None,
                0.00002,
                0.0,
            ),
        ];

        for (id, name, context_len, output_len, input_cost, output_cost) in default_models {
            let is_embedding = id.contains("bge") || id.contains("embedding");

            let model_info = ModelInfo {
                id: id.to_string(),
                name: name.to_string(),
                provider: "nebius".to_string(),
                max_context_length: context_len,
                max_output_length: output_len,
                supports_streaming: !is_embedding,
                supports_tools: !is_embedding,
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
static NEBIUS_REGISTRY: OnceLock<NebiusModelRegistry> = OnceLock::new();

/// Get global model registry
pub fn get_nebius_registry() -> &'static NebiusModelRegistry {
    NEBIUS_REGISTRY.get_or_init(NebiusModelRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_creation() {
        let registry = NebiusModelRegistry::new();
        assert!(!registry.get_all_models().is_empty());
    }

    #[test]
    fn test_default_impl() {
        let registry = NebiusModelRegistry::default();
        assert!(!registry.get_all_models().is_empty());
    }

    #[test]
    fn test_get_model() {
        let registry = get_nebius_registry();
        let model = registry.get_model("meta-llama/Meta-Llama-3.1-8B-Instruct");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.provider, "nebius");
    }

    #[test]
    fn test_get_model_nonexistent() {
        let registry = get_nebius_registry();
        let model = registry.get_model("nonexistent-model");
        assert!(model.is_none());
    }

    #[test]
    fn test_has_model() {
        let registry = get_nebius_registry();
        assert!(registry.has_model("meta-llama/Meta-Llama-3.1-8B-Instruct"));
        assert!(!registry.has_model("nonexistent"));
    }

    #[test]
    fn test_llama_model() {
        let registry = get_nebius_registry();
        let model = registry
            .get_model("meta-llama/Meta-Llama-3.1-8B-Instruct")
            .unwrap();

        assert_eq!(model.max_context_length, 128_000);
        assert!(model.supports_streaming);
        assert!(model.supports_tools);
    }

    #[test]
    fn test_embedding_model() {
        let registry = get_nebius_registry();
        let model = registry.get_model("BAAI/bge-en-icl").unwrap();

        assert!(!model.supports_streaming);
        assert!(!model.supports_tools);
    }

    #[test]
    fn test_model_info_properties() {
        let registry = get_nebius_registry();
        let models = registry.get_all_models();

        for model in models {
            assert!(!model.id.is_empty());
            assert!(!model.name.is_empty());
            assert_eq!(model.provider, "nebius");
            assert!(model.max_context_length > 0);
            assert_eq!(model.currency, "USD");
            assert!(model.input_cost_per_1k_tokens.is_some());
        }
    }

    #[test]
    fn test_global_registry() {
        let registry1 = get_nebius_registry();
        let registry2 = get_nebius_registry();

        assert_eq!(
            registry1.get_all_models().len(),
            registry2.get_all_models().len()
        );
    }
}
