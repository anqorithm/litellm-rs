//! OpenRouter Models Registry
//!
//! Model specifications and registry for OpenRouter API

use crate::core::types::ModelInfo;
use std::collections::HashMap;
use std::sync::LazyLock;

/// OpenRouter model features
#[derive(Debug, Clone, PartialEq)]
pub enum OpenRouterModelFeature {
    ChatCompletion,
    Streaming,
    FunctionCalling,
    Vision,
    Json,
}

/// OpenRouter model specification
#[derive(Debug, Clone)]
pub struct OpenRouterModelSpec {
    pub id: String,
    pub name: String,
    pub context_length: usize,
    pub max_output_tokens: Option<usize>,
    pub features: Vec<OpenRouterModelFeature>,
    pub prompt_cost: Option<f64>,     // Cost per 1M input tokens
    pub completion_cost: Option<f64>, // Cost per 1M output tokens
    pub provider: String,             // Underlying provider (e.g. "openai", "anthropic")
}

/// OpenRouter model registry
#[derive(Debug)]
pub struct OpenRouterModelRegistry {
    models: HashMap<String, OpenRouterModelSpec>,
}

impl OpenRouterModelRegistry {
    /// Expected number of OpenRouter models for capacity hint
    const EXPECTED_MODEL_COUNT: usize = 16;

    /// Create a new model registry
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::with_capacity(Self::EXPECTED_MODEL_COUNT),
        };

        registry.register_default_models();
        registry
    }

    /// Register a model
    pub fn register_model(&mut self, spec: OpenRouterModelSpec) {
        self.models.insert(spec.id.clone(), spec);
    }

    /// Get model specification by ID
    pub fn get_model_spec(&self, model_id: &str) -> Option<&OpenRouterModelSpec> {
        // Try exact match first
        if let Some(spec) = self.models.get(model_id) {
            return Some(spec);
        }

        // Try partial match for common aliases
        self.models.values().find(|spec| {
            spec.id.contains(model_id)
                || spec.name.to_lowercase().contains(&model_id.to_lowercase())
        })
    }

    /// Get all models
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        self.models
            .values()
            .map(|spec| ModelInfo {
                id: spec.id.clone(),
                name: spec.name.clone(),
                provider: "openrouter".to_string(),
                max_context_length: spec.context_length as u32,
                max_output_length: spec.max_output_tokens.map(|t| t as u32),
                supports_streaming: spec.features.contains(&OpenRouterModelFeature::Streaming),
                supports_tools: spec
                    .features
                    .contains(&OpenRouterModelFeature::FunctionCalling),
                supports_multimodal: spec.features.contains(&OpenRouterModelFeature::Vision),
                input_cost_per_1k_tokens: spec.prompt_cost.map(|cost| cost / 1_000.0),
                output_cost_per_1k_tokens: spec.completion_cost.map(|cost| cost / 1_000.0),
                currency: "USD".to_string(),
                capabilities: {
                    let mut caps = vec![crate::core::types::ProviderCapability::ChatCompletion];
                    if spec.features.contains(&OpenRouterModelFeature::Streaming) {
                        caps.push(crate::core::types::ProviderCapability::ChatCompletionStream);
                    }
                    if spec
                        .features
                        .contains(&OpenRouterModelFeature::FunctionCalling)
                    {
                        caps.push(crate::core::types::ProviderCapability::FunctionCalling);
                    }
                    caps
                },
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            })
            .collect()
    }

    /// Register default OpenRouter models
    fn register_default_models(&mut self) {
        // OpenAI models via OpenRouter
        self.register_model(OpenRouterModelSpec {
            id: "openai/gpt-4".to_string(),
            name: "GPT-4".to_string(),
            context_length: 8192,
            max_output_tokens: Some(4096),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::FunctionCalling,
            ],
            prompt_cost: Some(30.0),
            completion_cost: Some(60.0),
            provider: "openai".to_string(),
        });

        self.register_model(OpenRouterModelSpec {
            id: "openai/gpt-4-turbo".to_string(),
            name: "GPT-4 Turbo".to_string(),
            context_length: 128000,
            max_output_tokens: Some(4096),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::FunctionCalling,
                OpenRouterModelFeature::Vision,
                OpenRouterModelFeature::Json,
            ],
            prompt_cost: Some(10.0),
            completion_cost: Some(30.0),
            provider: "openai".to_string(),
        });

        self.register_model(OpenRouterModelSpec {
            id: "openai/gpt-3.5-turbo".to_string(),
            name: "GPT-3.5 Turbo".to_string(),
            context_length: 16385,
            max_output_tokens: Some(4096),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::FunctionCalling,
            ],
            prompt_cost: Some(0.5),
            completion_cost: Some(1.5),
            provider: "openai".to_string(),
        });

        // Anthropic models via OpenRouter
        self.register_model(OpenRouterModelSpec {
            id: "anthropic/claude-opus-4.6".to_string(),
            name: "Claude Opus 4.6".to_string(),
            context_length: 1_000_000,
            max_output_tokens: Some(32_000),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::FunctionCalling,
                OpenRouterModelFeature::Vision,
            ],
            prompt_cost: Some(5.0),
            completion_cost: Some(25.0),
            provider: "anthropic".to_string(),
        });

        self.register_model(OpenRouterModelSpec {
            id: "anthropic/claude-opus-4.5".to_string(),
            name: "Claude Opus 4.5".to_string(),
            context_length: 200000,
            max_output_tokens: Some(32_000),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::FunctionCalling,
                OpenRouterModelFeature::Vision,
            ],
            prompt_cost: Some(5.0),
            completion_cost: Some(25.0),
            provider: "anthropic".to_string(),
        });

        self.register_model(OpenRouterModelSpec {
            id: "anthropic/claude-sonnet-4.5".to_string(),
            name: "Claude Sonnet 4.5".to_string(),
            context_length: 200000,
            max_output_tokens: Some(16_000),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::FunctionCalling,
                OpenRouterModelFeature::Vision,
            ],
            prompt_cost: Some(3.0),
            completion_cost: Some(15.0),
            provider: "anthropic".to_string(),
        });

        self.register_model(OpenRouterModelSpec {
            id: "anthropic/claude-sonnet-4".to_string(),
            name: "Claude Sonnet 4".to_string(),
            context_length: 200000,
            max_output_tokens: Some(16_000),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::FunctionCalling,
                OpenRouterModelFeature::Vision,
            ],
            prompt_cost: Some(3.0),
            completion_cost: Some(15.0),
            provider: "anthropic".to_string(),
        });

        self.register_model(OpenRouterModelSpec {
            id: "anthropic/claude-3-opus".to_string(),
            name: "Claude 3 Opus".to_string(),
            context_length: 200000,
            max_output_tokens: Some(4096),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::Vision,
            ],
            prompt_cost: Some(15.0),
            completion_cost: Some(75.0),
            provider: "anthropic".to_string(),
        });

        self.register_model(OpenRouterModelSpec {
            id: "anthropic/claude-3-sonnet".to_string(),
            name: "Claude 3 Sonnet".to_string(),
            context_length: 200000,
            max_output_tokens: Some(4096),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::Vision,
            ],
            prompt_cost: Some(3.0),
            completion_cost: Some(15.0),
            provider: "anthropic".to_string(),
        });

        self.register_model(OpenRouterModelSpec {
            id: "anthropic/claude-3-haiku".to_string(),
            name: "Claude 3 Haiku".to_string(),
            context_length: 200000,
            max_output_tokens: Some(4096),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::Vision,
            ],
            prompt_cost: Some(0.25),
            completion_cost: Some(1.25),
            provider: "anthropic".to_string(),
        });

        // Google models via OpenRouter
        self.register_model(OpenRouterModelSpec {
            id: "google/gemini-pro".to_string(),
            name: "Gemini Pro".to_string(),
            context_length: 91728,
            max_output_tokens: Some(8192),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::Vision,
            ],
            prompt_cost: Some(0.5),
            completion_cost: Some(1.5),
            provider: "google".to_string(),
        });

        // Meta models via OpenRouter
        self.register_model(OpenRouterModelSpec {
            id: "meta-llama/llama-3-70b-instruct".to_string(),
            name: "Llama 3 70B Instruct".to_string(),
            context_length: 8192,
            max_output_tokens: Some(4096),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
            ],
            prompt_cost: Some(0.59),
            completion_cost: Some(0.79),
            provider: "meta".to_string(),
        });

        // DeepSeek models via OpenRouter
        self.register_model(OpenRouterModelSpec {
            id: "deepseek/deepseek-chat".to_string(),
            name: "DeepSeek Chat".to_string(),
            context_length: 32768,
            max_output_tokens: Some(4096),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::FunctionCalling,
                OpenRouterModelFeature::Json,
            ],
            prompt_cost: Some(0.14),
            completion_cost: Some(0.28),
            provider: "deepseek".to_string(),
        });

        // Mistral models via OpenRouter
        self.register_model(OpenRouterModelSpec {
            id: "mistralai/mistral-large".to_string(),
            name: "Mistral Large".to_string(),
            context_length: 128000,
            max_output_tokens: Some(4096),
            features: vec![
                OpenRouterModelFeature::ChatCompletion,
                OpenRouterModelFeature::Streaming,
                OpenRouterModelFeature::FunctionCalling,
                OpenRouterModelFeature::Json,
            ],
            prompt_cost: Some(3.0),
            completion_cost: Some(9.0),
            provider: "mistral".to_string(),
        });
    }
}

impl Default for OpenRouterModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global OpenRouter model registry instance
pub static OPENROUTER_REGISTRY: LazyLock<OpenRouterModelRegistry> =
    LazyLock::new(OpenRouterModelRegistry::new);

/// Get global OpenRouter model registry
pub fn get_openrouter_registry() -> &'static OpenRouterModelRegistry {
    &OPENROUTER_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry() {
        let registry = OpenRouterModelRegistry::new();

        // Test exact match
        let spec = registry.get_model_spec("openai/gpt-4");
        assert!(spec.is_some());
        assert_eq!(spec.unwrap().name, "GPT-4");

        // Test partial match
        let spec = registry.get_model_spec("gpt-4");
        assert!(spec.is_some());

        // Test non-existent model
        let spec = registry.get_model_spec("non-existent-model");
        assert!(spec.is_none());
    }

    #[test]
    fn test_get_all_models() {
        let registry = OpenRouterModelRegistry::new();
        let models = registry.get_all_models();

        assert!(!models.is_empty());

        // Check that all models have required fields
        for model in models {
            assert!(!model.id.is_empty());
            assert!(!model.name.is_empty());
            assert_eq!(model.provider, "openrouter");
            assert!(model.max_context_length > 0);
            assert_eq!(model.currency, "USD");
        }
    }

    #[test]
    fn test_model_features() {
        let registry = OpenRouterModelRegistry::new();
        let spec = registry.get_model_spec("openai/gpt-4-turbo").unwrap();

        assert!(
            spec.features
                .contains(&OpenRouterModelFeature::ChatCompletion)
        );
        assert!(spec.features.contains(&OpenRouterModelFeature::Streaming));
        assert!(
            spec.features
                .contains(&OpenRouterModelFeature::FunctionCalling)
        );
        assert!(spec.features.contains(&OpenRouterModelFeature::Vision));
        assert!(spec.features.contains(&OpenRouterModelFeature::Json));
    }

    #[test]
    fn test_global_registry() {
        let registry = get_openrouter_registry();
        assert!(registry.get_model_spec("openai/gpt-4").is_some());
    }

    #[test]
    fn test_anthropic_models() {
        let registry = OpenRouterModelRegistry::new();

        let claude_opus_46 = registry.get_model_spec("anthropic/claude-opus-4.6");
        assert!(claude_opus_46.is_some());

        let claude_sonnet_45 = registry.get_model_spec("anthropic/claude-sonnet-4.5");
        assert!(claude_sonnet_45.is_some());

        let claude_opus = registry.get_model_spec("anthropic/claude-3-opus");
        assert!(claude_opus.is_some());
        let spec = claude_opus.unwrap();
        assert_eq!(spec.provider, "anthropic");
        assert_eq!(spec.context_length, 200000);
        assert!(spec.features.contains(&OpenRouterModelFeature::Vision));

        let claude_sonnet = registry.get_model_spec("anthropic/claude-3-sonnet");
        assert!(claude_sonnet.is_some());

        let claude_haiku = registry.get_model_spec("anthropic/claude-3-haiku");
        assert!(claude_haiku.is_some());
    }

    #[test]
    fn test_google_models() {
        let registry = OpenRouterModelRegistry::new();

        let gemini = registry.get_model_spec("google/gemini-pro");
        assert!(gemini.is_some());
        let spec = gemini.unwrap();
        assert_eq!(spec.provider, "google");
        assert!(spec.features.contains(&OpenRouterModelFeature::Vision));
    }

    #[test]
    fn test_meta_models() {
        let registry = OpenRouterModelRegistry::new();

        let llama = registry.get_model_spec("meta-llama/llama-3-70b-instruct");
        assert!(llama.is_some());
        let spec = llama.unwrap();
        assert_eq!(spec.provider, "meta");
        assert!(
            !spec
                .features
                .contains(&OpenRouterModelFeature::FunctionCalling)
        );
    }

    #[test]
    fn test_deepseek_models() {
        let registry = OpenRouterModelRegistry::new();

        let deepseek = registry.get_model_spec("deepseek/deepseek-chat");
        assert!(deepseek.is_some());
        let spec = deepseek.unwrap();
        assert_eq!(spec.provider, "deepseek");
        assert!(
            spec.features
                .contains(&OpenRouterModelFeature::FunctionCalling)
        );
        assert!(spec.features.contains(&OpenRouterModelFeature::Json));
    }

    #[test]
    fn test_mistral_models() {
        let registry = OpenRouterModelRegistry::new();

        let mistral = registry.get_model_spec("mistralai/mistral-large");
        assert!(mistral.is_some());
        let spec = mistral.unwrap();
        assert_eq!(spec.provider, "mistral");
        assert_eq!(spec.context_length, 128000);
    }

    #[test]
    fn test_model_pricing() {
        let registry = OpenRouterModelRegistry::new();

        // GPT-4 should have pricing
        let gpt4 = registry.get_model_spec("openai/gpt-4").unwrap();
        assert!(gpt4.prompt_cost.is_some());
        assert!(gpt4.completion_cost.is_some());

        // Check cost conversion in ModelInfo
        let models = registry.get_all_models();
        let gpt4_info = models.iter().find(|m| m.id == "openai/gpt-4").unwrap();
        // 30.0 per 1M => 0.03 per 1k
        assert!(gpt4_info.input_cost_per_1k_tokens.is_some());
    }

    #[test]
    fn test_register_custom_model() {
        let mut registry = OpenRouterModelRegistry::new();

        let custom_model = OpenRouterModelSpec {
            id: "custom/my-model".to_string(),
            name: "My Custom Model".to_string(),
            context_length: 4096,
            max_output_tokens: Some(1024),
            features: vec![OpenRouterModelFeature::ChatCompletion],
            prompt_cost: Some(1.0),
            completion_cost: Some(2.0),
            provider: "custom".to_string(),
        };

        registry.register_model(custom_model);

        let spec = registry.get_model_spec("custom/my-model");
        assert!(spec.is_some());
        assert_eq!(spec.unwrap().name, "My Custom Model");
    }

    #[test]
    fn test_partial_match_by_name() {
        let registry = OpenRouterModelRegistry::new();

        // Match by name (case insensitive)
        let spec = registry.get_model_spec("claude 3 opus");
        assert!(spec.is_some());
    }

    #[test]
    fn test_model_info_capabilities() {
        let registry = OpenRouterModelRegistry::new();
        let models = registry.get_all_models();

        // GPT-4 turbo should have function calling capability
        let gpt4_turbo = models
            .iter()
            .find(|m| m.id == "openai/gpt-4-turbo")
            .unwrap();
        assert!(gpt4_turbo.supports_tools);
        assert!(gpt4_turbo.supports_streaming);
        assert!(gpt4_turbo.supports_multimodal);

        // Llama should not have function calling
        let llama = models
            .iter()
            .find(|m| m.id == "meta-llama/llama-3-70b-instruct")
            .unwrap();
        assert!(!llama.supports_tools);
    }

    #[test]
    fn test_default_impl() {
        let registry = OpenRouterModelRegistry::default();
        assert!(registry.get_model_spec("openai/gpt-4").is_some());
    }
}
