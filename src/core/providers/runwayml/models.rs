//! Runway ML Models
//!
//! Model registry and information for Runway ML video and image generation.

use crate::core::types::{ModelInfo, ProviderCapability};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Runway ML model types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunwayModelType {
    /// Gen-3 Alpha for high-quality video generation
    Gen3Alpha,
    /// Gen-3 Alpha Turbo for faster video generation
    Gen3AlphaTurbo,
    /// Image-to-video generation
    ImageToVideo,
}

/// Runway ML task types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunwayTaskType {
    /// Text-to-video generation
    TextToVideo,
    /// Image-to-video generation
    ImageToVideo,
    /// Video upscaling
    Upscale,
}

impl RunwayTaskType {
    /// Get the API task name for this type
    pub fn api_name(&self) -> &'static str {
        match self {
            RunwayTaskType::TextToVideo => "gen3a_turbo",
            RunwayTaskType::ImageToVideo => "gen3a_turbo",
            RunwayTaskType::Upscale => "upscale",
        }
    }
}

/// Runway ML model specification
#[derive(Debug, Clone)]
pub struct RunwayModelSpec {
    /// Model info
    pub model_info: ModelInfo,
    /// Model type
    pub model_type: RunwayModelType,
    /// API model name
    pub api_model: &'static str,
    /// Supported task types
    pub supported_tasks: Vec<RunwayTaskType>,
    /// Maximum video duration in seconds
    pub max_duration: u32,
    /// Supported resolutions
    pub supported_resolutions: Vec<&'static str>,
}

/// Runway ML model registry
pub struct RunwayMLModelRegistry {
    models: Vec<ModelInfo>,
    model_specs: HashMap<String, RunwayModelSpec>,
}

impl RunwayMLModelRegistry {
    /// Create a new model registry
    pub fn new() -> Self {
        let mut model_specs = HashMap::new();

        // Gen-3 Alpha Turbo - Fast video generation
        let gen3_alpha_turbo = RunwayModelSpec {
            model_info: ModelInfo {
                id: "runwayml/gen3a_turbo".to_string(),
                name: "Gen-3 Alpha Turbo".to_string(),
                provider: "runwayml".to_string(),
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
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("type".to_string(), serde_json::json!("video"));
                    m.insert("cost_per_second".to_string(), serde_json::json!("0.05"));
                    m
                },
            },
            model_type: RunwayModelType::Gen3AlphaTurbo,
            api_model: "gen3a_turbo",
            supported_tasks: vec![RunwayTaskType::TextToVideo, RunwayTaskType::ImageToVideo],
            max_duration: 10,
            supported_resolutions: vec!["720p", "1080p"],
        };
        model_specs.insert("runwayml/gen3a_turbo".to_string(), gen3_alpha_turbo.clone());
        model_specs.insert("gen3a_turbo".to_string(), gen3_alpha_turbo);

        // Gen-3 Alpha - High quality video generation
        let gen3_alpha = RunwayModelSpec {
            model_info: ModelInfo {
                id: "runwayml/gen3a".to_string(),
                name: "Gen-3 Alpha".to_string(),
                provider: "runwayml".to_string(),
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
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("type".to_string(), serde_json::json!("video"));
                    m.insert("cost_per_second".to_string(), serde_json::json!("0.10"));
                    m
                },
            },
            model_type: RunwayModelType::Gen3Alpha,
            api_model: "gen3a",
            supported_tasks: vec![RunwayTaskType::TextToVideo, RunwayTaskType::ImageToVideo],
            max_duration: 10,
            supported_resolutions: vec!["720p", "1080p"],
        };
        model_specs.insert("runwayml/gen3a".to_string(), gen3_alpha.clone());
        model_specs.insert("gen3a".to_string(), gen3_alpha);

        // Image-to-Video model
        let img2vid = RunwayModelSpec {
            model_info: ModelInfo {
                id: "runwayml/image_to_video".to_string(),
                name: "Image to Video".to_string(),
                provider: "runwayml".to_string(),
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
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("type".to_string(), serde_json::json!("video"));
                    m.insert("input_type".to_string(), serde_json::json!("image"));
                    m
                },
            },
            model_type: RunwayModelType::ImageToVideo,
            api_model: "gen3a_turbo",
            supported_tasks: vec![RunwayTaskType::ImageToVideo],
            max_duration: 10,
            supported_resolutions: vec!["720p", "1080p"],
        };
        model_specs.insert("runwayml/image_to_video".to_string(), img2vid.clone());
        model_specs.insert("image_to_video".to_string(), img2vid);

        // Collect all model infos
        let models: Vec<ModelInfo> = model_specs
            .values()
            .filter(|s| s.model_info.id.starts_with("runwayml/"))
            .map(|s| s.model_info.clone())
            .collect();

        Self {
            models,
            model_specs,
        }
    }

    /// Get all supported models
    pub fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    /// Get model specification by ID
    pub fn get_model_spec(&self, model: &str) -> Option<&RunwayModelSpec> {
        // Try with prefix first
        let model_name = model.strip_prefix("runwayml/").unwrap_or(model);
        self.model_specs
            .get(model_name)
            .or_else(|| self.model_specs.get(model))
    }

    /// Get the API model name for a given model ID
    pub fn get_api_model(&self, model: &str) -> &str {
        self.get_model_spec(model)
            .map(|s| s.api_model)
            .unwrap_or("gen3a_turbo") // Default to Gen-3 Alpha Turbo
    }

    /// Check if a model supports a specific task type
    pub fn supports_task(&self, model: &str, task: RunwayTaskType) -> bool {
        self.get_model_spec(model)
            .map(|s| s.supported_tasks.contains(&task))
            .unwrap_or(false)
    }

    /// Get maximum duration for a model
    pub fn get_max_duration(&self, model: &str) -> u32 {
        self.get_model_spec(model)
            .map(|s| s.max_duration)
            .unwrap_or(10)
    }

    /// Check if a model is supported
    pub fn supports_model(&self, model: &str) -> bool {
        self.get_model_spec(model).is_some()
    }

    /// Get model type
    pub fn get_model_type(&self, model: &str) -> Option<RunwayModelType> {
        self.get_model_spec(model).map(|s| s.model_type)
    }
}

impl Default for RunwayMLModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global model registry instance
pub static RUNWAYML_REGISTRY: Lazy<RunwayMLModelRegistry> = Lazy::new(RunwayMLModelRegistry::new);

/// Get the global Runway ML model registry
pub fn get_runwayml_registry() -> &'static RunwayMLModelRegistry {
    &RUNWAYML_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_creation() {
        let registry = RunwayMLModelRegistry::new();
        assert!(!registry.models().is_empty());
    }

    #[test]
    fn test_get_model_spec_with_prefix() {
        let registry = RunwayMLModelRegistry::new();
        let spec = registry.get_model_spec("runwayml/gen3a_turbo");
        assert!(spec.is_some());
        assert_eq!(spec.unwrap().api_model, "gen3a_turbo");
    }

    #[test]
    fn test_get_model_spec_without_prefix() {
        let registry = RunwayMLModelRegistry::new();
        let spec = registry.get_model_spec("gen3a_turbo");
        assert!(spec.is_some());
    }

    #[test]
    fn test_get_api_model() {
        let registry = RunwayMLModelRegistry::new();
        assert_eq!(
            registry.get_api_model("runwayml/gen3a_turbo"),
            "gen3a_turbo"
        );
        assert_eq!(registry.get_api_model("gen3a"), "gen3a");
    }

    #[test]
    fn test_get_api_model_unknown() {
        let registry = RunwayMLModelRegistry::new();
        assert_eq!(registry.get_api_model("unknown_model"), "gen3a_turbo");
    }

    #[test]
    fn test_supports_task() {
        let registry = RunwayMLModelRegistry::new();
        assert!(registry.supports_task("gen3a_turbo", RunwayTaskType::TextToVideo));
        assert!(registry.supports_task("gen3a_turbo", RunwayTaskType::ImageToVideo));
    }

    #[test]
    fn test_get_max_duration() {
        let registry = RunwayMLModelRegistry::new();
        assert_eq!(registry.get_max_duration("gen3a_turbo"), 10);
    }

    #[test]
    fn test_supports_model() {
        let registry = RunwayMLModelRegistry::new();
        assert!(registry.supports_model("gen3a_turbo"));
        assert!(registry.supports_model("runwayml/gen3a"));
        assert!(!registry.supports_model("unknown_model"));
    }

    #[test]
    fn test_get_model_type() {
        let registry = RunwayMLModelRegistry::new();
        assert_eq!(
            registry.get_model_type("gen3a_turbo"),
            Some(RunwayModelType::Gen3AlphaTurbo)
        );
        assert_eq!(
            registry.get_model_type("gen3a"),
            Some(RunwayModelType::Gen3Alpha)
        );
    }

    #[test]
    fn test_global_registry() {
        let registry = get_runwayml_registry();
        assert!(!registry.models().is_empty());
    }

    #[test]
    fn test_model_info_capabilities() {
        let registry = RunwayMLModelRegistry::new();
        for model in registry.models() {
            assert!(
                model
                    .capabilities
                    .contains(&ProviderCapability::ImageGeneration)
            );
        }
    }

    #[test]
    fn test_task_type_api_name() {
        assert_eq!(RunwayTaskType::TextToVideo.api_name(), "gen3a_turbo");
        assert_eq!(RunwayTaskType::ImageToVideo.api_name(), "gen3a_turbo");
        assert_eq!(RunwayTaskType::Upscale.api_name(), "upscale");
    }
}
