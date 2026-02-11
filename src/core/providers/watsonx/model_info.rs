//! Watsonx Model Information
//!
//! Static model information for IBM Watsonx.ai models.
//! Includes pricing, context lengths, and capabilities.

/// Information about a Watsonx model
#[derive(Debug, Clone)]
pub struct WatsonxModel {
    /// Model ID (e.g., "ibm/granite-13b-chat-v2")
    pub model_id: &'static str,
    /// Display name
    pub display_name: &'static str,
    /// Maximum context length in tokens
    pub max_context_length: usize,
    /// Maximum output tokens
    pub max_output_length: usize,
    /// Input cost per million tokens (USD)
    pub input_cost_per_million: f64,
    /// Output cost per million tokens (USD)
    pub output_cost_per_million: f64,
    /// Whether the model supports tool/function calling
    pub supports_tools: bool,
    /// Whether the model supports chat format (vs completion)
    pub supports_chat: bool,
    /// Model provider/family
    pub provider: &'static str,
}

/// Static model information for common Watsonx models
static WATSONX_MODELS: &[WatsonxModel] = &[
    // IBM Granite Models
    WatsonxModel {
        model_id: "ibm/granite-13b-chat-v2",
        display_name: "Granite 13B Chat v2",
        max_context_length: 8192,
        max_output_length: 4096,
        input_cost_per_million: 0.15,
        output_cost_per_million: 0.15,
        supports_tools: true,
        supports_chat: true,
        provider: "ibm",
    },
    WatsonxModel {
        model_id: "ibm/granite-20b-multilingual",
        display_name: "Granite 20B Multilingual",
        max_context_length: 8192,
        max_output_length: 4096,
        input_cost_per_million: 0.20,
        output_cost_per_million: 0.20,
        supports_tools: true,
        supports_chat: true,
        provider: "ibm",
    },
    WatsonxModel {
        model_id: "ibm/granite-3b-code-instruct",
        display_name: "Granite 3B Code Instruct",
        max_context_length: 8192,
        max_output_length: 4096,
        input_cost_per_million: 0.05,
        output_cost_per_million: 0.05,
        supports_tools: false,
        supports_chat: true,
        provider: "ibm",
    },
    WatsonxModel {
        model_id: "ibm/granite-8b-code-instruct",
        display_name: "Granite 8B Code Instruct",
        max_context_length: 8192,
        max_output_length: 4096,
        input_cost_per_million: 0.10,
        output_cost_per_million: 0.10,
        supports_tools: false,
        supports_chat: true,
        provider: "ibm",
    },
    WatsonxModel {
        model_id: "ibm/granite-20b-code-instruct",
        display_name: "Granite 20B Code Instruct",
        max_context_length: 8192,
        max_output_length: 4096,
        input_cost_per_million: 0.20,
        output_cost_per_million: 0.20,
        supports_tools: false,
        supports_chat: true,
        provider: "ibm",
    },
    WatsonxModel {
        model_id: "ibm/granite-34b-code-instruct",
        display_name: "Granite 34B Code Instruct",
        max_context_length: 8192,
        max_output_length: 4096,
        input_cost_per_million: 0.30,
        output_cost_per_million: 0.30,
        supports_tools: false,
        supports_chat: true,
        provider: "ibm",
    },
    // Meta Llama Models on Watsonx
    WatsonxModel {
        model_id: "meta-llama/llama-3-1-70b-instruct",
        display_name: "Llama 3.1 70B Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        input_cost_per_million: 0.90,
        output_cost_per_million: 0.90,
        supports_tools: true,
        supports_chat: true,
        provider: "meta",
    },
    WatsonxModel {
        model_id: "meta-llama/llama-3-1-8b-instruct",
        display_name: "Llama 3.1 8B Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        input_cost_per_million: 0.15,
        output_cost_per_million: 0.15,
        supports_tools: true,
        supports_chat: true,
        provider: "meta",
    },
    WatsonxModel {
        model_id: "meta-llama/llama-3-2-1b-instruct",
        display_name: "Llama 3.2 1B Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        input_cost_per_million: 0.05,
        output_cost_per_million: 0.05,
        supports_tools: true,
        supports_chat: true,
        provider: "meta",
    },
    WatsonxModel {
        model_id: "meta-llama/llama-3-2-3b-instruct",
        display_name: "Llama 3.2 3B Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        input_cost_per_million: 0.08,
        output_cost_per_million: 0.08,
        supports_tools: true,
        supports_chat: true,
        provider: "meta",
    },
    WatsonxModel {
        model_id: "meta-llama/llama-3-2-11b-vision-instruct",
        display_name: "Llama 3.2 11B Vision Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        input_cost_per_million: 0.20,
        output_cost_per_million: 0.20,
        supports_tools: true,
        supports_chat: true,
        provider: "meta",
    },
    WatsonxModel {
        model_id: "meta-llama/llama-3-2-90b-vision-instruct",
        display_name: "Llama 3.2 90B Vision Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        input_cost_per_million: 1.00,
        output_cost_per_million: 1.00,
        supports_tools: true,
        supports_chat: true,
        provider: "meta",
    },
    // Mistral Models on Watsonx
    WatsonxModel {
        model_id: "mistralai/mistral-large",
        display_name: "Mistral Large",
        max_context_length: 128000,
        max_output_length: 4096,
        input_cost_per_million: 3.00,
        output_cost_per_million: 9.00,
        supports_tools: true,
        supports_chat: true,
        provider: "mistral",
    },
    WatsonxModel {
        model_id: "mistralai/mixtral-8x7b-instruct-v01",
        display_name: "Mixtral 8x7B Instruct",
        max_context_length: 32768,
        max_output_length: 4096,
        input_cost_per_million: 0.45,
        output_cost_per_million: 0.45,
        supports_tools: true,
        supports_chat: true,
        provider: "mistral",
    },
    // DeepSeek Models on Watsonx
    WatsonxModel {
        model_id: "deepseek-ai/deepseek-coder-33b-instruct",
        display_name: "DeepSeek Coder 33B Instruct",
        max_context_length: 16384,
        max_output_length: 4096,
        input_cost_per_million: 0.30,
        output_cost_per_million: 0.30,
        supports_tools: false,
        supports_chat: true,
        provider: "deepseek",
    },
    // Allam (Arabic) Models
    WatsonxModel {
        model_id: "sdaia/allam-1-13b-instruct",
        display_name: "Allam 1 13B Instruct",
        max_context_length: 8192,
        max_output_length: 4096,
        input_cost_per_million: 0.20,
        output_cost_per_million: 0.20,
        supports_tools: false,
        supports_chat: true,
        provider: "sdaia",
    },
    // FLAN Models
    WatsonxModel {
        model_id: "google/flan-t5-xxl",
        display_name: "FLAN-T5 XXL",
        max_context_length: 4096,
        max_output_length: 2048,
        input_cost_per_million: 0.10,
        output_cost_per_million: 0.10,
        supports_tools: false,
        supports_chat: false,
        provider: "google",
    },
    WatsonxModel {
        model_id: "google/flan-ul2",
        display_name: "FLAN-UL2",
        max_context_length: 4096,
        max_output_length: 2048,
        input_cost_per_million: 0.20,
        output_cost_per_million: 0.20,
        supports_tools: false,
        supports_chat: false,
        provider: "google",
    },
];

/// Get model information by model ID
pub fn get_model_info(model_id: &str) -> Option<&'static WatsonxModel> {
    WATSONX_MODELS.iter().find(|m| m.model_id == model_id)
}

/// Get all available models
pub fn get_available_models() -> &'static [WatsonxModel] {
    WATSONX_MODELS
}

/// Get models by provider
#[cfg(test)]
pub fn get_models_by_provider(provider: &str) -> Vec<&'static WatsonxModel> {
    WATSONX_MODELS
        .iter()
        .filter(|m| m.provider.eq_ignore_ascii_case(provider))
        .collect()
}

/// Check if a model supports chat format
#[cfg(test)]
pub fn supports_chat(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|m| m.supports_chat)
        .unwrap_or(true)
}

/// Check if a model supports tool calling
pub fn supports_tools(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|m| m.supports_tools)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_info() {
        let model = get_model_info("ibm/granite-13b-chat-v2");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.display_name, "Granite 13B Chat v2");
        assert!(model.supports_chat);
        assert!(model.supports_tools);
    }

    #[test]
    fn test_get_model_info_unknown() {
        let model = get_model_info("unknown-model");
        assert!(model.is_none());
    }

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());

        // Check that we have IBM models
        let ibm_models: Vec<_> = models.iter().filter(|m| m.provider == "ibm").collect();
        assert!(!ibm_models.is_empty());

        // Check that we have Meta models
        let meta_models: Vec<_> = models.iter().filter(|m| m.provider == "meta").collect();
        assert!(!meta_models.is_empty());
    }

    #[test]
    fn test_get_models_by_provider() {
        let ibm_models = get_models_by_provider("ibm");
        assert!(!ibm_models.is_empty());
        for model in ibm_models {
            assert_eq!(model.provider, "ibm");
        }
    }

    #[test]
    fn test_supports_chat() {
        assert!(supports_chat("ibm/granite-13b-chat-v2"));
        assert!(supports_chat("meta-llama/llama-3-1-70b-instruct"));
        // Unknown models default to true
        assert!(supports_chat("unknown-model"));
    }

    #[test]
    fn test_supports_tools() {
        assert!(supports_tools("ibm/granite-13b-chat-v2"));
        assert!(supports_tools("meta-llama/llama-3-1-70b-instruct"));
        assert!(!supports_tools("ibm/granite-3b-code-instruct"));
        // Unknown models default to false
        assert!(!supports_tools("unknown-model"));
    }

    #[test]
    fn test_model_pricing() {
        for model in get_available_models() {
            assert!(model.input_cost_per_million >= 0.0);
            assert!(model.output_cost_per_million >= 0.0);
            assert!(model.max_context_length > 0);
            assert!(model.max_output_length > 0);
        }
    }
}
