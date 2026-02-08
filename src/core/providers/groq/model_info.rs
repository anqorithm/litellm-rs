//! Groq Model Information
//!
//! Contains model configurations and capabilities for Groq-supported models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Groq model identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GroqModel {
    // Llama 3.3 models
    Llama33_70B,

    // Llama 3.2 models
    Llama32_90BTextPreview,
    Llama32_11BTextPreview,
    Llama32_3BPreview,
    Llama32_1BPreview,

    // Llama 3.1 models
    Llama31_405B,
    Llama31_70B,
    Llama31_8B,

    // Llama 3 models
    Llama3_70B,
    Llama3_8B,

    // Mixtral models
    Mixtral8x7B,

    // Gemma models
    Gemma2_9B,
    Gemma7B,

    // Distilled models
    Llama3GroqToolUse70B,
    Llama3GroqToolUse8B,

    // Audio models
    WhisperLargeV3,
    WhisperLargeV3Turbo,
    DistilWhisperLargeV3,
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

    /// Whether this is a reasoning model
    pub is_reasoning: bool,

    /// Whether the model supports vision
    pub supports_multimodal: bool,

    /// Whether this is an audio model
    pub is_audio: bool,

    /// Cost per 1M input tokens (USD)
    pub input_cost_per_million: f64,

    /// Cost per 1M output tokens (USD)
    pub output_cost_per_million: f64,
}

/// Static model configurations
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelInfo>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    // Llama 3.3 models
    configs.insert(
        "llama-3.3-70b-versatile",
        ModelInfo {
            model_id: "llama-3.3-70b-versatile",
            display_name: "Llama 3.3 70B",
            max_context_length: 128000,
            max_output_length: 32768,
            supports_tools: true,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.59,
            output_cost_per_million: 0.79,
        },
    );

    // Llama 3.2 models
    configs.insert(
        "llama-3.2-90b-text-preview",
        ModelInfo {
            model_id: "llama-3.2-90b-text-preview",
            display_name: "Llama 3.2 90B Text Preview",
            max_context_length: 128000,
            max_output_length: 8192,
            supports_tools: false,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.90,
            output_cost_per_million: 0.90,
        },
    );

    configs.insert(
        "llama-3.2-11b-text-preview",
        ModelInfo {
            model_id: "llama-3.2-11b-text-preview",
            display_name: "Llama 3.2 11B Text Preview",
            max_context_length: 128000,
            max_output_length: 8192,
            supports_tools: false,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.18,
            output_cost_per_million: 0.18,
        },
    );

    // Llama 3.1 models
    configs.insert(
        "llama-3.1-405b-reasoning",
        ModelInfo {
            model_id: "llama-3.1-405b-reasoning",
            display_name: "Llama 3.1 405B Reasoning",
            max_context_length: 131072,
            max_output_length: 16384,
            supports_tools: true,
            is_reasoning: true,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 3.00,
            output_cost_per_million: 3.00,
        },
    );

    configs.insert(
        "llama-3.1-70b-versatile",
        ModelInfo {
            model_id: "llama-3.1-70b-versatile",
            display_name: "Llama 3.1 70B",
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.59,
            output_cost_per_million: 0.79,
        },
    );

    configs.insert(
        "llama-3.1-8b-instant",
        ModelInfo {
            model_id: "llama-3.1-8b-instant",
            display_name: "Llama 3.1 8B",
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.05,
            output_cost_per_million: 0.08,
        },
    );

    // Mixtral
    configs.insert(
        "mixtral-8x7b-32768",
        ModelInfo {
            model_id: "mixtral-8x7b-32768",
            display_name: "Mixtral 8x7B",
            max_context_length: 32768,
            max_output_length: 32768,
            supports_tools: true,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.24,
            output_cost_per_million: 0.24,
        },
    );

    // Gemma models
    configs.insert(
        "gemma2-9b-it",
        ModelInfo {
            model_id: "gemma2-9b-it",
            display_name: "Gemma2 9B",
            max_context_length: 8192,
            max_output_length: 8192,
            supports_tools: true,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.20,
            output_cost_per_million: 0.20,
        },
    );

    // Tool use optimized models
    configs.insert(
        "llama3-groq-70b-8192-tool-use-preview",
        ModelInfo {
            model_id: "llama3-groq-70b-8192-tool-use-preview",
            display_name: "Llama 3 Groq 70B Tool Use",
            max_context_length: 8192,
            max_output_length: 8192,
            supports_tools: true,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.89,
            output_cost_per_million: 0.89,
        },
    );

    configs.insert(
        "llama3-groq-8b-8192-tool-use-preview",
        ModelInfo {
            model_id: "llama3-groq-8b-8192-tool-use-preview",
            display_name: "Llama 3 Groq 8B Tool Use",
            max_context_length: 8192,
            max_output_length: 8192,
            supports_tools: true,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.19,
            output_cost_per_million: 0.19,
        },
    );

    // Audio models
    configs.insert(
        "whisper-large-v3",
        ModelInfo {
            model_id: "whisper-large-v3",
            display_name: "Whisper Large v3",
            max_context_length: 0, // Audio model
            max_output_length: 0,  // Audio model
            supports_tools: false,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: true,
            input_cost_per_million: 0.111, // Per hour of audio
            output_cost_per_million: 0.0,
        },
    );

    configs.insert(
        "whisper-large-v3-turbo",
        ModelInfo {
            model_id: "whisper-large-v3-turbo",
            display_name: "Whisper Large v3 Turbo",
            max_context_length: 0,
            max_output_length: 0,
            supports_tools: false,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: true,
            input_cost_per_million: 0.04, // Per hour of audio
            output_cost_per_million: 0.0,
        },
    );

    configs.insert(
        "distil-whisper-large-v3-en",
        ModelInfo {
            model_id: "distil-whisper-large-v3-en",
            display_name: "Distil Whisper Large v3",
            max_context_length: 0,
            max_output_length: 0,
            supports_tools: false,
            is_reasoning: false,
            supports_multimodal: false,
            is_audio: true,
            input_cost_per_million: 0.02, // Per hour of audio
            output_cost_per_million: 0.0,
        },
    );

    // Reasoning models (matching Python LiteLLM configuration)
    configs.insert(
        "deepseek-r1-distill-llama-70b",
        ModelInfo {
            model_id: "deepseek-r1-distill-llama-70b",
            display_name: "DeepSeek R1 Distill Llama 70B",
            max_context_length: 131072,
            max_output_length: 131072,
            supports_tools: true,
            is_reasoning: true,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.59,
            output_cost_per_million: 0.79,
        },
    );

    configs.insert(
        "qwen3-32b",
        ModelInfo {
            model_id: "qwen3-32b",
            display_name: "Qwen 3 32B",
            max_context_length: 131072,
            max_output_length: 131072,
            supports_tools: true,
            is_reasoning: true,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.59,
            output_cost_per_million: 0.79,
        },
    );

    configs.insert(
        "gpt-oss-20b",
        ModelInfo {
            model_id: "gpt-oss-20b",
            display_name: "GPT OSS 20B",
            max_context_length: 131072,
            max_output_length: 32766,
            supports_tools: true,
            is_reasoning: true,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.15,
            output_cost_per_million: 0.75,
        },
    );

    configs.insert(
        "gpt-oss-120b",
        ModelInfo {
            model_id: "gpt-oss-120b",
            display_name: "GPT OSS 120B",
            max_context_length: 131072,
            max_output_length: 32766,
            supports_tools: true,
            is_reasoning: true,
            supports_multimodal: false,
            is_audio: false,
            input_cost_per_million: 0.15,
            output_cost_per_million: 0.75,
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
        let info = get_model_info("llama-3.3-70b-versatile");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.model_id, "llama-3.3-70b-versatile");
        assert_eq!(info.display_name, "Llama 3.3 70B");
        assert_eq!(info.max_context_length, 128000);
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
        assert!(is_reasoning_model("llama-3.1-405b-reasoning"));
        assert!(is_reasoning_model("deepseek-r1-distill-llama-70b"));
        assert!(is_reasoning_model("qwen3-32b"));
        assert!(!is_reasoning_model("llama-3.3-70b-versatile"));
        assert!(!is_reasoning_model("mixtral-8x7b-32768"));
        assert!(!is_reasoning_model("nonexistent-model"));
    }

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"llama-3.3-70b-versatile"));
        assert!(models.contains(&"mixtral-8x7b-32768"));
        assert!(models.contains(&"whisper-large-v3"));
    }

    #[test]
    fn test_get_tool_capable_models() {
        let models = get_tool_capable_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"llama-3.3-70b-versatile"));
        assert!(models.contains(&"mixtral-8x7b-32768"));
        // Whisper models don't support tools
        assert!(!models.contains(&"whisper-large-v3"));
    }

    #[test]
    fn test_audio_models() {
        let whisper = get_model_info("whisper-large-v3").unwrap();
        assert!(whisper.is_audio);
        assert!(!whisper.supports_tools);
        assert_eq!(whisper.max_context_length, 0);

        let whisper_turbo = get_model_info("whisper-large-v3-turbo").unwrap();
        assert!(whisper_turbo.is_audio);

        let distil_whisper = get_model_info("distil-whisper-large-v3-en").unwrap();
        assert!(distil_whisper.is_audio);
    }

    #[test]
    fn test_model_info_costs() {
        let info = get_model_info("llama-3.1-8b-instant").unwrap();
        assert!(info.input_cost_per_million > 0.0);
        assert!(info.output_cost_per_million > 0.0);
        assert!(info.input_cost_per_million < info.output_cost_per_million);
    }

    #[test]
    fn test_groq_model_enum() {
        let model = GroqModel::Llama33_70B;
        assert_eq!(format!("{:?}", model), "Llama33_70B");

        let model = GroqModel::WhisperLargeV3;
        assert_eq!(format!("{:?}", model), "WhisperLargeV3");
    }

    #[test]
    fn test_model_info_serialization() {
        let info = get_model_info("mixtral-8x7b-32768").unwrap();
        assert_eq!(info.model_id, "mixtral-8x7b-32768");
        assert_eq!(info.max_context_length, 32768);
        assert_eq!(info.max_output_length, 32768);
    }

    #[test]
    fn test_tool_use_models() {
        let tool_70b = get_model_info("llama3-groq-70b-8192-tool-use-preview").unwrap();
        assert!(tool_70b.supports_tools);
        assert_eq!(tool_70b.max_context_length, 8192);

        let tool_8b = get_model_info("llama3-groq-8b-8192-tool-use-preview").unwrap();
        assert!(tool_8b.supports_tools);
    }

    #[test]
    fn test_gemma_models() {
        let gemma = get_model_info("gemma2-9b-it").unwrap();
        assert_eq!(gemma.display_name, "Gemma2 9B");
        assert!(gemma.supports_tools);
        assert!(!gemma.is_audio);
    }
}
