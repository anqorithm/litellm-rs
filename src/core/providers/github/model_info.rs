//! GitHub Models Model Information
//!
//! Contains model metadata, pricing, and capability information for GitHub Models.

/// GitHub model information
#[derive(Debug, Clone)]
pub struct GitHubModel {
    /// Model ID used in API calls
    pub model_id: &'static str,
    /// Display name for the model
    pub display_name: &'static str,
    /// Context window size
    pub max_context_length: u32,
    /// Maximum output tokens
    pub max_output_length: u32,
    /// Whether the model supports tools/function calling
    pub supports_tools: bool,
    /// Whether the model supports vision/images
    pub supports_multimodal: bool,
    /// Whether the model supports streaming
    pub supports_streaming: bool,
    /// Input cost per million tokens (USD)
    pub input_cost_per_million: f64,
    /// Output cost per million tokens (USD)
    pub output_cost_per_million: f64,
}

/// Static model registry for GitHub Models
static GITHUB_MODELS: &[GitHubModel] = &[
    // OpenAI Models
    GitHubModel {
        model_id: "gpt-4o",
        display_name: "GPT-4o",
        max_context_length: 128000,
        max_output_length: 16384,
        supports_tools: true,
        supports_multimodal: true,
        supports_streaming: true,
        input_cost_per_million: 2.5,
        output_cost_per_million: 10.0,
    },
    GitHubModel {
        model_id: "gpt-4o-mini",
        display_name: "GPT-4o Mini",
        max_context_length: 128000,
        max_output_length: 16384,
        supports_tools: true,
        supports_multimodal: true,
        supports_streaming: true,
        input_cost_per_million: 0.15,
        output_cost_per_million: 0.6,
    },
    GitHubModel {
        model_id: "o1-preview",
        display_name: "O1 Preview",
        max_context_length: 128000,
        max_output_length: 32768,
        supports_tools: false,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 15.0,
        output_cost_per_million: 60.0,
    },
    GitHubModel {
        model_id: "o1-mini",
        display_name: "O1 Mini",
        max_context_length: 128000,
        max_output_length: 65536,
        supports_tools: false,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 3.0,
        output_cost_per_million: 12.0,
    },
    // Meta Llama Models
    GitHubModel {
        model_id: "meta-llama-3.1-405b-instruct",
        display_name: "Meta Llama 3.1 405B Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: true,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
    GitHubModel {
        model_id: "meta-llama-3.1-70b-instruct",
        display_name: "Meta Llama 3.1 70B Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: true,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
    GitHubModel {
        model_id: "meta-llama-3.1-8b-instruct",
        display_name: "Meta Llama 3.1 8B Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: true,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
    // Mistral Models
    GitHubModel {
        model_id: "mistral-large-2407",
        display_name: "Mistral Large 2407",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: true,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
    GitHubModel {
        model_id: "mistral-small-2409",
        display_name: "Mistral Small 2409",
        max_context_length: 32000,
        max_output_length: 4096,
        supports_tools: true,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
    // Cohere Models
    GitHubModel {
        model_id: "cohere-command-r-plus",
        display_name: "Cohere Command R+",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: true,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
    GitHubModel {
        model_id: "cohere-command-r",
        display_name: "Cohere Command R",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: true,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
    // AI21 Models
    GitHubModel {
        model_id: "ai21-jamba-1.5-large",
        display_name: "AI21 Jamba 1.5 Large",
        max_context_length: 256000,
        max_output_length: 4096,
        supports_tools: false,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
    GitHubModel {
        model_id: "ai21-jamba-1.5-mini",
        display_name: "AI21 Jamba 1.5 Mini",
        max_context_length: 256000,
        max_output_length: 4096,
        supports_tools: false,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
    // Phi Models
    GitHubModel {
        model_id: "phi-3.5-moe-instruct",
        display_name: "Phi 3.5 MoE Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: false,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
    GitHubModel {
        model_id: "phi-3.5-mini-instruct",
        display_name: "Phi 3.5 Mini Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: false,
        supports_multimodal: false,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
    GitHubModel {
        model_id: "phi-3.5-vision-instruct",
        display_name: "Phi 3.5 Vision Instruct",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: false,
        supports_multimodal: true,
        supports_streaming: true,
        input_cost_per_million: 0.0,
        output_cost_per_million: 0.0,
    },
];

/// Get all available GitHub models
pub fn get_available_models() -> Vec<&'static str> {
    GITHUB_MODELS.iter().map(|m| m.model_id).collect()
}

/// Get model information by ID
pub fn get_model_info(model_id: &str) -> Option<&'static GitHubModel> {
    GITHUB_MODELS.iter().find(|m| m.model_id == model_id)
}

/// Check if a model supports vision
#[cfg(test)]
pub fn is_vision_model(model_id: &str) -> bool {
    get_model_info(model_id).is_some_and(|m| m.supports_multimodal)
}

/// Check if a model supports tools
#[cfg(test)]
pub fn supports_tools(model_id: &str) -> bool {
    get_model_info(model_id).is_some_and(|m| m.supports_tools)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"gpt-4o"));
        assert!(models.contains(&"gpt-4o-mini"));
        assert!(models.contains(&"meta-llama-3.1-70b-instruct"));
    }

    #[test]
    fn test_get_model_info() {
        let model = get_model_info("gpt-4o");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.model_id, "gpt-4o");
        assert_eq!(model.max_context_length, 128000);
        assert!(model.supports_tools);
        assert!(model.supports_multimodal);
    }

    #[test]
    fn test_get_model_info_nonexistent() {
        let model = get_model_info("nonexistent-model");
        assert!(model.is_none());
    }

    #[test]
    fn test_is_vision_model() {
        assert!(is_vision_model("gpt-4o"));
        assert!(is_vision_model("gpt-4o-mini"));
        assert!(is_vision_model("phi-3.5-vision-instruct"));
        assert!(!is_vision_model("meta-llama-3.1-70b-instruct"));
    }

    #[test]
    fn test_supports_tools() {
        assert!(supports_tools("gpt-4o"));
        assert!(supports_tools("meta-llama-3.1-70b-instruct"));
        assert!(supports_tools("mistral-large-2407"));
        assert!(!supports_tools("o1-preview"));
    }

    #[test]
    fn test_model_pricing() {
        let model = get_model_info("gpt-4o").unwrap();
        assert_eq!(model.input_cost_per_million, 2.5);
        assert_eq!(model.output_cost_per_million, 10.0);

        // Free models should have zero cost
        let llama = get_model_info("meta-llama-3.1-70b-instruct").unwrap();
        assert_eq!(llama.input_cost_per_million, 0.0);
        assert_eq!(llama.output_cost_per_million, 0.0);
    }
}
