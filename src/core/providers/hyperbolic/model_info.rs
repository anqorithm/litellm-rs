//! Hyperbolic Model Information
//!
//! Contains model configurations and capabilities for Hyperbolic-supported models.
//! Hyperbolic provides access to various open-source and proprietary models.

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
        "meta-llama/Meta-Llama-3.1-8B-Instruct",
        ModelInfo {
            model_id: "meta-llama/Meta-Llama-3.1-8B-Instruct",
            display_name: "Llama 3.1 8B Instruct",
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.10,
            output_cost_per_million: 0.10,
        },
    );

    configs.insert(
        "meta-llama/Meta-Llama-3.1-70B-Instruct",
        ModelInfo {
            model_id: "meta-llama/Meta-Llama-3.1-70B-Instruct",
            display_name: "Llama 3.1 70B Instruct",
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.40,
            output_cost_per_million: 0.40,
        },
    );

    configs.insert(
        "meta-llama/Meta-Llama-3.1-405B-Instruct",
        ModelInfo {
            model_id: "meta-llama/Meta-Llama-3.1-405B-Instruct",
            display_name: "Llama 3.1 405B Instruct",
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 2.00,
            output_cost_per_million: 2.00,
        },
    );

    configs.insert(
        "meta-llama/Llama-3.2-3B-Instruct",
        ModelInfo {
            model_id: "meta-llama/Llama-3.2-3B-Instruct",
            display_name: "Llama 3.2 3B Instruct",
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_million: 0.06,
            output_cost_per_million: 0.06,
        },
    );

    // Qwen models
    configs.insert(
        "Qwen/Qwen2.5-72B-Instruct",
        ModelInfo {
            model_id: "Qwen/Qwen2.5-72B-Instruct",
            display_name: "Qwen 2.5 72B Instruct",
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.40,
            output_cost_per_million: 0.40,
        },
    );

    configs.insert(
        "Qwen/Qwen2.5-Coder-32B-Instruct",
        ModelInfo {
            model_id: "Qwen/Qwen2.5-Coder-32B-Instruct",
            display_name: "Qwen 2.5 Coder 32B Instruct",
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.20,
            output_cost_per_million: 0.20,
        },
    );

    // DeepSeek models
    configs.insert(
        "deepseek-ai/DeepSeek-V2.5",
        ModelInfo {
            model_id: "deepseek-ai/DeepSeek-V2.5",
            display_name: "DeepSeek V2.5",
            max_context_length: 65536,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.14,
            output_cost_per_million: 0.28,
        },
    );

    configs.insert(
        "deepseek-ai/DeepSeek-R1",
        ModelInfo {
            model_id: "deepseek-ai/DeepSeek-R1",
            display_name: "DeepSeek R1",
            max_context_length: 65536,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.55,
            output_cost_per_million: 2.19,
        },
    );

    // Mistral models
    configs.insert(
        "mistralai/Mistral-7B-Instruct-v0.3",
        ModelInfo {
            model_id: "mistralai/Mistral-7B-Instruct-v0.3",
            display_name: "Mistral 7B Instruct v0.3",
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_million: 0.10,
            output_cost_per_million: 0.10,
        },
    );

    // Hermes models
    configs.insert(
        "NousResearch/Hermes-3-Llama-3.1-70B",
        ModelInfo {
            model_id: "NousResearch/Hermes-3-Llama-3.1-70B",
            display_name: "Hermes 3 Llama 3.1 70B",
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_million: 0.40,
            output_cost_per_million: 0.40,
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
        let info = get_model_info("meta-llama/Meta-Llama-3.1-70B-Instruct");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.model_id, "meta-llama/Meta-Llama-3.1-70B-Instruct");
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
        assert!(models.contains(&"meta-llama/Meta-Llama-3.1-70B-Instruct"));
        assert!(models.contains(&"Qwen/Qwen2.5-72B-Instruct"));
    }

    #[test]
    fn test_get_tool_capable_models() {
        let models = get_tool_capable_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"meta-llama/Meta-Llama-3.1-70B-Instruct"));
        // Mistral 7B v0.3 doesn't support tools
        assert!(!models.contains(&"mistralai/Mistral-7B-Instruct-v0.3"));
    }

    #[test]
    fn test_model_info_costs() {
        let info = get_model_info("meta-llama/Meta-Llama-3.1-8B-Instruct").unwrap();
        assert!(info.input_cost_per_million > 0.0);
        assert!(info.output_cost_per_million > 0.0);
    }

    #[test]
    fn test_deepseek_model() {
        let info = get_model_info("deepseek-ai/DeepSeek-V2.5").unwrap();
        assert_eq!(info.max_context_length, 65536);
        assert!(info.supports_tools);
    }

    #[test]
    fn test_qwen_coder_model() {
        let info = get_model_info("Qwen/Qwen2.5-Coder-32B-Instruct").unwrap();
        assert!(info.supports_tools);
        assert_eq!(info.max_context_length, 32768);
    }
}
