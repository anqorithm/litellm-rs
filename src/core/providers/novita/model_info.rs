//! Novita Model Information
//!
//! Contains model configurations and capabilities for Novita-supported models.
//! Novita provides access to various open-source and proprietary models.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model ID as used in API calls
    pub model_id: &'static str,

    /// Human-friendly model name
    pub display_name: &'static str,

    /// Maximum context length (tokens)
    pub max_context_length: u32,

    /// Maximum output tokens
    pub max_output_length: u32,

    /// Whether the model supports tool/function calling
    pub supports_tools: bool,

    /// Whether the model supports vision
    pub supports_multimodal: bool,

    /// Cost per 1M input tokens (USD)
    pub input_cost_per_million: f64,

    /// Cost per 1M output tokens (USD)
    pub output_cost_per_million: f64,
}

/// Static model configurations
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelInfo>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    // Meta Llama models
    configs.insert(
        "meta-llama/llama-3.1-8b-instruct",
        ModelInfo {
            model_id: "meta-llama/llama-3.1-8b-instruct",
            display_name: "Llama 3.1 8B Instruct",
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.05,
            output_cost_per_million: 0.08,
        },
    );

    configs.insert(
        "meta-llama/llama-3.1-70b-instruct",
        ModelInfo {
            model_id: "meta-llama/llama-3.1-70b-instruct",
            display_name: "Llama 3.1 70B Instruct",
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.59,
            output_cost_per_million: 0.79,
        },
    );

    configs.insert(
        "meta-llama/llama-3.1-405b-instruct",
        ModelInfo {
            model_id: "meta-llama/llama-3.1-405b-instruct",
            display_name: "Llama 3.1 405B Instruct",
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 3.00,
            output_cost_per_million: 3.00,
        },
    );

    // Mistral models
    configs.insert(
        "mistralai/mistral-7b-instruct",
        ModelInfo {
            model_id: "mistralai/mistral-7b-instruct",
            display_name: "Mistral 7B Instruct",
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_million: 0.05,
            output_cost_per_million: 0.05,
        },
    );

    configs.insert(
        "mistralai/mixtral-8x7b-instruct",
        ModelInfo {
            model_id: "mistralai/mixtral-8x7b-instruct",
            display_name: "Mixtral 8x7B Instruct",
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.24,
            output_cost_per_million: 0.24,
        },
    );

    // Qwen models
    configs.insert(
        "qwen/qwen-2-7b-instruct",
        ModelInfo {
            model_id: "qwen/qwen-2-7b-instruct",
            display_name: "Qwen 2 7B Instruct",
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_million: 0.07,
            output_cost_per_million: 0.07,
        },
    );

    configs.insert(
        "qwen/qwen-2-72b-instruct",
        ModelInfo {
            model_id: "qwen/qwen-2-72b-instruct",
            display_name: "Qwen 2 72B Instruct",
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.59,
            output_cost_per_million: 0.79,
        },
    );

    // DeepSeek models
    configs.insert(
        "deepseek/deepseek-v2.5",
        ModelInfo {
            model_id: "deepseek/deepseek-v2.5",
            display_name: "DeepSeek V2.5",
            max_context_length: 65536,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.14,
            output_cost_per_million: 0.28,
        },
    );

    // Yi models
    configs.insert(
        "01-ai/yi-1.5-34b-chat",
        ModelInfo {
            model_id: "01-ai/yi-1.5-34b-chat",
            display_name: "Yi 1.5 34B Chat",
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_million: 0.18,
            output_cost_per_million: 0.18,
        },
    );

    configs
});

/// Get model information for a given model ID
pub fn get_model_info(model_id: &str) -> Option<&'static ModelInfo> {
    MODEL_CONFIGS.get(model_id)
}

/// Get all available model IDs
pub fn get_available_models() -> Vec<&'static str> {
    MODEL_CONFIGS.keys().copied().collect()
}

/// Get all models that support tool/function calling
#[cfg(test)]
pub fn get_tool_capable_models() -> Vec<&'static str> {
    MODEL_CONFIGS
        .iter()
        .filter(|(_, info)| info.supports_tools)
        .map(|(id, _)| *id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_info_valid() {
        let info = get_model_info("meta-llama/llama-3.1-70b-instruct");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.model_id, "meta-llama/llama-3.1-70b-instruct");
        assert_eq!(info.display_name, "Llama 3.1 70B Instruct");
        assert_eq!(info.max_context_length, 131072);
        assert!(info.supports_tools);
    }

    #[test]
    fn test_get_model_info_invalid() {
        let info = get_model_info("nonexistent-model");
        assert!(info.is_none());
    }

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"meta-llama/llama-3.1-70b-instruct"));
        assert!(models.contains(&"mistralai/mixtral-8x7b-instruct"));
    }

    #[test]
    fn test_get_tool_capable_models() {
        let models = get_tool_capable_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"meta-llama/llama-3.1-70b-instruct"));
        // Mistral 7B doesn't support tools
        assert!(!models.contains(&"mistralai/mistral-7b-instruct"));
    }

    #[test]
    fn test_model_info_costs() {
        let info = get_model_info("meta-llama/llama-3.1-8b-instruct").unwrap();
        assert!(info.input_cost_per_million > 0.0);
        assert!(info.output_cost_per_million > 0.0);
    }

    #[test]
    fn test_mixtral_model() {
        let info = get_model_info("mistralai/mixtral-8x7b-instruct").unwrap();
        assert_eq!(info.max_context_length, 32768);
        assert!(info.supports_tools);
    }

    #[test]
    fn test_deepseek_model() {
        let info = get_model_info("deepseek/deepseek-v2.5").unwrap();
        assert_eq!(info.max_context_length, 65536);
        assert!(info.supports_tools);
    }
}
