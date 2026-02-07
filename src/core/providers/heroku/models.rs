//! Heroku Model Registry
//!
//! Model registry system for Heroku AI Inference models.
//! Heroku provides access to various AI models including Claude, Amazon Nova, and more.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::providers::base::get_pricing_db;
use crate::core::types::ModelInfo;

use super::config::PROVIDER_NAME;

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
    /// Vision/multimodal support
    Vision,
    /// Embedding support
    Embedding,
    /// Image generation support
    ImageGeneration,
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
    /// Underlying provider (e.g., "anthropic", "amazon", "openai")
    pub underlying_provider: Option<String>,
}

/// Heroku Model Registry
pub struct HerokuModelRegistry {
    models: HashMap<String, ModelSpec>,
}

impl Default for HerokuModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HerokuModelRegistry {
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
        let model_ids = pricing_db.get_provider_models(PROVIDER_NAME);

        for model_id in &model_ids {
            if let Some(model_info) = pricing_db.to_model_info(model_id, PROVIDER_NAME) {
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
        let mut features = vec![ModelFeature::SystemMessages, ModelFeature::StreamingSupport];

        if model_info.supports_tools {
            features.push(ModelFeature::FunctionCalling);
            features.push(ModelFeature::ToolCalling);
        }

        if model_info.supports_multimodal {
            features.push(ModelFeature::Vision);
        }

        // Check for embedding models
        if model_info.id.contains("embed") {
            features.push(ModelFeature::Embedding);
        }

        // Check for image generation models
        if model_info.id.contains("stable") || model_info.id.contains("image") {
            features.push(ModelFeature::ImageGeneration);
        }

        features
    }

    /// Create model configuration
    fn create_config(&self, model_info: &ModelInfo) -> ModelConfig {
        let underlying_provider = if model_info.id.contains("claude") {
            Some("anthropic".to_string())
        } else if model_info.id.contains("nova") {
            Some("amazon".to_string())
        } else if model_info.id.contains("gpt") {
            Some("openai".to_string())
        } else if model_info.id.contains("cohere") || model_info.id.contains("embed") {
            Some("cohere".to_string())
        } else if model_info.id.contains("stable") {
            Some("stability".to_string())
        } else {
            None
        };

        ModelConfig {
            max_concurrent_requests: Some(10),
            underlying_provider,
            ..Default::default()
        }
    }

    /// Add default Heroku models based on the documented offerings
    fn add_default_models(&mut self) {
        // Heroku Inference models as documented
        let default_models = vec![
            // Claude models (Anthropic via Heroku)
            (
                "claude-4-5-sonnet",
                "Claude 4.5 Sonnet",
                200_000,
                Some(8_192),
                0.003, // Estimated pricing
                0.015,
                true, // supports tools
                true, // supports multimodal
                "anthropic",
            ),
            (
                "claude-4-5-haiku",
                "Claude 4.5 Haiku",
                200_000,
                Some(8_192),
                0.00025,
                0.00125,
                true,
                true,
                "anthropic",
            ),
            (
                "claude-4-sonnet",
                "Claude 4 Sonnet",
                200_000,
                Some(8_192),
                0.003,
                0.015,
                true,
                true,
                "anthropic",
            ),
            (
                "claude-3-7-sonnet",
                "Claude 3.7 Sonnet",
                200_000,
                Some(8_192),
                0.003,
                0.015,
                true,
                true,
                "anthropic",
            ),
            (
                "claude-3-5-sonnet-latest",
                "Claude 3.5 Sonnet Latest",
                200_000,
                Some(8_192),
                0.003,
                0.015,
                true,
                true,
                "anthropic",
            ),
            (
                "claude-3-5-haiku",
                "Claude 3.5 Haiku",
                200_000,
                Some(8_192),
                0.00025,
                0.00125,
                true,
                false,
                "anthropic",
            ),
            (
                "claude-3-0-haiku",
                "Claude 3.0 Haiku",
                200_000,
                Some(4_096),
                0.00025,
                0.00125,
                true,
                true,
                "anthropic",
            ),
            // Amazon Nova models
            (
                "amazon-nova-lite",
                "Amazon Nova Lite",
                128_000,
                Some(4_096),
                0.0001,
                0.0004,
                true,
                true,
                "amazon",
            ),
            (
                "amazon-nova-pro",
                "Amazon Nova Pro",
                128_000,
                Some(4_096),
                0.0008,
                0.0032,
                true,
                true,
                "amazon",
            ),
            // OpenAI model
            (
                "gpt-oss-120b",
                "GPT OSS 120B",
                128_000,
                Some(4_096),
                0.001,
                0.003,
                true,
                false,
                "openai",
            ),
        ];

        for (
            id,
            name,
            context_len,
            output_len,
            input_cost,
            output_cost,
            supports_tools,
            supports_multimodal,
            provider,
        ) in default_models
        {
            let model_info = ModelInfo {
                id: id.to_string(),
                name: name.to_string(),
                provider: PROVIDER_NAME.to_string(),
                max_context_length: context_len,
                max_output_length: output_len,
                supports_streaming: true,
                supports_tools,
                supports_multimodal,
                input_cost_per_1k_tokens: Some(input_cost),
                output_cost_per_1k_tokens: Some(output_cost),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: {
                    let mut m = HashMap::new();
                    m.insert(
                        "underlying_provider".to_string(),
                        serde_json::Value::String(provider.to_string()),
                    );
                    m.insert("heroku_managed".to_string(), serde_json::Value::Bool(true));
                    m
                },
            };

            let features = self.detect_features(&model_info);
            let config = ModelConfig {
                max_concurrent_requests: Some(10),
                underlying_provider: Some(provider.to_string()),
                ..Default::default()
            };

            self.models.insert(
                id.to_string(),
                ModelSpec {
                    model_info,
                    features,
                    config,
                },
            );
        }

        // Add embedding model
        let embed_model = ModelInfo {
            id: "cohere-embed-multilingual".to_string(),
            name: "Cohere Embed Multilingual".to_string(),
            provider: PROVIDER_NAME.to_string(),
            max_context_length: 512,
            max_output_length: None,
            supports_streaming: false,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k_tokens: Some(0.0001),
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert(
                    "underlying_provider".to_string(),
                    serde_json::Value::String("cohere".to_string()),
                );
                m.insert(
                    "embedding_dimensions".to_string(),
                    serde_json::Value::Number(1024.into()),
                );
                m
            },
        };

        self.models.insert(
            "cohere-embed-multilingual".to_string(),
            ModelSpec {
                model_info: embed_model,
                features: vec![ModelFeature::Embedding],
                config: ModelConfig {
                    max_concurrent_requests: Some(20),
                    underlying_provider: Some("cohere".to_string()),
                    ..Default::default()
                },
            },
        );

        // Add image generation model
        let image_model = ModelInfo {
            id: "stable-image-ultra".to_string(),
            name: "Stable Image Ultra".to_string(),
            provider: PROVIDER_NAME.to_string(),
            max_context_length: 0,
            max_output_length: None,
            supports_streaming: false,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert(
                    "underlying_provider".to_string(),
                    serde_json::Value::String("stability".to_string()),
                );
                m.insert(
                    "image_generation".to_string(),
                    serde_json::Value::Bool(true),
                );
                m
            },
        };

        self.models.insert(
            "stable-image-ultra".to_string(),
            ModelSpec {
                model_info: image_model,
                features: vec![ModelFeature::ImageGeneration],
                config: ModelConfig {
                    max_concurrent_requests: Some(5),
                    underlying_provider: Some("stability".to_string()),
                    ..Default::default()
                },
            },
        );
    }

    /// Get all models
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        self.models
            .values()
            .map(|spec| spec.model_info.clone())
            .collect()
    }

    /// Get chat models only (excluding embedding and image generation)
    pub fn get_chat_models(&self) -> Vec<ModelInfo> {
        self.models
            .values()
            .filter(|spec| {
                !spec.features.contains(&ModelFeature::Embedding)
                    && !spec.features.contains(&ModelFeature::ImageGeneration)
            })
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

    /// Get the underlying provider for a model
    pub fn get_underlying_provider(&self, model_id: &str) -> Option<&str> {
        self.models
            .get(model_id)
            .and_then(|spec| spec.config.underlying_provider.as_deref())
    }
}

/// Global registry instance
static HEROKU_REGISTRY: OnceLock<HerokuModelRegistry> = OnceLock::new();

/// Get global Heroku model registry
pub fn get_heroku_registry() -> &'static HerokuModelRegistry {
    HEROKU_REGISTRY.get_or_init(HerokuModelRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_creation() {
        let registry = HerokuModelRegistry::new();
        assert!(!registry.get_all_models().is_empty());
    }

    #[test]
    fn test_feature_detection() {
        let registry = get_heroku_registry();
        let models = registry.get_chat_models();

        assert!(!models.is_empty());

        for model in &models {
            assert!(registry.supports_feature(&model.id, &ModelFeature::SystemMessages));
            assert!(registry.supports_feature(&model.id, &ModelFeature::StreamingSupport));
        }
    }

    #[test]
    fn test_models_with_feature() {
        let registry = get_heroku_registry();
        let tool_models = registry.get_models_with_feature(&ModelFeature::ToolCalling);
        assert!(!tool_models.is_empty());
    }

    #[test]
    fn test_claude_models() {
        let registry = get_heroku_registry();

        // Check that Claude models exist
        let claude_models: Vec<_> = registry
            .get_all_models()
            .into_iter()
            .filter(|m| m.id.contains("claude"))
            .collect();

        assert!(!claude_models.is_empty());

        for model in claude_models {
            assert!(registry.supports_feature(&model.id, &ModelFeature::ToolCalling));
        }
    }

    #[test]
    fn test_embedding_model() {
        let registry = get_heroku_registry();
        let embed_models = registry.get_models_with_feature(&ModelFeature::Embedding);

        assert!(!embed_models.is_empty());
        assert!(embed_models.contains(&"cohere-embed-multilingual".to_string()));
    }

    #[test]
    fn test_image_generation_model() {
        let registry = get_heroku_registry();
        let image_models = registry.get_models_with_feature(&ModelFeature::ImageGeneration);

        assert!(!image_models.is_empty());
        assert!(image_models.contains(&"stable-image-ultra".to_string()));
    }

    #[test]
    fn test_default_impl() {
        let registry = HerokuModelRegistry::default();
        assert!(!registry.get_all_models().is_empty());
    }

    #[test]
    fn test_get_model_spec() {
        let registry = get_heroku_registry();
        let spec = registry.get_model_spec("claude-4-5-sonnet");
        assert!(spec.is_some());
        let spec = spec.unwrap();
        assert_eq!(spec.model_info.provider, PROVIDER_NAME);
    }

    #[test]
    fn test_get_model_spec_nonexistent() {
        let registry = get_heroku_registry();
        let spec = registry.get_model_spec("nonexistent-model");
        assert!(spec.is_none());
    }

    #[test]
    fn test_underlying_provider() {
        let registry = get_heroku_registry();

        assert_eq!(
            registry.get_underlying_provider("claude-4-5-sonnet"),
            Some("anthropic")
        );
        assert_eq!(
            registry.get_underlying_provider("amazon-nova-lite"),
            Some("amazon")
        );
        assert_eq!(
            registry.get_underlying_provider("gpt-oss-120b"),
            Some("openai")
        );
        assert_eq!(
            registry.get_underlying_provider("cohere-embed-multilingual"),
            Some("cohere")
        );
    }

    #[test]
    fn test_model_feature_equality() {
        assert_eq!(ModelFeature::FunctionCalling, ModelFeature::FunctionCalling);
        assert_ne!(ModelFeature::FunctionCalling, ModelFeature::Vision);
    }

    #[test]
    fn test_model_info_properties() {
        let registry = get_heroku_registry();
        let models = registry.get_chat_models();

        for model in models {
            assert!(!model.id.is_empty());
            assert!(!model.name.is_empty());
            assert_eq!(model.provider, PROVIDER_NAME);
            assert!(model.max_context_length > 0);
            assert_eq!(model.currency, "USD");
        }
    }

    #[test]
    fn test_global_registry() {
        let registry1 = get_heroku_registry();
        let registry2 = get_heroku_registry();

        assert_eq!(
            registry1.get_all_models().len(),
            registry2.get_all_models().len()
        );
    }

    #[test]
    fn test_heroku_managed_metadata() {
        let registry = get_heroku_registry();
        let models = registry.get_all_models();

        for model in models {
            assert!(
                model.metadata.contains_key("heroku_managed")
                    || model.metadata.contains_key("underlying_provider")
            );
        }
    }
}
