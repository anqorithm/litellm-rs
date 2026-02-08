//! Perplexity Model Registry
//!
//! Model definitions and registry for Perplexity AI models

use crate::core::types::model::ModelInfo;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Perplexity model features
#[derive(Debug, Clone, PartialEq)]
pub enum ModelFeature {
    /// Supports web search integration
    WebSearch,
    /// Supports citations in responses
    Citations,
    /// Supports streaming output
    Streaming,
    /// Supports reasoning/thinking mode
    Reasoning,
}

/// Model specification with metadata
#[derive(Debug, Clone)]
pub struct ModelSpec {
    /// Model ID
    pub id: &'static str,
    /// Display name
    pub name: &'static str,
    /// Maximum context length
    pub max_context: u32,
    /// Maximum output tokens
    pub max_output: Option<u32>,
    /// Supported features
    pub features: Vec<ModelFeature>,
    /// Input cost per 1K tokens (USD)
    pub input_cost_per_1k: f64,
    /// Output cost per 1K tokens (USD)
    pub output_cost_per_1k: f64,
    /// Search cost per request (USD)
    pub search_cost_per_request: Option<f64>,
}

/// Perplexity model registry
#[derive(Debug)]
pub struct PerplexityModelRegistry {
    models: HashMap<&'static str, ModelSpec>,
}

impl PerplexityModelRegistry {
    /// Create new model registry with Perplexity models
    fn new() -> Self {
        let mut models = HashMap::new();

        // Sonar models (search-integrated)
        models.insert(
            "sonar",
            ModelSpec {
                id: "sonar",
                name: "Sonar",
                max_context: 127072,
                max_output: Some(4096),
                features: vec![
                    ModelFeature::WebSearch,
                    ModelFeature::Citations,
                    ModelFeature::Streaming,
                ],
                input_cost_per_1k: 0.001,
                output_cost_per_1k: 0.001,
                search_cost_per_request: Some(0.005),
            },
        );

        models.insert(
            "sonar-pro",
            ModelSpec {
                id: "sonar-pro",
                name: "Sonar Pro",
                max_context: 200000,
                max_output: Some(8192),
                features: vec![
                    ModelFeature::WebSearch,
                    ModelFeature::Citations,
                    ModelFeature::Streaming,
                ],
                input_cost_per_1k: 0.003,
                output_cost_per_1k: 0.015,
                search_cost_per_request: Some(0.005),
            },
        );

        models.insert(
            "sonar-reasoning",
            ModelSpec {
                id: "sonar-reasoning",
                name: "Sonar Reasoning",
                max_context: 127072,
                max_output: Some(4096),
                features: vec![
                    ModelFeature::WebSearch,
                    ModelFeature::Citations,
                    ModelFeature::Streaming,
                    ModelFeature::Reasoning,
                ],
                input_cost_per_1k: 0.001,
                output_cost_per_1k: 0.005,
                search_cost_per_request: Some(0.005),
            },
        );

        models.insert(
            "sonar-reasoning-pro",
            ModelSpec {
                id: "sonar-reasoning-pro",
                name: "Sonar Reasoning Pro",
                max_context: 127072,
                max_output: Some(8192),
                features: vec![
                    ModelFeature::WebSearch,
                    ModelFeature::Citations,
                    ModelFeature::Streaming,
                    ModelFeature::Reasoning,
                ],
                input_cost_per_1k: 0.002,
                output_cost_per_1k: 0.008,
                search_cost_per_request: Some(0.005),
            },
        );

        // Chat models (without search)
        models.insert(
            "llama-3.1-sonar-small-128k-chat",
            ModelSpec {
                id: "llama-3.1-sonar-small-128k-chat",
                name: "Llama 3.1 Sonar Small Chat",
                max_context: 127072,
                max_output: Some(4096),
                features: vec![ModelFeature::Streaming],
                input_cost_per_1k: 0.0002,
                output_cost_per_1k: 0.0002,
                search_cost_per_request: None,
            },
        );

        models.insert(
            "llama-3.1-sonar-large-128k-chat",
            ModelSpec {
                id: "llama-3.1-sonar-large-128k-chat",
                name: "Llama 3.1 Sonar Large Chat",
                max_context: 127072,
                max_output: Some(4096),
                features: vec![ModelFeature::Streaming],
                input_cost_per_1k: 0.001,
                output_cost_per_1k: 0.001,
                search_cost_per_request: None,
            },
        );

        // Online models (search-enabled)
        models.insert(
            "llama-3.1-sonar-small-128k-online",
            ModelSpec {
                id: "llama-3.1-sonar-small-128k-online",
                name: "Llama 3.1 Sonar Small Online",
                max_context: 127072,
                max_output: Some(4096),
                features: vec![
                    ModelFeature::WebSearch,
                    ModelFeature::Citations,
                    ModelFeature::Streaming,
                ],
                input_cost_per_1k: 0.0002,
                output_cost_per_1k: 0.0002,
                search_cost_per_request: Some(0.005),
            },
        );

        models.insert(
            "llama-3.1-sonar-large-128k-online",
            ModelSpec {
                id: "llama-3.1-sonar-large-128k-online",
                name: "Llama 3.1 Sonar Large Online",
                max_context: 127072,
                max_output: Some(4096),
                features: vec![
                    ModelFeature::WebSearch,
                    ModelFeature::Citations,
                    ModelFeature::Streaming,
                ],
                input_cost_per_1k: 0.001,
                output_cost_per_1k: 0.001,
                search_cost_per_request: Some(0.005),
            },
        );

        Self { models }
    }

    /// Get model spec by ID
    pub fn get_model_spec(&self, model_id: &str) -> Option<&ModelSpec> {
        self.models.get(model_id)
    }

    /// Get all models as ModelInfo
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        self.models
            .values()
            .map(|spec| ModelInfo {
                id: spec.id.to_string(),
                name: spec.name.to_string(),
                provider: "perplexity".to_string(),
                max_context_length: spec.max_context,
                max_output_length: spec.max_output,
                supports_streaming: spec.features.contains(&ModelFeature::Streaming),
                supports_tools: false, // Perplexity doesn't support tool calling
                supports_multimodal: false,
                capabilities: Vec::new(),
                input_cost_per_1k_tokens: Some(spec.input_cost_per_1k),
                output_cost_per_1k_tokens: Some(spec.output_cost_per_1k),
                currency: "USD".to_string(),
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            })
            .collect()
    }

    /// Check if model supports a feature
    pub fn model_supports_feature(&self, model_id: &str, feature: &ModelFeature) -> bool {
        self.models
            .get(model_id)
            .map(|spec| spec.features.contains(feature))
            .unwrap_or(false)
    }

    /// Get models that support web search
    pub fn get_search_models(&self) -> Vec<&'static str> {
        self.models
            .iter()
            .filter(|(_, spec)| spec.features.contains(&ModelFeature::WebSearch))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get models that support reasoning
    pub fn get_reasoning_models(&self) -> Vec<&'static str> {
        self.models
            .iter()
            .filter(|(_, spec)| spec.features.contains(&ModelFeature::Reasoning))
            .map(|(id, _)| *id)
            .collect()
    }
}

/// Global model registry instance
static PERPLEXITY_REGISTRY: LazyLock<PerplexityModelRegistry> =
    LazyLock::new(PerplexityModelRegistry::new);

/// Get the global Perplexity model registry
pub fn get_perplexity_registry() -> &'static PerplexityModelRegistry {
    &PERPLEXITY_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = get_perplexity_registry();
        assert!(!registry.models.is_empty());
    }

    #[test]
    fn test_get_model_spec() {
        let registry = get_perplexity_registry();
        let spec = registry.get_model_spec("sonar");
        assert!(spec.is_some());
        let spec = spec.unwrap();
        assert_eq!(spec.id, "sonar");
        assert!(spec.features.contains(&ModelFeature::WebSearch));
    }

    #[test]
    fn test_get_model_spec_not_found() {
        let registry = get_perplexity_registry();
        let spec = registry.get_model_spec("nonexistent-model");
        assert!(spec.is_none());
    }

    #[test]
    fn test_get_all_models() {
        let registry = get_perplexity_registry();
        let models = registry.get_all_models();
        assert!(models.len() >= 4); // At least 4 models should be registered

        // Check that all models have required fields
        for model in &models {
            assert!(!model.id.is_empty());
            assert!(!model.name.is_empty());
            assert_eq!(model.provider, "perplexity");
            assert!(model.max_context_length > 0);
        }
    }

    #[test]
    fn test_model_supports_feature() {
        let registry = get_perplexity_registry();

        // Sonar should support web search
        assert!(registry.model_supports_feature("sonar", &ModelFeature::WebSearch));
        assert!(registry.model_supports_feature("sonar", &ModelFeature::Citations));
        assert!(registry.model_supports_feature("sonar", &ModelFeature::Streaming));

        // Reasoning model should support reasoning
        assert!(registry.model_supports_feature("sonar-reasoning", &ModelFeature::Reasoning));

        // Chat model should not support web search
        assert!(
            !registry.model_supports_feature(
                "llama-3.1-sonar-small-128k-chat",
                &ModelFeature::WebSearch
            )
        );
    }

    #[test]
    fn test_get_search_models() {
        let registry = get_perplexity_registry();
        let search_models = registry.get_search_models();
        assert!(!search_models.is_empty());
        assert!(search_models.contains(&"sonar"));
        assert!(search_models.contains(&"sonar-pro"));
    }

    #[test]
    fn test_get_reasoning_models() {
        let registry = get_perplexity_registry();
        let reasoning_models = registry.get_reasoning_models();
        assert!(!reasoning_models.is_empty());
        assert!(reasoning_models.contains(&"sonar-reasoning"));
        assert!(reasoning_models.contains(&"sonar-reasoning-pro"));
    }

    #[test]
    fn test_model_pricing() {
        let registry = get_perplexity_registry();
        let spec = registry.get_model_spec("sonar-pro").unwrap();
        assert!(spec.input_cost_per_1k > 0.0);
        assert!(spec.output_cost_per_1k > 0.0);
        assert!(spec.search_cost_per_request.is_some());
    }

    #[test]
    fn test_model_context_length() {
        let registry = get_perplexity_registry();
        let spec = registry.get_model_spec("sonar").unwrap();
        assert!(spec.max_context >= 100000);
        assert!(spec.max_output.unwrap() >= 4096);
    }

    #[test]
    fn test_chat_model_no_search_cost() {
        let registry = get_perplexity_registry();
        let spec = registry
            .get_model_spec("llama-3.1-sonar-small-128k-chat")
            .unwrap();
        assert!(spec.search_cost_per_request.is_none());
    }
}
