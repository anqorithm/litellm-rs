//! Snowflake Model Information
//!
//! Static model information for Snowflake Cortex AI models.
//! Includes capabilities and context lengths.

/// Information about a Snowflake Cortex model
#[derive(Debug, Clone)]
pub struct SnowflakeModel {
    /// Model ID (e.g., "claude-3-5-sonnet")
    pub model_id: &'static str,
    /// Display name
    pub display_name: &'static str,
    /// Maximum context length in tokens
    pub max_context_length: usize,
    /// Maximum output tokens
    pub max_output_length: usize,
    /// Whether the model supports tool/function calling
    pub supports_tools: bool,
    /// Whether the model supports streaming
    pub supports_streaming: bool,
    /// Model provider/family
    pub provider: &'static str,
    /// Short description
    pub description: &'static str,
}

/// Static model information for Snowflake Cortex models
static SNOWFLAKE_MODELS: &[SnowflakeModel] = &[
    // Anthropic Claude Models
    SnowflakeModel {
        model_id: "claude-3-5-sonnet",
        display_name: "Claude 3.5 Sonnet",
        max_context_length: 200000,
        max_output_length: 8192,
        supports_tools: true,
        supports_streaming: true,
        provider: "anthropic",
        description: "Anthropic's most intelligent model with tool calling support",
    },
    SnowflakeModel {
        model_id: "claude-3-5-haiku",
        display_name: "Claude 3.5 Haiku",
        max_context_length: 200000,
        max_output_length: 8192,
        supports_tools: true,
        supports_streaming: true,
        provider: "anthropic",
        description: "Fast and cost-effective Claude model with tool calling",
    },
    // Meta Llama Models
    SnowflakeModel {
        model_id: "llama3.1-8b",
        display_name: "Llama 3.1 8B",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "meta",
        description: "Meta's efficient 8B parameter model",
    },
    SnowflakeModel {
        model_id: "llama3.1-70b",
        display_name: "Llama 3.1 70B",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "meta",
        description: "Meta's powerful 70B parameter model",
    },
    SnowflakeModel {
        model_id: "llama3.1-405b",
        display_name: "Llama 3.1 405B",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "meta",
        description: "Meta's largest and most capable model",
    },
    SnowflakeModel {
        model_id: "llama3.2-1b",
        display_name: "Llama 3.2 1B",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "meta",
        description: "Compact model for edge deployment",
    },
    SnowflakeModel {
        model_id: "llama3.2-3b",
        display_name: "Llama 3.2 3B",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "meta",
        description: "Small but capable model for mobile and edge",
    },
    // Mistral Models
    SnowflakeModel {
        model_id: "mistral-large",
        display_name: "Mistral Large",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "mistral",
        description: "Mistral's most capable model",
    },
    SnowflakeModel {
        model_id: "mistral-large2",
        display_name: "Mistral Large 2",
        max_context_length: 128000,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "mistral",
        description: "Latest version of Mistral Large",
    },
    SnowflakeModel {
        model_id: "mixtral-8x7b",
        display_name: "Mixtral 8x7B",
        max_context_length: 32768,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "mistral",
        description: "Efficient mixture-of-experts model",
    },
    SnowflakeModel {
        model_id: "mistral-7b",
        display_name: "Mistral 7B",
        max_context_length: 32768,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "mistral",
        description: "Compact and efficient Mistral model",
    },
    // Snowflake Arctic
    SnowflakeModel {
        model_id: "snowflake-arctic",
        display_name: "Snowflake Arctic",
        max_context_length: 8192,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "snowflake",
        description: "Snowflake's enterprise-focused model",
    },
    // Reka Models
    SnowflakeModel {
        model_id: "reka-core",
        display_name: "Reka Core",
        max_context_length: 32768,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "reka",
        description: "Reka's most capable multimodal model",
    },
    SnowflakeModel {
        model_id: "reka-flash",
        display_name: "Reka Flash",
        max_context_length: 32768,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "reka",
        description: "Fast and efficient Reka model",
    },
    // Gemma Models
    SnowflakeModel {
        model_id: "gemma-7b",
        display_name: "Gemma 7B",
        max_context_length: 8192,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "google",
        description: "Google's open-weight Gemma model",
    },
    // Jamba Models
    SnowflakeModel {
        model_id: "jamba-1.5-mini",
        display_name: "Jamba 1.5 Mini",
        max_context_length: 256000,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "ai21",
        description: "AI21's efficient hybrid model",
    },
    SnowflakeModel {
        model_id: "jamba-1.5-large",
        display_name: "Jamba 1.5 Large",
        max_context_length: 256000,
        max_output_length: 4096,
        supports_tools: false,
        supports_streaming: true,
        provider: "ai21",
        description: "AI21's large hybrid architecture model",
    },
];

/// Get model information by model ID
pub fn get_model_info(model_id: &str) -> Option<&'static SnowflakeModel> {
    SNOWFLAKE_MODELS.iter().find(|m| m.model_id == model_id)
}

/// Get all available models
pub fn get_available_models() -> &'static [SnowflakeModel] {
    SNOWFLAKE_MODELS
}

/// Get models by provider
#[cfg(test)]
pub fn get_models_by_provider(provider: &str) -> Vec<&'static SnowflakeModel> {
    SNOWFLAKE_MODELS
        .iter()
        .filter(|m| m.provider.eq_ignore_ascii_case(provider))
        .collect()
}

/// Check if a model supports tool calling
#[cfg(test)]
pub fn supports_tools(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|m| m.supports_tools)
        .unwrap_or(false)
}

/// Check if a model supports streaming
#[cfg(test)]
pub fn supports_streaming(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|m| m.supports_streaming)
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_info() {
        let model = get_model_info("claude-3-5-sonnet");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.display_name, "Claude 3.5 Sonnet");
        assert!(model.supports_tools);
        assert!(model.supports_streaming);
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

        // Check that we have Claude models
        let claude_models: Vec<_> = models
            .iter()
            .filter(|m| m.provider == "anthropic")
            .collect();
        assert!(!claude_models.is_empty());

        // Check that we have Llama models
        let llama_models: Vec<_> = models.iter().filter(|m| m.provider == "meta").collect();
        assert!(!llama_models.is_empty());
    }

    #[test]
    fn test_get_models_by_provider() {
        let anthropic_models = get_models_by_provider("anthropic");
        assert!(!anthropic_models.is_empty());
        for model in anthropic_models {
            assert_eq!(model.provider, "anthropic");
        }

        let meta_models = get_models_by_provider("meta");
        assert!(!meta_models.is_empty());
        for model in meta_models {
            assert_eq!(model.provider, "meta");
        }
    }

    #[test]
    fn test_supports_tools() {
        // Claude models should support tools
        assert!(supports_tools("claude-3-5-sonnet"));
        assert!(supports_tools("claude-3-5-haiku"));

        // Llama models don't support tools
        assert!(!supports_tools("llama3.1-70b"));

        // Unknown models default to false
        assert!(!supports_tools("unknown-model"));
    }

    #[test]
    fn test_supports_streaming() {
        assert!(supports_streaming("claude-3-5-sonnet"));
        assert!(supports_streaming("llama3.1-70b"));
        // Unknown models default to true
        assert!(supports_streaming("unknown-model"));
    }

    #[test]
    fn test_model_context_lengths() {
        for model in get_available_models() {
            assert!(model.max_context_length > 0);
            assert!(model.max_output_length > 0);
        }
    }

    #[test]
    fn test_claude_has_high_context() {
        let claude = get_model_info("claude-3-5-sonnet").unwrap();
        assert_eq!(claude.max_context_length, 200000);
    }

    #[test]
    fn test_jamba_has_highest_context() {
        let jamba = get_model_info("jamba-1.5-large").unwrap();
        assert_eq!(jamba.max_context_length, 256000);
    }
}
