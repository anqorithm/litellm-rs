//! Model Configuration for Bedrock Models
//!
//! Defines model families, capabilities, and routing configuration
//! for all supported Bedrock models.

use crate::core::providers::unified_provider::ProviderError;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Bedrock model families
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BedrockModelFamily {
    Claude,
    TitanText,
    TitanEmbedding,
    TitanImage,
    Nova,
    Llama,
    Mistral,
    AI21,
    Cohere,
    DeepSeek,
    StabilityAI,
}

/// Bedrock API types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BedrockApiType {
    Invoke,
    Converse,
    InvokeStream,
    ConverseStream,
}

/// Model configuration for routing and capabilities
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub family: BedrockModelFamily,
    pub api_type: BedrockApiType,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub supports_multimodal: bool,
    pub max_context_length: u32,
    pub max_output_length: Option<u32>,
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
}

/// Model configuration database
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelConfig>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    // Claude models
    configs.insert(
        "anthropic.claude-3-opus-20240229",
        ModelConfig {
            family: BedrockModelFamily::Claude,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: true,
            supports_multimodal: true,
            max_context_length: 200000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.015,
            output_cost_per_1k: 0.075,
        },
    );

    configs.insert(
        "anthropic.claude-3-sonnet-20240229",
        ModelConfig {
            family: BedrockModelFamily::Claude,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: true,
            supports_multimodal: true,
            max_context_length: 200000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        },
    );

    configs.insert(
        "anthropic.claude-3-haiku-20240307",
        ModelConfig {
            family: BedrockModelFamily::Claude,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: true,
            supports_multimodal: true,
            max_context_length: 200000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00025,
            output_cost_per_1k: 0.00125,
        },
    );

    configs.insert(
        "anthropic.claude-3-5-sonnet-20241022",
        ModelConfig {
            family: BedrockModelFamily::Claude,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: true,
            supports_multimodal: true,
            max_context_length: 200000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        },
    );
    configs.insert(
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
        ModelConfig {
            family: BedrockModelFamily::Claude,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: true,
            supports_multimodal: true,
            max_context_length: 200000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        },
    );

    configs.insert(
        "anthropic.claude-3-5-haiku-20241022",
        ModelConfig {
            family: BedrockModelFamily::Claude,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: true,
            supports_multimodal: true,
            max_context_length: 200000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.001,
            output_cost_per_1k: 0.005,
        },
    );

    configs.insert(
        "anthropic.claude-v2:1",
        ModelConfig {
            family: BedrockModelFamily::Claude,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 100000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.008,
            output_cost_per_1k: 0.024,
        },
    );

    configs.insert(
        "anthropic.claude-v2",
        ModelConfig {
            family: BedrockModelFamily::Claude,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 100000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.008,
            output_cost_per_1k: 0.024,
        },
    );

    configs.insert(
        "anthropic.claude-instant-v1",
        ModelConfig {
            family: BedrockModelFamily::Claude,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 100000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00163,
            output_cost_per_1k: 0.00551,
        },
    );

    // Titan models
    configs.insert(
        "amazon.titan-text-express-v1",
        ModelConfig {
            family: BedrockModelFamily::TitanText,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 8000,
            max_output_length: Some(8000),
            input_cost_per_1k: 0.0002,
            output_cost_per_1k: 0.0006,
        },
    );

    configs.insert(
        "amazon.titan-text-lite-v1",
        ModelConfig {
            family: BedrockModelFamily::TitanText,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 4000,
            max_output_length: Some(4000),
            input_cost_per_1k: 0.00015,
            output_cost_per_1k: 0.0002,
        },
    );

    configs.insert(
        "amazon.titan-text-premier-v1:0",
        ModelConfig {
            family: BedrockModelFamily::TitanText,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 32000,
            max_output_length: Some(32000),
            input_cost_per_1k: 0.0005,
            output_cost_per_1k: 0.0015,
        },
    );

    configs.insert(
        "amazon.titan-embed-text-v1",
        ModelConfig {
            family: BedrockModelFamily::TitanEmbedding,
            api_type: BedrockApiType::Invoke,
            supports_streaming: false,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 8000,
            max_output_length: None,
            input_cost_per_1k: 0.0001,
            output_cost_per_1k: 0.0,
        },
    );

    // Nova models
    configs.insert(
        "amazon.nova-micro-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Nova,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: true,
            supports_multimodal: true,
            max_context_length: 128000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.000035,
            output_cost_per_1k: 0.00014,
        },
    );

    configs.insert(
        "amazon.nova-lite-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Nova,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: true,
            supports_multimodal: true,
            max_context_length: 300000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00006,
            output_cost_per_1k: 0.00024,
        },
    );

    configs.insert(
        "amazon.nova-pro-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Nova,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: true,
            supports_multimodal: true,
            max_context_length: 300000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.0008,
            output_cost_per_1k: 0.0032,
        },
    );

    // Meta Llama models
    configs.insert(
        "meta.llama3-2-1b-instruct-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Llama,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 131072,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00001,
            output_cost_per_1k: 0.00001,
        },
    );

    configs.insert(
        "meta.llama3-2-3b-instruct-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Llama,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 131072,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.000015,
            output_cost_per_1k: 0.000015,
        },
    );

    configs.insert(
        "meta.llama3-2-11b-instruct-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Llama,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 131072,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.000032,
            output_cost_per_1k: 0.000032,
        },
    );

    configs.insert(
        "meta.llama3-2-90b-instruct-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Llama,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 131072,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00072,
            output_cost_per_1k: 0.00072,
        },
    );

    configs.insert(
        "meta.llama3-1-8b-instruct-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Llama,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 131072,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00022,
            output_cost_per_1k: 0.00022,
        },
    );

    configs.insert(
        "meta.llama3-1-70b-instruct-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Llama,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 131072,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00099,
            output_cost_per_1k: 0.00099,
        },
    );

    configs.insert(
        "meta.llama3-1-405b-instruct-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Llama,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 131072,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00532,
            output_cost_per_1k: 0.016,
        },
    );

    configs.insert(
        "meta.llama3-8b-instruct-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Llama,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 8192,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.0003,
            output_cost_per_1k: 0.0006,
        },
    );

    configs.insert(
        "meta.llama3-70b-instruct-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Llama,
            api_type: BedrockApiType::Converse,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 8192,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00265,
            output_cost_per_1k: 0.0035,
        },
    );

    configs.insert(
        "meta.llama2-13b-chat-v1",
        ModelConfig {
            family: BedrockModelFamily::Llama,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 4096,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00075,
            output_cost_per_1k: 0.001,
        },
    );

    configs.insert(
        "meta.llama2-70b-chat-v1",
        ModelConfig {
            family: BedrockModelFamily::Llama,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 4096,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00195,
            output_cost_per_1k: 0.00256,
        },
    );

    // AI21 models
    configs.insert(
        "ai21.jamba-1-5-large-v1:0",
        ModelConfig {
            family: BedrockModelFamily::AI21,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 256000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.002,
            output_cost_per_1k: 0.008,
        },
    );

    configs.insert(
        "ai21.jamba-1-5-mini-v1:0",
        ModelConfig {
            family: BedrockModelFamily::AI21,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 256000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.0002,
            output_cost_per_1k: 0.0004,
        },
    );

    configs.insert(
        "ai21.jamba-instruct-v1:0",
        ModelConfig {
            family: BedrockModelFamily::AI21,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 70000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.0005,
            output_cost_per_1k: 0.0007,
        },
    );

    // Cohere models
    configs.insert(
        "cohere.command-r-plus-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Cohere,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 128000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        },
    );

    configs.insert(
        "cohere.command-r-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Cohere,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 128000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.0005,
            output_cost_per_1k: 0.0015,
        },
    );

    configs.insert(
        "cohere.command-text-v14",
        ModelConfig {
            family: BedrockModelFamily::Cohere,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 4096,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.0015,
            output_cost_per_1k: 0.002,
        },
    );

    configs.insert(
        "cohere.command-light-text-v14",
        ModelConfig {
            family: BedrockModelFamily::Cohere,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 4096,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.0003,
            output_cost_per_1k: 0.0006,
        },
    );

    // Mistral models
    configs.insert(
        "mistral.mistral-7b-instruct-v0:2",
        ModelConfig {
            family: BedrockModelFamily::Mistral,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 32000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00015,
            output_cost_per_1k: 0.0002,
        },
    );

    configs.insert(
        "mistral.mixtral-8x7b-instruct-v0:1",
        ModelConfig {
            family: BedrockModelFamily::Mistral,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 32000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.00045,
            output_cost_per_1k: 0.0007,
        },
    );

    configs.insert(
        "mistral.mistral-large-2402-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Mistral,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 32000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.004,
            output_cost_per_1k: 0.012,
        },
    );

    configs.insert(
        "mistral.mistral-large-2407-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Mistral,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 128000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.002,
            output_cost_per_1k: 0.006,
        },
    );

    configs.insert(
        "mistral.mistral-small-2402-v1:0",
        ModelConfig {
            family: BedrockModelFamily::Mistral,
            api_type: BedrockApiType::Invoke,
            supports_streaming: true,
            supports_function_calling: false,
            supports_multimodal: false,
            max_context_length: 32000,
            max_output_length: Some(4096),
            input_cost_per_1k: 0.001,
            output_cost_per_1k: 0.003,
        },
    );

    configs
});

/// Get model configuration for a specific model ID
pub fn get_model_config(model_id: &str) -> Result<&'static ModelConfig, ProviderError> {
    MODEL_CONFIGS.get(model_id).ok_or_else(|| {
        ProviderError::model_not_found("bedrock", format!("Model {} not supported", model_id))
    })
}

/// Check if a model supports a specific capability
pub fn model_supports_capability(model_id: &str, capability: &str) -> bool {
    if let Ok(config) = get_model_config(model_id) {
        match capability {
            "streaming" => config.supports_streaming,
            "function_calling" => config.supports_function_calling,
            "multimodal" => config.supports_multimodal,
            _ => false,
        }
    } else {
        false
    }
}

/// Get all supported model IDs
pub fn get_all_model_ids() -> Vec<&'static str> {
    MODEL_CONFIGS.keys().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config_lookup() {
        let config = get_model_config("anthropic.claude-3-opus-20240229").unwrap();
        assert_eq!(config.family, BedrockModelFamily::Claude);
        assert_eq!(config.api_type, BedrockApiType::Converse);
        assert!(config.supports_streaming);
        assert!(config.supports_function_calling);
        assert!(config.supports_multimodal);

        let sonnet_v2 = get_model_config("anthropic.claude-3-5-sonnet-20241022-v2:0").unwrap();
        assert_eq!(sonnet_v2.family, BedrockModelFamily::Claude);
        assert_eq!(sonnet_v2.api_type, BedrockApiType::Converse);
    }

    #[test]
    fn test_model_capabilities() {
        assert!(model_supports_capability(
            "anthropic.claude-3-opus-20240229",
            "streaming"
        ));
        assert!(model_supports_capability(
            "anthropic.claude-3-opus-20240229",
            "function_calling"
        ));
        assert!(model_supports_capability(
            "anthropic.claude-3-opus-20240229",
            "multimodal"
        ));

        assert!(!model_supports_capability(
            "amazon.titan-text-express-v1",
            "function_calling"
        ));
        assert!(!model_supports_capability(
            "amazon.titan-text-express-v1",
            "multimodal"
        ));
    }

    #[test]
    fn test_unknown_model() {
        assert!(get_model_config("unknown-model").is_err());
        assert!(!model_supports_capability("unknown-model", "streaming"));
    }

    #[test]
    fn test_model_families() {
        let claude_config = get_model_config("anthropic.claude-3-opus-20240229").unwrap();
        assert_eq!(claude_config.family, BedrockModelFamily::Claude);

        let titan_config = get_model_config("amazon.titan-text-express-v1").unwrap();
        assert_eq!(titan_config.family, BedrockModelFamily::TitanText);

        let nova_config = get_model_config("amazon.nova-pro-v1:0").unwrap();
        assert_eq!(nova_config.family, BedrockModelFamily::Nova);
    }

    #[test]
    fn test_api_types() {
        let claude_config = get_model_config("anthropic.claude-3-opus-20240229").unwrap();
        assert_eq!(claude_config.api_type, BedrockApiType::Converse);

        let titan_config = get_model_config("amazon.titan-text-express-v1").unwrap();
        assert_eq!(titan_config.api_type, BedrockApiType::Invoke);
    }
}
