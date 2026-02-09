//! Gemini Model Registry
//!
//! Unified model registry system containing capabilities and pricing information for all Gemini models

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::types::model::ModelInfo;

mod models_catalog;

/// Model features
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelFeature {
    /// Multimodal support (images, videos, audio)
    MultimodalSupport,
    /// Tool calling support
    ToolCalling,
    /// Function calling support
    FunctionCalling,
    /// Streaming support
    StreamingSupport,
    /// Context caching support
    ContextCaching,
    /// System instructions support
    SystemInstructions,
    /// Batch processing support
    BatchProcessing,
    /// JSON mode support
    JsonMode,
    /// Code execution support
    CodeExecution,
    /// Search grounding support
    SearchGrounding,
    /// Video understanding support
    VideoUnderstanding,
    /// Audio understanding support  
    AudioUnderstanding,
    /// Real-time streaming support
    RealtimeStreaming,
}

/// Model family classification
#[derive(Debug, Clone, PartialEq)]
pub enum GeminiModelFamily {
    /// Gemini 3.0 series (2025 - Latest)
    Gemini3Pro,
    Gemini3ProDeepThink,
    Gemini3Flash,
    Gemini3ProImage,

    /// Gemini 2.5 series (2025)
    Gemini25Pro,
    Gemini25Flash,
    Gemini25FlashLite,

    /// Gemini 2.0 series
    Gemini20Flash,
    Gemini20FlashThinking,

    /// Gemini 1.5 series
    Gemini15Pro,
    Gemini15Flash,
    Gemini15Flash8B,

    /// Gemini 1.0 series
    Gemini10Pro,
    Gemini10ProVision,

    /// Experimental models
    GeminiExperimental,
}

/// Model
#[derive(Debug, Clone)]
pub struct ModelPricing {
    /// Input token price (USD per million tokens)
    pub input_price: f64,
    /// Output token price (USD per million tokens)
    pub output_price: f64,
    /// Cached input price (optional)
    pub cached_input_price: Option<f64>,
    /// Image price (per image)
    pub image_price: Option<f64>,
    /// Video price (per second)
    pub video_price_per_second: Option<f64>,
    /// Audio price (per second)
    pub audio_price_per_second: Option<f64>,
}

/// Model limits
#[derive(Debug, Clone)]
pub struct ModelLimits {
    /// Maximum context length
    pub max_context_length: u32,
    /// Maximum output tokens
    pub max_output_tokens: u32,
    /// Maximum image count
    pub max_images: Option<u32>,
    /// Maximum video length (seconds)
    pub max_video_seconds: Option<u32>,
    /// Maximum audio length (seconds)
    pub max_audio_seconds: Option<u32>,
    /// Requests per minute limit
    pub rpm_limit: Option<u32>,
    /// Tokens per minute limit
    pub tpm_limit: Option<u32>,
}

/// Model
#[derive(Debug, Clone)]
pub struct ModelSpec {
    /// Model
    pub model_info: ModelInfo,
    /// Model
    pub family: GeminiModelFamily,
    /// Supported features
    pub features: Vec<ModelFeature>,
    /// Pricing information
    pub pricing: ModelPricing,
    /// Limit information
    pub limits: ModelLimits,
}

/// Model
#[derive(Debug, Clone)]
pub struct GeminiModelRegistry {
    models: HashMap<String, ModelSpec>,
}

impl GeminiModelRegistry {
    /// Expected number of Gemini models for capacity hint
    const EXPECTED_MODEL_COUNT: usize = 12;

    /// Create
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::with_capacity(Self::EXPECTED_MODEL_COUNT),
        };
        registry.initialize_models();
        registry
    }

    /// Initialize all Gemini models
    fn initialize_models(&mut self) {
        models_catalog::initialize_models(self);
    }

    /// Model
    fn register_model(&mut self, id: &str, spec: ModelSpec) {
        self.models.insert(id.to_string(), spec);
    }

    /// Model
    pub fn get_model_spec(&self, model_id: &str) -> Option<&ModelSpec> {
        self.models.get(model_id)
    }

    /// Model
    pub fn list_models(&self) -> Vec<&ModelSpec> {
        self.models.values().collect()
    }

    /// Check
    pub fn supports_feature(&self, model_id: &str, feature: &ModelFeature) -> bool {
        self.get_model_spec(model_id)
            .map(|spec| spec.features.contains(feature))
            .unwrap_or(false)
    }

    /// Model
    pub fn get_model_family(&self, model_id: &str) -> Option<&GeminiModelFamily> {
        self.get_model_spec(model_id).map(|spec| &spec.family)
    }

    /// Model
    pub fn get_model_pricing(&self, model_id: &str) -> Option<&ModelPricing> {
        self.get_model_spec(model_id).map(|spec| &spec.pricing)
    }

    /// Model
    pub fn get_model_limits(&self, model_id: &str) -> Option<&ModelLimits> {
        self.get_model_spec(model_id).map(|spec| &spec.limits)
    }

    /// Detect model family from model name string
    pub fn from_model_name(model_name: &str) -> Option<GeminiModelFamily> {
        let model_lower = model_name.to_lowercase();

        // Gemini 3.0 series (check first as most specific)
        if model_lower.contains("gemini-3") && model_lower.contains("deep-think") {
            Some(GeminiModelFamily::Gemini3ProDeepThink)
        } else if model_lower.contains("gemini-3") && model_lower.contains("image") {
            Some(GeminiModelFamily::Gemini3ProImage)
        } else if model_lower.contains("gemini-3-flash") || model_lower.contains("gemini-3.0-flash")
        {
            Some(GeminiModelFamily::Gemini3Flash)
        } else if model_lower.contains("gemini-3-pro") || model_lower.contains("gemini-3.0-pro") {
            Some(GeminiModelFamily::Gemini3Pro)
        }
        // Gemini 2.5 series
        else if model_lower.contains("gemini-2.5-flash-lite") {
            Some(GeminiModelFamily::Gemini25FlashLite)
        } else if model_lower.contains("gemini-2.5-flash") {
            Some(GeminiModelFamily::Gemini25Flash)
        } else if model_lower.contains("gemini-2.5-pro") {
            Some(GeminiModelFamily::Gemini25Pro)
        }
        // Gemini 2.0 series
        else if model_lower.contains("gemini-2.0-flash-thinking") {
            Some(GeminiModelFamily::Gemini20FlashThinking)
        } else if model_lower.contains("gemini-2.0-flash") || model_lower.contains("gemini-2-flash")
        {
            Some(GeminiModelFamily::Gemini20Flash)
        }
        // Gemini 1.5 series
        else if model_lower.contains("gemini-1.5-pro") || model_lower.contains("gemini-15-pro") {
            Some(GeminiModelFamily::Gemini15Pro)
        } else if model_lower.contains("gemini-1.5-flash-8b") {
            Some(GeminiModelFamily::Gemini15Flash8B)
        } else if model_lower.contains("gemini-1.5-flash")
            || model_lower.contains("gemini-15-flash")
        {
            Some(GeminiModelFamily::Gemini15Flash)
        }
        // Gemini 1.0 series
        else if model_lower.contains("gemini-1.0-pro-vision") {
            Some(GeminiModelFamily::Gemini10ProVision)
        } else if model_lower.contains("gemini-1.0-pro") || model_lower.contains("gemini-pro") {
            Some(GeminiModelFamily::Gemini10Pro)
        }
        // Experimental
        else if model_lower.contains("gemini-exp") {
            Some(GeminiModelFamily::GeminiExperimental)
        } else {
            None
        }
    }
}

impl Default for GeminiModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Model
pub fn get_gemini_registry() -> &'static GeminiModelRegistry {
    static REGISTRY: OnceLock<GeminiModelRegistry> = OnceLock::new();
    REGISTRY.get_or_init(GeminiModelRegistry::new)
}

/// Cost calculation utility
pub struct CostCalculator;

impl CostCalculator {
    /// Calculate basic cost
    pub fn calculate_cost(
        model_id: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> Option<f64> {
        let registry = get_gemini_registry();
        let pricing = registry.get_model_pricing(model_id)?;

        let input_cost = (prompt_tokens as f64 / 1_000_000.0) * pricing.input_price;
        let output_cost = (completion_tokens as f64 / 1_000_000.0) * pricing.output_price;

        Some(input_cost + output_cost)
    }

    /// Calculate multimodal cost
    pub fn calculate_multimodal_cost(
        model_id: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        cached_tokens: Option<u32>,
        images: Option<u32>,
        video_seconds: Option<u32>,
        audio_seconds: Option<u32>,
    ) -> Option<f64> {
        let registry = get_gemini_registry();
        let pricing = registry.get_model_pricing(model_id)?;

        let mut total_cost = 0.0;
        let mut remaining_prompt_tokens = prompt_tokens;

        // Handle
        if let (Some(cached), Some(cached_price)) = (cached_tokens, pricing.cached_input_price) {
            let cached_cost = (cached as f64 / 1_000_000.0) * cached_price;
            total_cost += cached_cost;
            remaining_prompt_tokens = remaining_prompt_tokens.saturating_sub(cached);
        }

        // Regular input tokens
        let input_cost = (remaining_prompt_tokens as f64 / 1_000_000.0) * pricing.input_price;
        total_cost += input_cost;

        // Output tokens
        let output_cost = (completion_tokens as f64 / 1_000_000.0) * pricing.output_price;
        total_cost += output_cost;

        // Image cost
        if let (Some(img_count), Some(img_price)) = (images, pricing.image_price) {
            total_cost += img_count as f64 * img_price;
        }

        // Video cost
        if let (Some(video_secs), Some(video_price)) =
            (video_seconds, pricing.video_price_per_second)
        {
            total_cost += video_secs as f64 * video_price;
        }

        // Audio cost
        if let (Some(audio_secs), Some(audio_price)) =
            (audio_seconds, pricing.audio_price_per_second)
        {
            total_cost += audio_secs as f64 * audio_price;
        }

        Some(total_cost)
    }

    /// Estimate token count
    pub fn estimate_tokens(text: &str) -> u32 {
        // Gemini uses approximately 4 characters = 1 token ratio (English)
        (text.len() as f32 / 4.0).ceil() as u32
    }
}

#[cfg(test)]
#[path = "models_tests.rs"]
mod tests;
