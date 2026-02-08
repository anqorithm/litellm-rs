//! OCI Generative AI Model Information
//!
//! Static model information for OCI Generative AI models.
//! Includes Cohere Command and Meta Llama models.

/// Information about an OCI Generative AI model
#[derive(Debug, Clone)]
pub struct OciModel {
    /// Model ID (e.g., "cohere.command-r-plus")
    pub model_id: &'static str,
    /// Display name
    pub display_name: &'static str,
    /// Maximum context length in tokens
    pub max_context_length: u32,
    /// Maximum output tokens
    pub max_output_length: u32,
    /// Input cost per million tokens (USD)
    pub input_cost_per_million: f64,
    /// Output cost per million tokens (USD)
    pub output_cost_per_million: f64,
    /// Whether the model supports tool/function calling
    pub supports_tools: bool,
    /// Whether the model supports vision
    pub supports_multimodal: bool,
    /// Model provider/family
    pub provider: &'static str,
}

/// Static model information for OCI Generative AI models
static OCI_MODELS: &[OciModel] = &[
    // Cohere Command R+ Model
    OciModel {
        model_id: "cohere.command-r-plus",
        display_name: "Cohere Command R+",
        max_context_length: 128_000,
        max_output_length: 4_096,
        input_cost_per_million: 3.0,
        output_cost_per_million: 15.0,
        supports_tools: true,
        supports_multimodal: false,
        provider: "cohere",
    },
    // Cohere Command R Model
    OciModel {
        model_id: "cohere.command-r-16k",
        display_name: "Cohere Command R",
        max_context_length: 16_000,
        max_output_length: 4_096,
        input_cost_per_million: 0.5,
        output_cost_per_million: 1.5,
        supports_tools: true,
        supports_multimodal: false,
        provider: "cohere",
    },
    // Cohere Command Model
    OciModel {
        model_id: "cohere.command",
        display_name: "Cohere Command",
        max_context_length: 4_096,
        max_output_length: 4_096,
        input_cost_per_million: 1.0,
        output_cost_per_million: 2.0,
        supports_tools: true,
        supports_multimodal: false,
        provider: "cohere",
    },
    // Cohere Command Light Model
    OciModel {
        model_id: "cohere.command-light",
        display_name: "Cohere Command Light",
        max_context_length: 4_096,
        max_output_length: 4_096,
        input_cost_per_million: 0.3,
        output_cost_per_million: 0.6,
        supports_tools: false,
        supports_multimodal: false,
        provider: "cohere",
    },
    // Meta Llama 3.1 405B
    OciModel {
        model_id: "meta.llama-3.1-405b-instruct",
        display_name: "Llama 3.1 405B Instruct",
        max_context_length: 128_000,
        max_output_length: 4_096,
        input_cost_per_million: 5.0,
        output_cost_per_million: 16.0,
        supports_tools: true,
        supports_multimodal: false,
        provider: "meta",
    },
    // Meta Llama 3.1 70B
    OciModel {
        model_id: "meta.llama-3.1-70b-instruct",
        display_name: "Llama 3.1 70B Instruct",
        max_context_length: 128_000,
        max_output_length: 4_096,
        input_cost_per_million: 0.9,
        output_cost_per_million: 0.9,
        supports_tools: true,
        supports_multimodal: false,
        provider: "meta",
    },
    // Meta Llama 3 70B
    OciModel {
        model_id: "meta.llama-3-70b-instruct",
        display_name: "Llama 3 70B Instruct",
        max_context_length: 8_192,
        max_output_length: 4_096,
        input_cost_per_million: 0.9,
        output_cost_per_million: 0.9,
        supports_tools: true,
        supports_multimodal: false,
        provider: "meta",
    },
    // Meta Llama 2 70B
    OciModel {
        model_id: "meta.llama-2-70b-chat",
        display_name: "Llama 2 70B Chat",
        max_context_length: 4_096,
        max_output_length: 4_096,
        input_cost_per_million: 0.9,
        output_cost_per_million: 0.9,
        supports_tools: false,
        supports_multimodal: false,
        provider: "meta",
    },
];

/// Get model information by model ID
pub fn get_model_info(model_id: &str) -> Option<&'static OciModel> {
    OCI_MODELS.iter().find(|m| m.model_id == model_id)
}

/// Get all available models
pub fn get_available_models() -> &'static [OciModel] {
    OCI_MODELS
}

/// Get models by provider
#[cfg(test)]
pub fn get_models_by_provider(provider: &str) -> Vec<&'static OciModel> {
    OCI_MODELS
        .iter()
        .filter(|m| m.provider.eq_ignore_ascii_case(provider))
        .collect()
}

/// Check if a model supports tool calling
pub fn supports_tools(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|m| m.supports_tools)
        .unwrap_or(false)
}

/// Check if a model supports vision
#[cfg(test)]
pub fn supports_vision(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|m| m.supports_multimodal)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_info() {
        let model = get_model_info("cohere.command-r-plus");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.display_name, "Cohere Command R+");
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
        assert!(models.len() >= 5);
    }

    #[test]
    fn test_get_models_by_provider() {
        let cohere_models = get_models_by_provider("cohere");
        assert!(!cohere_models.is_empty());
        for model in cohere_models {
            assert_eq!(model.provider, "cohere");
        }

        let meta_models = get_models_by_provider("meta");
        assert!(!meta_models.is_empty());
    }

    #[test]
    fn test_supports_tools() {
        assert!(supports_tools("cohere.command-r-plus"));
        assert!(supports_tools("meta.llama-3.1-70b-instruct"));
        assert!(!supports_tools("cohere.command-light"));
        assert!(!supports_tools("unknown-model"));
    }

    #[test]
    fn test_supports_vision() {
        // Currently no OCI models support vision
        assert!(!supports_vision("cohere.command-r-plus"));
        assert!(!supports_vision("unknown-model"));
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
