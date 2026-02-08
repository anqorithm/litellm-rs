//! Lambda Labs AI Model Information
//!
//! Contains model configurations and capabilities for Lambda Labs-supported models.
//! Lambda Labs provides GPU-accelerated inference for popular open-source models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Lambda Labs model identifier
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LambdaModel {
    // Hermes 3 models (based on Llama 3.1)
    Hermes3Llama31_405B_FP8,
    Hermes3Llama31_405B_FP8_128K,

    // Llama 3.3 models
    Llama33_70B_Instruct_FP8,

    // Llama 3.1 models
    Llama31_405B_Instruct_FP8,
    Llama31_70B_Instruct_FP8,
    Llama31_8B_Instruct,

    // Qwen models
    Qwen25_72B_Instruct,
    Qwen25Coder_32B_Instruct,

    // DeepSeek models
    DeepSeekR1,
    DeepSeekR1_671B,
}

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

    /// Whether the model supports vision/multimodal input
    pub supports_multimodal: bool,

    /// Whether this is a reasoning model
    pub is_reasoning: bool,

    /// Cost per 1M input tokens (USD)
    pub input_cost_per_million: f64,

    /// Cost per 1M output tokens (USD)
    pub output_cost_per_million: f64,
}

/// Static model configurations
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelInfo>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    // Hermes 3 models (Nous Research fine-tuned Llama 3.1)
    configs.insert(
        "hermes-3-llama-3.1-405b-fp8",
        ModelInfo {
            model_id: "hermes-3-llama-3.1-405b-fp8",
            display_name: "Hermes 3 Llama 3.1 405B FP8",
            max_context_length: 32768,
            max_output_length: 4096,
            supports_tools: true,
            supports_multimodal: false,
            is_reasoning: false,
            input_cost_per_million: 0.80,
            output_cost_per_million: 0.80,
        },
    );

    configs.insert(
        "hermes-3-llama-3.1-405b-fp8-128k",
        ModelInfo {
            model_id: "hermes-3-llama-3.1-405b-fp8-128k",
            display_name: "Hermes 3 Llama 3.1 405B FP8 128K",
            max_context_length: 128000,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            is_reasoning: false,
            input_cost_per_million: 0.90,
            output_cost_per_million: 0.90,
        },
    );

    // Llama 3.3 models
    configs.insert(
        "llama-3.3-70b-instruct-fp8",
        ModelInfo {
            model_id: "llama-3.3-70b-instruct-fp8",
            display_name: "Llama 3.3 70B Instruct FP8",
            max_context_length: 128000,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            is_reasoning: false,
            input_cost_per_million: 0.20,
            output_cost_per_million: 0.20,
        },
    );

    // Llama 3.1 models
    configs.insert(
        "llama-3.1-405b-instruct-fp8",
        ModelInfo {
            model_id: "llama-3.1-405b-instruct-fp8",
            display_name: "Llama 3.1 405B Instruct FP8",
            max_context_length: 128000,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            is_reasoning: false,
            input_cost_per_million: 0.80,
            output_cost_per_million: 0.80,
        },
    );

    configs.insert(
        "llama-3.1-70b-instruct-fp8",
        ModelInfo {
            model_id: "llama-3.1-70b-instruct-fp8",
            display_name: "Llama 3.1 70B Instruct FP8",
            max_context_length: 128000,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            is_reasoning: false,
            input_cost_per_million: 0.20,
            output_cost_per_million: 0.20,
        },
    );

    configs.insert(
        "llama-3.1-8b-instruct",
        ModelInfo {
            model_id: "llama-3.1-8b-instruct",
            display_name: "Llama 3.1 8B Instruct",
            max_context_length: 128000,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            is_reasoning: false,
            input_cost_per_million: 0.10,
            output_cost_per_million: 0.10,
        },
    );

    // Qwen models
    configs.insert(
        "qwen2.5-72b-instruct",
        ModelInfo {
            model_id: "qwen2.5-72b-instruct",
            display_name: "Qwen 2.5 72B Instruct",
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            is_reasoning: false,
            input_cost_per_million: 0.20,
            output_cost_per_million: 0.20,
        },
    );

    configs.insert(
        "qwen2.5-coder-32b-instruct",
        ModelInfo {
            model_id: "qwen2.5-coder-32b-instruct",
            display_name: "Qwen 2.5 Coder 32B Instruct",
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            is_reasoning: false,
            input_cost_per_million: 0.10,
            output_cost_per_million: 0.10,
        },
    );

    // DeepSeek reasoning models
    configs.insert(
        "deepseek-r1",
        ModelInfo {
            model_id: "deepseek-r1",
            display_name: "DeepSeek R1",
            max_context_length: 64000,
            max_output_length: 16384,
            supports_tools: true,
            supports_multimodal: false,
            is_reasoning: true,
            input_cost_per_million: 0.55,
            output_cost_per_million: 2.19,
        },
    );

    configs.insert(
        "deepseek-r1-671b",
        ModelInfo {
            model_id: "deepseek-r1-671b",
            display_name: "DeepSeek R1 671B",
            max_context_length: 64000,
            max_output_length: 16384,
            supports_tools: true,
            supports_multimodal: false,
            is_reasoning: true,
            input_cost_per_million: 0.55,
            output_cost_per_million: 2.19,
        },
    );

    configs
});

/// Get model information for a given model ID
pub fn get_model_info(model_id: &str) -> Option<&'static ModelInfo> {
    MODEL_CONFIGS.get(model_id)
}

/// Check if a model supports reasoning
pub fn is_reasoning_model(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|info| info.is_reasoning)
        .unwrap_or(false)
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
        let info = get_model_info("hermes-3-llama-3.1-405b-fp8");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.model_id, "hermes-3-llama-3.1-405b-fp8");
        assert_eq!(info.display_name, "Hermes 3 Llama 3.1 405B FP8");
        assert_eq!(info.max_context_length, 32768);
        assert!(info.supports_tools);
        assert!(!info.is_reasoning);
    }

    #[test]
    fn test_get_model_info_invalid() {
        let info = get_model_info("nonexistent-model");
        assert!(info.is_none());
    }

    #[test]
    fn test_is_reasoning_model() {
        assert!(is_reasoning_model("deepseek-r1"));
        assert!(is_reasoning_model("deepseek-r1-671b"));
        assert!(!is_reasoning_model("hermes-3-llama-3.1-405b-fp8"));
        assert!(!is_reasoning_model("llama-3.3-70b-instruct-fp8"));
        assert!(!is_reasoning_model("nonexistent-model"));
    }

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"hermes-3-llama-3.1-405b-fp8"));
        assert!(models.contains(&"llama-3.3-70b-instruct-fp8"));
        assert!(models.contains(&"deepseek-r1"));
    }

    #[test]
    fn test_get_tool_capable_models() {
        let models = get_tool_capable_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"hermes-3-llama-3.1-405b-fp8"));
        assert!(models.contains(&"llama-3.1-8b-instruct"));
    }

    #[test]
    fn test_model_info_costs() {
        let info = get_model_info("llama-3.1-8b-instruct").unwrap();
        assert!(info.input_cost_per_million > 0.0);
        assert!(info.output_cost_per_million > 0.0);
    }

    #[test]
    fn test_lambda_model_enum() {
        let model = LambdaModel::Hermes3Llama31_405B_FP8;
        assert_eq!(format!("{:?}", model), "Hermes3Llama31_405B_FP8");

        let model = LambdaModel::DeepSeekR1;
        assert_eq!(format!("{:?}", model), "DeepSeekR1");
    }

    #[test]
    fn test_128k_context_model() {
        let info = get_model_info("hermes-3-llama-3.1-405b-fp8-128k").unwrap();
        assert_eq!(info.max_context_length, 128000);
        assert_eq!(info.max_output_length, 8192);
    }

    #[test]
    fn test_qwen_models() {
        let qwen = get_model_info("qwen2.5-72b-instruct").unwrap();
        assert_eq!(qwen.display_name, "Qwen 2.5 72B Instruct");
        assert!(qwen.supports_tools);

        let coder = get_model_info("qwen2.5-coder-32b-instruct").unwrap();
        assert_eq!(coder.display_name, "Qwen 2.5 Coder 32B Instruct");
    }
}
