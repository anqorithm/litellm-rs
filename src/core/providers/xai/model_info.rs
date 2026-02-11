//! xAI Model Information
//!
//! Model configurations for Grok models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// xAI model identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum XAIModel {
    // Grok 4 models (2025 - Latest)
    Grok4,
    // Grok 3 models (2025)
    Grok3,
    Grok3Mini,
    Grok3Fast,
    // Grok 2 models (2024)
    Grok2,
    Grok2Mini,
    Grok21212,   // Grok 2 (December 2024 update)
    Grok2Vision, // Grok 2 with vision
    // Experimental
    GrokBeta,
    GrokVision,
}

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model ID as used in API
    pub model_id: &'static str,
    /// Display name
    pub display_name: &'static str,
    /// Maximum context length
    pub max_context_length: u32,
    /// Maximum output tokens
    pub max_output_length: u32,
    /// Whether the model supports tools/functions
    pub supports_tools: bool,
    /// Whether the model supports vision
    pub supports_multimodal: bool,
    /// Whether the model supports web search
    pub supports_web_search: bool,
    /// Whether the model has reasoning capabilities
    pub supports_reasoning: bool,
    /// Input cost per million tokens (in USD)
    pub input_cost_per_million: f64,
    /// Output cost per million tokens (in USD)
    pub output_cost_per_million: f64,
    /// Reasoning tokens cost per million (if applicable)
    pub reasoning_cost_per_million: Option<f64>,
}

/// Static model configurations
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelInfo>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    // ==================== Grok 4 (2025 - Latest) ====================
    configs.insert(
        "grok-4",
        ModelInfo {
            model_id: "grok-4",
            display_name: "Grok 4",
            max_context_length: 256000, // 256K context
            max_output_length: 128000,
            supports_tools: true,
            supports_multimodal: true,
            supports_web_search: true,
            supports_reasoning: true,
            input_cost_per_million: 3.0,
            output_cost_per_million: 15.0,
            reasoning_cost_per_million: Some(15.0),
        },
    );

    // ==================== Grok 3 (2025) ====================
    configs.insert(
        "grok-3",
        ModelInfo {
            model_id: "grok-3",
            display_name: "Grok 3",
            max_context_length: 131072, // 128K context
            max_output_length: 65536,
            supports_tools: true,
            supports_multimodal: true,
            supports_web_search: true,
            supports_reasoning: true,
            input_cost_per_million: 3.0,
            output_cost_per_million: 15.0,
            reasoning_cost_per_million: Some(10.0),
        },
    );

    configs.insert(
        "grok-3-mini",
        ModelInfo {
            model_id: "grok-3-mini",
            display_name: "Grok 3 Mini",
            max_context_length: 131072,
            max_output_length: 32768,
            supports_tools: true,
            supports_multimodal: false,
            supports_web_search: true,
            supports_reasoning: true,
            input_cost_per_million: 0.3,
            output_cost_per_million: 0.5,
            reasoning_cost_per_million: Some(0.5),
        },
    );

    configs.insert(
        "grok-3-fast",
        ModelInfo {
            model_id: "grok-3-fast",
            display_name: "Grok 3 Fast",
            max_context_length: 131072,
            max_output_length: 32768,
            supports_tools: true,
            supports_multimodal: false,
            supports_web_search: true,
            supports_reasoning: false,
            input_cost_per_million: 5.0,
            output_cost_per_million: 25.0,
            reasoning_cost_per_million: None,
        },
    );

    // ==================== Grok 2 (2024) ====================
    configs.insert(
        "grok-2",
        ModelInfo {
            model_id: "grok-2",
            display_name: "Grok-2",
            max_context_length: 131072, // 128K context
            max_output_length: 32768,
            supports_tools: true,
            supports_multimodal: false,
            supports_web_search: true,
            supports_reasoning: true,
            input_cost_per_million: 2.0,
            output_cost_per_million: 10.0,
            reasoning_cost_per_million: Some(10.0),
        },
    );

    configs.insert(
        "grok-2-mini",
        ModelInfo {
            model_id: "grok-2-mini",
            display_name: "Grok-2 Mini",
            max_context_length: 131072,
            max_output_length: 16384,
            supports_tools: true,
            supports_multimodal: false,
            supports_web_search: true,
            supports_reasoning: false,
            input_cost_per_million: 0.5,
            output_cost_per_million: 2.0,
            reasoning_cost_per_million: None,
        },
    );

    configs.insert(
        "grok-2-1212",
        ModelInfo {
            model_id: "grok-2-1212",
            display_name: "Grok-2 (Dec 2024)",
            max_context_length: 131072,
            max_output_length: 32768,
            supports_tools: true,
            supports_multimodal: false,
            supports_web_search: true,
            supports_reasoning: true,
            input_cost_per_million: 2.0,
            output_cost_per_million: 10.0,
            reasoning_cost_per_million: Some(10.0),
        },
    );

    configs.insert(
        "grok-2-vision-1212",
        ModelInfo {
            model_id: "grok-2-vision-1212",
            display_name: "Grok-2 Vision (Dec 2024)",
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: true,
            supports_web_search: true,
            supports_reasoning: false,
            input_cost_per_million: 2.0,
            output_cost_per_million: 10.0,
            reasoning_cost_per_million: None,
        },
    );

    // ==================== Experimental Models ====================
    configs.insert(
        "grok-beta",
        ModelInfo {
            model_id: "grok-beta",
            display_name: "Grok Beta",
            max_context_length: 131072,
            max_output_length: 32768,
            supports_tools: true,
            supports_multimodal: true,
            supports_web_search: true,
            supports_reasoning: true,
            input_cost_per_million: 5.0,
            output_cost_per_million: 15.0,
            reasoning_cost_per_million: Some(15.0),
        },
    );

    configs.insert(
        "grok-vision-beta",
        ModelInfo {
            model_id: "grok-vision-beta",
            display_name: "Grok Vision Beta",
            max_context_length: 8192,
            max_output_length: 4096,
            supports_tools: true,
            supports_multimodal: true,
            supports_web_search: true,
            supports_reasoning: false,
            input_cost_per_million: 5.0,
            output_cost_per_million: 15.0,
            reasoning_cost_per_million: None,
        },
    );

    configs
});

/// Get model information by ID
pub fn get_model_info(model_id: &str) -> Option<&'static ModelInfo> {
    // Handle xai/ prefix
    let model_id = model_id.strip_prefix("xai/").unwrap_or(model_id);
    MODEL_CONFIGS.get(model_id)
}

/// Get all available model IDs
pub fn get_available_models() -> Vec<&'static str> {
    MODEL_CONFIGS.keys().copied().collect()
}

/// Check if a model supports reasoning tokens
#[cfg(test)]
pub fn supports_reasoning_tokens(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|info| info.supports_reasoning)
        .unwrap_or(false)
}

/// Calculate cost including reasoning tokens
pub fn calculate_cost_with_reasoning(
    model_id: &str,
    input_tokens: u32,
    output_tokens: u32,
    reasoning_tokens: Option<u32>,
) -> Option<f64> {
    let model_info = get_model_info(model_id)?;

    let input_cost = (input_tokens as f64) * (model_info.input_cost_per_million / 1_000_000.0);
    let output_cost = (output_tokens as f64) * (model_info.output_cost_per_million / 1_000_000.0);

    let reasoning_cost = if let (Some(reasoning_tokens), Some(reasoning_rate)) =
        (reasoning_tokens, model_info.reasoning_cost_per_million)
    {
        (reasoning_tokens as f64) * (reasoning_rate / 1_000_000.0)
    } else {
        0.0
    };

    Some(input_cost + output_cost + reasoning_cost)
}

impl XAIModel {
    /// Get the API model ID
    pub fn model_id(&self) -> &'static str {
        match self {
            XAIModel::Grok4 => "grok-4",
            XAIModel::Grok3 => "grok-3",
            XAIModel::Grok3Mini => "grok-3-mini",
            XAIModel::Grok3Fast => "grok-3-fast",
            XAIModel::Grok2 => "grok-2",
            XAIModel::Grok2Mini => "grok-2-mini",
            XAIModel::Grok21212 => "grok-2-1212",
            XAIModel::Grok2Vision => "grok-2-vision-1212",
            XAIModel::GrokBeta => "grok-beta",
            XAIModel::GrokVision => "grok-vision-beta",
        }
    }

    /// Get model information
    pub fn info(&self) -> &'static ModelInfo {
        get_model_info(self.model_id()).expect("Model info should exist for enum variant")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info() {
        // Test Grok-4 model info (latest)
        let info = get_model_info("grok-4").unwrap();
        assert_eq!(info.model_id, "grok-4");
        assert_eq!(info.max_context_length, 256000);
        assert!(info.supports_reasoning);
        assert!(info.supports_multimodal);
        assert!(info.supports_web_search);

        // Test Grok-3 model info
        let info = get_model_info("grok-3").unwrap();
        assert_eq!(info.model_id, "grok-3");
        assert!(info.supports_reasoning);

        // Test Grok-2 model info
        let info = get_model_info("grok-2").unwrap();
        assert_eq!(info.model_id, "grok-2");
        assert_eq!(info.max_context_length, 131072);
        assert!(info.supports_reasoning);
        assert!(info.supports_web_search);
        assert!(info.reasoning_cost_per_million.is_some());

        // Test Grok-2-Mini model info
        let info = get_model_info("grok-2-mini").unwrap();
        assert_eq!(info.model_id, "grok-2-mini");
        assert!(!info.supports_reasoning);
        assert!(info.reasoning_cost_per_million.is_none());

        // Test with xai/ prefix
        let info = get_model_info("xai/grok-2").unwrap();
        assert_eq!(info.model_id, "grok-2");
    }

    #[test]
    fn test_available_models() {
        let models = get_available_models();
        assert!(models.contains(&"grok-4"));
        assert!(models.contains(&"grok-3"));
        assert!(models.contains(&"grok-3-mini"));
        assert!(models.contains(&"grok-2"));
        assert!(models.contains(&"grok-2-mini"));
        assert!(models.contains(&"grok-beta"));
        assert!(models.contains(&"grok-vision-beta"));
    }

    #[test]
    fn test_supports_reasoning() {
        assert!(supports_reasoning_tokens("grok-4"));
        assert!(supports_reasoning_tokens("grok-3"));
        assert!(supports_reasoning_tokens("grok-3-mini"));
        assert!(supports_reasoning_tokens("grok-2"));
        assert!(supports_reasoning_tokens("grok-beta"));
        assert!(!supports_reasoning_tokens("grok-2-mini"));
        assert!(!supports_reasoning_tokens("grok-vision-beta"));
        assert!(!supports_reasoning_tokens("grok-3-fast"));
    }

    #[test]
    fn test_cost_calculation() {
        // Test basic cost calculation
        let cost = calculate_cost_with_reasoning("grok-2", 1000, 500, None).unwrap();
        let expected = (1000.0 * 2.0 / 1_000_000.0) + (500.0 * 10.0 / 1_000_000.0);
        assert!((cost - expected).abs() < 0.0001);

        // Test with reasoning tokens
        let cost = calculate_cost_with_reasoning("grok-2", 1000, 500, Some(200)).unwrap();
        let expected = (1000.0 * 2.0 / 1_000_000.0)
            + (500.0 * 10.0 / 1_000_000.0)
            + (200.0 * 10.0 / 1_000_000.0);
        assert!((cost - expected).abs() < 0.0001);

        // Test model without reasoning support
        let cost = calculate_cost_with_reasoning("grok-2-mini", 1000, 500, Some(200)).unwrap();
        let expected = (1000.0 * 0.5 / 1_000_000.0) + (500.0 * 2.0 / 1_000_000.0); // reasoning ignored
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_xai_model_enum() {
        assert_eq!(XAIModel::Grok4.model_id(), "grok-4");
        assert_eq!(XAIModel::Grok3.model_id(), "grok-3");
        assert_eq!(XAIModel::Grok3Mini.model_id(), "grok-3-mini");
        assert_eq!(XAIModel::Grok2.model_id(), "grok-2");
        assert_eq!(XAIModel::Grok2Mini.model_id(), "grok-2-mini");
        assert_eq!(XAIModel::GrokBeta.model_id(), "grok-beta");
        assert_eq!(XAIModel::GrokVision.model_id(), "grok-vision-beta");

        let info = XAIModel::Grok4.info();
        assert_eq!(info.display_name, "Grok 4");
    }
}
