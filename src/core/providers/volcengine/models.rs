//! Volcengine Model Registry
//!
//! Model definitions for ByteDance's Volcengine AI platform (Doubao models)

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::providers::base::get_pricing_db;
use crate::core::types::model::ModelInfo;

/// Type alias for model definition tuple: (id, name, context_len, output_len, input_cost, output_cost)
type ModelDefinition<'a> = (&'a str, &'a str, u32, Option<u32>, f64, f64);

/// Volcengine model registry
pub struct VolcengineModelRegistry {
    models: HashMap<String, ModelInfo>,
}

impl Default for VolcengineModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl VolcengineModelRegistry {
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
        let model_ids = pricing_db.get_provider_models("volcengine");

        for model_id in &model_ids {
            if let Some(model_info) = pricing_db.to_model_info(model_id, "volcengine") {
                self.models.insert(model_id.clone(), model_info);
            }
        }

        // Add default models if none loaded from pricing DB
        if self.models.is_empty() {
            self.add_default_models();
        }
    }

    /// Add default Volcengine models (Doubao series)
    fn add_default_models(&mut self) {
        // Volcengine Doubao models - pricing in CNY, converted to approximate USD
        // 1 USD ~ 7.2 CNY (approximate rate)
        let default_models: Vec<ModelDefinition> = vec![
            // Doubao Pro models
            (
                "doubao-pro-32k",
                "Doubao Pro 32K",
                32_768,
                Some(4_096),
                0.0001, // ~0.0008 CNY/1k tokens
                0.0002, // ~0.002 CNY/1k tokens
            ),
            (
                "doubao-pro-128k",
                "Doubao Pro 128K",
                128_000,
                Some(4_096),
                0.0007, // ~0.005 CNY/1k tokens
                0.0013, // ~0.009 CNY/1k tokens
            ),
            (
                "doubao-pro-256k",
                "Doubao Pro 256K",
                256_000,
                Some(4_096),
                0.0007,
                0.0013,
            ),
            // Doubao Lite models (cheaper)
            (
                "doubao-lite-32k",
                "Doubao Lite 32K",
                32_768,
                Some(4_096),
                0.00004, // ~0.0003 CNY/1k tokens
                0.00008, // ~0.0006 CNY/1k tokens
            ),
            (
                "doubao-lite-128k",
                "Doubao Lite 128K",
                128_000,
                Some(4_096),
                0.00011, // ~0.0008 CNY/1k tokens
                0.00013, // ~0.001 CNY/1k tokens
            ),
            // Doubao embedding models
            (
                "doubao-embedding",
                "Doubao Embedding",
                4_096,
                None,
                0.00007, // ~0.0005 CNY/1k tokens
                0.0,
            ),
            (
                "doubao-embedding-large",
                "Doubao Embedding Large",
                4_096,
                None,
                0.0001,
                0.0,
            ),
            // Doubao Vision models
            (
                "doubao-vision-pro-32k",
                "Doubao Vision Pro 32K",
                32_768,
                Some(4_096),
                0.0003, // Higher cost for vision
                0.0003,
            ),
            // Skylark models (another series on Volcengine)
            (
                "skylark-pro",
                "Skylark Pro",
                8_192,
                Some(4_096),
                0.0012,
                0.0012,
            ),
            (
                "skylark-lite",
                "Skylark Lite",
                4_096,
                Some(4_096),
                0.0004,
                0.0004,
            ),
            (
                "skylark-chat",
                "Skylark Chat",
                8_192,
                Some(4_096),
                0.0012,
                0.0012,
            ),
        ];

        for (id, name, context_len, output_len, input_cost, output_cost) in default_models {
            let is_embedding = id.contains("embedding");
            let is_vision = id.contains("vision");

            let model_info = ModelInfo {
                id: id.to_string(),
                name: name.to_string(),
                provider: "volcengine".to_string(),
                max_context_length: context_len,
                max_output_length: output_len,
                supports_streaming: !is_embedding,
                supports_tools: !is_embedding && !is_vision,
                supports_multimodal: is_vision,
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
static VOLCENGINE_REGISTRY: OnceLock<VolcengineModelRegistry> = OnceLock::new();

/// Get global model registry
pub fn get_volcengine_registry() -> &'static VolcengineModelRegistry {
    VOLCENGINE_REGISTRY.get_or_init(VolcengineModelRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_creation() {
        let registry = VolcengineModelRegistry::new();
        assert!(!registry.get_all_models().is_empty());
    }

    #[test]
    fn test_default_impl() {
        let registry = VolcengineModelRegistry::default();
        assert!(!registry.get_all_models().is_empty());
    }

    #[test]
    fn test_get_model() {
        let registry = get_volcengine_registry();
        let model = registry.get_model("doubao-pro-32k");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.provider, "volcengine");
    }

    #[test]
    fn test_get_model_nonexistent() {
        let registry = get_volcengine_registry();
        let model = registry.get_model("nonexistent-model");
        assert!(model.is_none());
    }

    #[test]
    fn test_has_model() {
        let registry = get_volcengine_registry();
        assert!(registry.has_model("doubao-pro-32k"));
        assert!(!registry.has_model("nonexistent"));
    }

    #[test]
    fn test_doubao_pro_model() {
        let registry = get_volcengine_registry();
        let model = registry.get_model("doubao-pro-32k").unwrap();

        assert_eq!(model.max_context_length, 32_768);
        assert!(model.supports_streaming);
        assert!(model.supports_tools);
        assert!(!model.supports_multimodal);
    }

    #[test]
    fn test_doubao_vision_model() {
        let registry = get_volcengine_registry();
        let model = registry.get_model("doubao-vision-pro-32k").unwrap();

        assert!(model.supports_multimodal);
    }

    #[test]
    fn test_embedding_model() {
        let registry = get_volcengine_registry();
        let model = registry.get_model("doubao-embedding").unwrap();

        assert!(!model.supports_streaming);
        assert!(!model.supports_tools);
    }

    #[test]
    fn test_model_info_properties() {
        let registry = get_volcengine_registry();
        let models = registry.get_all_models();

        for model in models {
            assert!(!model.id.is_empty());
            assert!(!model.name.is_empty());
            assert_eq!(model.provider, "volcengine");
            assert!(model.max_context_length > 0);
            assert_eq!(model.currency, "USD");
            assert!(model.input_cost_per_1k_tokens.is_some());
        }
    }

    #[test]
    fn test_global_registry() {
        let registry1 = get_volcengine_registry();
        let registry2 = get_volcengine_registry();

        assert_eq!(
            registry1.get_all_models().len(),
            registry2.get_all_models().len()
        );
    }
}
