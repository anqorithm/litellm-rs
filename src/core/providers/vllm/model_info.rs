//! vLLM Model Information
//!
//! Contains model configurations for commonly deployed vLLM models.
//! Since vLLM serves various open-source models, this provides metadata
//! for popular models that are frequently deployed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// vLLM model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VLLMModelInfo {
    /// Model ID as used in API calls
    pub model_id: String,

    /// Human-friendly model name
    pub display_name: String,

    /// Maximum context length (tokens)
    pub max_context_length: u32,

    /// Maximum output tokens
    pub max_output_length: u32,

    /// Whether the model supports tool/function calling
    pub supports_tools: bool,

    /// Whether the model supports vision
    pub supports_multimodal: bool,

    /// Model family (e.g., "llama", "mistral", "qwen")
    pub family: String,
}

impl VLLMModelInfo {
    /// Create a new model info with basic parameters
    pub fn new(
        model_id: impl Into<String>,
        display_name: impl Into<String>,
        max_context_length: u32,
    ) -> Self {
        Self {
            model_id: model_id.into(),
            display_name: display_name.into(),
            max_context_length,
            max_output_length: max_context_length / 2, // Default to half of context
            supports_tools: false,
            supports_multimodal: false,
            family: "unknown".to_string(),
        }
    }

    /// Create a custom model info (for unknown models)
    pub fn custom(model_id: impl Into<String>) -> Self {
        let id = model_id.into();
        Self {
            model_id: id.clone(),
            display_name: id.clone(),
            max_context_length: 4096,   // Conservative default
            max_output_length: 2048,    // Conservative default
            supports_tools: false,      // Unknown capability
            supports_multimodal: false, // Unknown capability
            family: "custom".to_string(),
        }
    }
}

/// Static model configurations for commonly deployed vLLM models
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, VLLMModelInfo>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    // Llama 3.1 models
    configs.insert(
        "meta-llama/Meta-Llama-3.1-8B-Instruct",
        VLLMModelInfo {
            model_id: "meta-llama/Meta-Llama-3.1-8B-Instruct".to_string(),
            display_name: "Llama 3.1 8B Instruct".to_string(),
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "llama".to_string(),
        },
    );

    configs.insert(
        "meta-llama/Meta-Llama-3.1-70B-Instruct",
        VLLMModelInfo {
            model_id: "meta-llama/Meta-Llama-3.1-70B-Instruct".to_string(),
            display_name: "Llama 3.1 70B Instruct".to_string(),
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "llama".to_string(),
        },
    );

    configs.insert(
        "meta-llama/Meta-Llama-3.1-405B-Instruct",
        VLLMModelInfo {
            model_id: "meta-llama/Meta-Llama-3.1-405B-Instruct".to_string(),
            display_name: "Llama 3.1 405B Instruct".to_string(),
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "llama".to_string(),
        },
    );

    // Llama 3.2 models
    configs.insert(
        "meta-llama/Llama-3.2-1B-Instruct",
        VLLMModelInfo {
            model_id: "meta-llama/Llama-3.2-1B-Instruct".to_string(),
            display_name: "Llama 3.2 1B Instruct".to_string(),
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "llama".to_string(),
        },
    );

    configs.insert(
        "meta-llama/Llama-3.2-3B-Instruct",
        VLLMModelInfo {
            model_id: "meta-llama/Llama-3.2-3B-Instruct".to_string(),
            display_name: "Llama 3.2 3B Instruct".to_string(),
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "llama".to_string(),
        },
    );

    // Llama 3.3 models
    configs.insert(
        "meta-llama/Llama-3.3-70B-Instruct",
        VLLMModelInfo {
            model_id: "meta-llama/Llama-3.3-70B-Instruct".to_string(),
            display_name: "Llama 3.3 70B Instruct".to_string(),
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "llama".to_string(),
        },
    );

    // Mistral models
    configs.insert(
        "mistralai/Mistral-7B-Instruct-v0.3",
        VLLMModelInfo {
            model_id: "mistralai/Mistral-7B-Instruct-v0.3".to_string(),
            display_name: "Mistral 7B Instruct v0.3".to_string(),
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "mistral".to_string(),
        },
    );

    configs.insert(
        "mistralai/Mixtral-8x7B-Instruct-v0.1",
        VLLMModelInfo {
            model_id: "mistralai/Mixtral-8x7B-Instruct-v0.1".to_string(),
            display_name: "Mixtral 8x7B Instruct".to_string(),
            max_context_length: 32768,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "mistral".to_string(),
        },
    );

    configs.insert(
        "mistralai/Mixtral-8x22B-Instruct-v0.1",
        VLLMModelInfo {
            model_id: "mistralai/Mixtral-8x22B-Instruct-v0.1".to_string(),
            display_name: "Mixtral 8x22B Instruct".to_string(),
            max_context_length: 65536,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "mistral".to_string(),
        },
    );

    // Qwen models
    configs.insert(
        "Qwen/Qwen2.5-7B-Instruct",
        VLLMModelInfo {
            model_id: "Qwen/Qwen2.5-7B-Instruct".to_string(),
            display_name: "Qwen 2.5 7B Instruct".to_string(),
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "qwen".to_string(),
        },
    );

    configs.insert(
        "Qwen/Qwen2.5-72B-Instruct",
        VLLMModelInfo {
            model_id: "Qwen/Qwen2.5-72B-Instruct".to_string(),
            display_name: "Qwen 2.5 72B Instruct".to_string(),
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "qwen".to_string(),
        },
    );

    // DeepSeek models
    configs.insert(
        "deepseek-ai/DeepSeek-V3",
        VLLMModelInfo {
            model_id: "deepseek-ai/DeepSeek-V3".to_string(),
            display_name: "DeepSeek V3".to_string(),
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "deepseek".to_string(),
        },
    );

    configs.insert(
        "deepseek-ai/DeepSeek-R1",
        VLLMModelInfo {
            model_id: "deepseek-ai/DeepSeek-R1".to_string(),
            display_name: "DeepSeek R1".to_string(),
            max_context_length: 131072,
            max_output_length: 32768,
            supports_tools: true,
            supports_multimodal: false,
            family: "deepseek".to_string(),
        },
    );

    // CodeLlama models
    configs.insert(
        "codellama/CodeLlama-34b-Instruct-hf",
        VLLMModelInfo {
            model_id: "codellama/CodeLlama-34b-Instruct-hf".to_string(),
            display_name: "CodeLlama 34B Instruct".to_string(),
            max_context_length: 16384,
            max_output_length: 4096,
            supports_tools: false,
            supports_multimodal: false,
            family: "codellama".to_string(),
        },
    );

    // Phi models
    configs.insert(
        "microsoft/Phi-3-mini-4k-instruct",
        VLLMModelInfo {
            model_id: "microsoft/Phi-3-mini-4k-instruct".to_string(),
            display_name: "Phi-3 Mini 4K Instruct".to_string(),
            max_context_length: 4096,
            max_output_length: 2048,
            supports_tools: true,
            supports_multimodal: false,
            family: "phi".to_string(),
        },
    );

    configs.insert(
        "microsoft/Phi-3-medium-128k-instruct",
        VLLMModelInfo {
            model_id: "microsoft/Phi-3-medium-128k-instruct".to_string(),
            display_name: "Phi-3 Medium 128K Instruct".to_string(),
            max_context_length: 131072,
            max_output_length: 8192,
            supports_tools: true,
            supports_multimodal: false,
            family: "phi".to_string(),
        },
    );

    // Gemma models
    configs.insert(
        "google/gemma-2-9b-it",
        VLLMModelInfo {
            model_id: "google/gemma-2-9b-it".to_string(),
            display_name: "Gemma 2 9B IT".to_string(),
            max_context_length: 8192,
            max_output_length: 4096,
            supports_tools: false,
            supports_multimodal: false,
            family: "gemma".to_string(),
        },
    );

    configs.insert(
        "google/gemma-2-27b-it",
        VLLMModelInfo {
            model_id: "google/gemma-2-27b-it".to_string(),
            display_name: "Gemma 2 27B IT".to_string(),
            max_context_length: 8192,
            max_output_length: 4096,
            supports_tools: false,
            supports_multimodal: false,
            family: "gemma".to_string(),
        },
    );

    configs
});

/// Get model information for a given model ID
pub fn get_model_info(model_id: &str) -> Option<VLLMModelInfo> {
    MODEL_CONFIGS.get(model_id).cloned()
}

/// Get model info, returning a custom model if not found
pub fn get_or_create_model_info(model_id: &str) -> VLLMModelInfo {
    get_model_info(model_id).unwrap_or_else(|| VLLMModelInfo::custom(model_id))
}

/// Get all known model IDs
#[cfg(test)]
pub fn get_known_models() -> Vec<&'static str> {
    MODEL_CONFIGS.keys().copied().collect()
}

/// Get models by family
#[cfg(test)]
pub fn get_models_by_family(family: &str) -> Vec<&'static str> {
    MODEL_CONFIGS
        .iter()
        .filter(|(_, info)| info.family == family)
        .map(|(id, _)| *id)
        .collect()
}

/// Get models that support tools
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
        let info = get_model_info("meta-llama/Meta-Llama-3.1-8B-Instruct");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.model_id, "meta-llama/Meta-Llama-3.1-8B-Instruct");
        assert_eq!(info.display_name, "Llama 3.1 8B Instruct");
        assert_eq!(info.max_context_length, 131072);
        assert!(info.supports_tools);
        assert_eq!(info.family, "llama");
    }

    #[test]
    fn test_get_model_info_invalid() {
        let info = get_model_info("nonexistent-model");
        assert!(info.is_none());
    }

    #[test]
    fn test_get_or_create_model_info_known() {
        let info = get_or_create_model_info("mistralai/Mistral-7B-Instruct-v0.3");
        assert_eq!(info.model_id, "mistralai/Mistral-7B-Instruct-v0.3");
        assert_eq!(info.family, "mistral");
    }

    #[test]
    fn test_get_or_create_model_info_custom() {
        let info = get_or_create_model_info("my-custom-model");
        assert_eq!(info.model_id, "my-custom-model");
        assert_eq!(info.display_name, "my-custom-model");
        assert_eq!(info.family, "custom");
        assert_eq!(info.max_context_length, 4096); // Default
    }

    #[test]
    fn test_vllm_model_info_new() {
        let info = VLLMModelInfo::new("test-model", "Test Model", 8192);
        assert_eq!(info.model_id, "test-model");
        assert_eq!(info.display_name, "Test Model");
        assert_eq!(info.max_context_length, 8192);
        assert_eq!(info.max_output_length, 4096); // Half of context
    }

    #[test]
    fn test_vllm_model_info_custom() {
        let info = VLLMModelInfo::custom("custom-model");
        assert_eq!(info.model_id, "custom-model");
        assert_eq!(info.family, "custom");
        assert!(!info.supports_tools);
        assert!(!info.supports_multimodal);
    }

    #[test]
    fn test_get_known_models() {
        let models = get_known_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"meta-llama/Meta-Llama-3.1-8B-Instruct"));
        assert!(models.contains(&"mistralai/Mistral-7B-Instruct-v0.3"));
    }

    #[test]
    fn test_get_models_by_family() {
        let llama_models = get_models_by_family("llama");
        assert!(!llama_models.is_empty());
        for model in llama_models {
            assert!(model.contains("llama") || model.contains("Llama"));
        }

        let mistral_models = get_models_by_family("mistral");
        assert!(!mistral_models.is_empty());
    }

    #[test]
    fn test_get_tool_capable_models() {
        let tool_models = get_tool_capable_models();
        assert!(!tool_models.is_empty());

        for model_id in tool_models {
            let info = get_model_info(model_id).unwrap();
            assert!(info.supports_tools);
        }
    }

    #[test]
    fn test_model_info_serialization() {
        let info = get_model_info("Qwen/Qwen2.5-7B-Instruct").unwrap();
        let json = serde_json::to_value(&info).unwrap();

        assert_eq!(json["model_id"], "Qwen/Qwen2.5-7B-Instruct");
        assert_eq!(json["display_name"], "Qwen 2.5 7B Instruct");
        assert_eq!(json["family"], "qwen");
    }

    #[test]
    fn test_deepseek_models() {
        let v3 = get_model_info("deepseek-ai/DeepSeek-V3").unwrap();
        assert_eq!(v3.family, "deepseek");
        assert!(v3.supports_tools);

        let r1 = get_model_info("deepseek-ai/DeepSeek-R1").unwrap();
        assert_eq!(r1.max_output_length, 32768); // R1 has larger output
    }

    #[test]
    fn test_phi_models() {
        let phi3_mini = get_model_info("microsoft/Phi-3-mini-4k-instruct").unwrap();
        assert_eq!(phi3_mini.max_context_length, 4096);

        let phi3_medium = get_model_info("microsoft/Phi-3-medium-128k-instruct").unwrap();
        assert_eq!(phi3_medium.max_context_length, 131072);
    }
}
