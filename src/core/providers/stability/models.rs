//! Stability AI Models
//!
//! Model registry and information for Stability AI.

use crate::core::types::{ModelInfo, ProviderCapability};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Stability AI model endpoints
pub const STABILITY_ENDPOINTS: &[(&str, &str)] = &[
    ("sd3", "/v2beta/stable-image/generate/sd3"),
    ("sd3-turbo", "/v2beta/stable-image/generate/sd3"),
    ("sd3.5", "/v2beta/stable-image/generate/sd3"),
    ("sd3.5-large", "/v2beta/stable-image/generate/sd3"),
    ("sd3.5-large-turbo", "/v2beta/stable-image/generate/sd3"),
    ("sd3.5-medium", "/v2beta/stable-image/generate/sd3"),
    ("stable-image-ultra", "/v2beta/stable-image/generate/ultra"),
    ("stable-image-core", "/v2beta/stable-image/generate/core"),
];

/// OpenAI size to Stability aspect ratio mapping
pub const SIZE_TO_ASPECT_RATIO: &[(&str, &str)] = &[
    ("1024x1024", "1:1"),
    ("1792x1024", "16:9"),
    ("1024x1792", "9:16"),
    ("512x512", "1:1"),
    ("256x256", "1:1"),
    ("1536x1024", "3:2"),
    ("1024x1536", "2:3"),
    ("1152x896", "4:3"),
    ("896x1152", "3:4"),
    ("1344x768", "16:9"),
    ("768x1344", "9:16"),
];

/// Stability AI model registry
pub struct StabilityModelRegistry {
    models: Vec<ModelInfo>,
    endpoint_map: HashMap<String, String>,
    aspect_ratio_map: HashMap<String, String>,
}

impl StabilityModelRegistry {
    /// Create a new model registry
    pub fn new() -> Self {
        let models = vec![
            ModelInfo {
                id: "stability/sd3".to_string(),
                name: "Stable Diffusion 3".to_string(),
                provider: "stability".to_string(),
                max_context_length: 0,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: true,
                input_cost_per_1k_tokens: None,
                output_cost_per_1k_tokens: None,
                currency: "USD".to_string(),
                capabilities: vec![ProviderCapability::ImageGeneration],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "stability/sd3-turbo".to_string(),
                name: "Stable Diffusion 3 Turbo".to_string(),
                provider: "stability".to_string(),
                max_context_length: 0,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: true,
                input_cost_per_1k_tokens: None,
                output_cost_per_1k_tokens: None,
                currency: "USD".to_string(),
                capabilities: vec![ProviderCapability::ImageGeneration],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "stability/sd3.5-large".to_string(),
                name: "Stable Diffusion 3.5 Large".to_string(),
                provider: "stability".to_string(),
                max_context_length: 0,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: true,
                input_cost_per_1k_tokens: None,
                output_cost_per_1k_tokens: None,
                currency: "USD".to_string(),
                capabilities: vec![ProviderCapability::ImageGeneration],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "stability/sd3.5-large-turbo".to_string(),
                name: "Stable Diffusion 3.5 Large Turbo".to_string(),
                provider: "stability".to_string(),
                max_context_length: 0,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: true,
                input_cost_per_1k_tokens: None,
                output_cost_per_1k_tokens: None,
                currency: "USD".to_string(),
                capabilities: vec![ProviderCapability::ImageGeneration],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "stability/sd3.5-medium".to_string(),
                name: "Stable Diffusion 3.5 Medium".to_string(),
                provider: "stability".to_string(),
                max_context_length: 0,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: true,
                input_cost_per_1k_tokens: None,
                output_cost_per_1k_tokens: None,
                currency: "USD".to_string(),
                capabilities: vec![ProviderCapability::ImageGeneration],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "stability/stable-image-ultra".to_string(),
                name: "Stable Image Ultra".to_string(),
                provider: "stability".to_string(),
                max_context_length: 0,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: true,
                input_cost_per_1k_tokens: None,
                output_cost_per_1k_tokens: None,
                currency: "USD".to_string(),
                capabilities: vec![ProviderCapability::ImageGeneration],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "stability/stable-image-core".to_string(),
                name: "Stable Image Core".to_string(),
                provider: "stability".to_string(),
                max_context_length: 0,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: true,
                input_cost_per_1k_tokens: None,
                output_cost_per_1k_tokens: None,
                currency: "USD".to_string(),
                capabilities: vec![ProviderCapability::ImageGeneration],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
        ];

        let mut endpoint_map = HashMap::new();
        for (model, endpoint) in STABILITY_ENDPOINTS {
            endpoint_map.insert(model.to_string(), endpoint.to_string());
        }

        let mut aspect_ratio_map = HashMap::new();
        for (size, ratio) in SIZE_TO_ASPECT_RATIO {
            aspect_ratio_map.insert(size.to_string(), ratio.to_string());
        }

        Self {
            models,
            endpoint_map,
            aspect_ratio_map,
        }
    }

    /// Get all supported models
    pub fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    /// Get endpoint for a model
    pub fn get_endpoint(&self, model: &str) -> &str {
        // Remove "stability/" prefix if present
        let model_name = model.strip_prefix("stability/").unwrap_or(model);

        self.endpoint_map
            .get(model_name)
            .map(|s| s.as_str())
            .unwrap_or("/v2beta/stable-image/generate/sd3")
    }

    /// Map OpenAI size to Stability aspect ratio
    pub fn size_to_aspect_ratio(&self, size: &str) -> Option<&str> {
        self.aspect_ratio_map.get(size).map(|s| s.as_str())
    }

    /// Check if model is supported
    pub fn supports_model(&self, model: &str) -> bool {
        let model_name = model.strip_prefix("stability/").unwrap_or(model);
        self.endpoint_map.contains_key(model_name) || self.models.iter().any(|m| m.id == model)
    }
}

impl Default for StabilityModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global model registry instance
pub static STABILITY_REGISTRY: LazyLock<StabilityModelRegistry> =
    LazyLock::new(StabilityModelRegistry::new);

/// Get the global Stability model registry
pub fn get_stability_registry() -> &'static StabilityModelRegistry {
    &STABILITY_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_creation() {
        let registry = StabilityModelRegistry::new();
        assert!(!registry.models().is_empty());
    }

    #[test]
    fn test_get_endpoint_sd3() {
        let registry = StabilityModelRegistry::new();
        let endpoint = registry.get_endpoint("sd3");
        assert_eq!(endpoint, "/v2beta/stable-image/generate/sd3");
    }

    #[test]
    fn test_get_endpoint_with_prefix() {
        let registry = StabilityModelRegistry::new();
        let endpoint = registry.get_endpoint("stability/sd3");
        assert_eq!(endpoint, "/v2beta/stable-image/generate/sd3");
    }

    #[test]
    fn test_get_endpoint_ultra() {
        let registry = StabilityModelRegistry::new();
        let endpoint = registry.get_endpoint("stable-image-ultra");
        assert_eq!(endpoint, "/v2beta/stable-image/generate/ultra");
    }

    #[test]
    fn test_get_endpoint_core() {
        let registry = StabilityModelRegistry::new();
        let endpoint = registry.get_endpoint("stable-image-core");
        assert_eq!(endpoint, "/v2beta/stable-image/generate/core");
    }

    #[test]
    fn test_get_endpoint_unknown_defaults_to_sd3() {
        let registry = StabilityModelRegistry::new();
        let endpoint = registry.get_endpoint("unknown-model");
        assert_eq!(endpoint, "/v2beta/stable-image/generate/sd3");
    }

    #[test]
    fn test_size_to_aspect_ratio_square() {
        let registry = StabilityModelRegistry::new();
        assert_eq!(registry.size_to_aspect_ratio("1024x1024"), Some("1:1"));
    }

    #[test]
    fn test_size_to_aspect_ratio_landscape() {
        let registry = StabilityModelRegistry::new();
        assert_eq!(registry.size_to_aspect_ratio("1792x1024"), Some("16:9"));
    }

    #[test]
    fn test_size_to_aspect_ratio_portrait() {
        let registry = StabilityModelRegistry::new();
        assert_eq!(registry.size_to_aspect_ratio("1024x1792"), Some("9:16"));
    }

    #[test]
    fn test_size_to_aspect_ratio_unknown() {
        let registry = StabilityModelRegistry::new();
        assert_eq!(registry.size_to_aspect_ratio("9999x9999"), None);
    }

    #[test]
    fn test_supports_model() {
        let registry = StabilityModelRegistry::new();
        assert!(registry.supports_model("sd3"));
        assert!(registry.supports_model("stability/sd3"));
        assert!(registry.supports_model("stable-image-ultra"));
    }

    #[test]
    fn test_global_registry() {
        let registry = get_stability_registry();
        assert!(!registry.models().is_empty());
    }
}
