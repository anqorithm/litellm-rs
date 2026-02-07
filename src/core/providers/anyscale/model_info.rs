//! Anyscale Model Information

use crate::core::types::ModelInfo;

pub fn get_supported_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "meta-llama/Llama-2-70b-chat-hf".to_string(),
            name: "Llama 2 70B Chat".to_string(),
            provider: "anyscale".to_string(),
            max_context_length: 4096,
            max_output_length: Some(4096),
            input_cost_per_1k_tokens: Some(0.001),
            output_cost_per_1k_tokens: Some(0.001),
            supports_streaming: true,
            supports_tools: false,
            supports_multimodal: false,
            ..Default::default()
        },
        ModelInfo {
            id: "mistralai/Mistral-7B-Instruct-v0.1".to_string(),
            name: "Mistral 7B Instruct v0.1".to_string(),
            provider: "anyscale".to_string(),
            max_context_length: 8192,
            max_output_length: Some(8192),
            input_cost_per_1k_tokens: Some(0.00015),
            output_cost_per_1k_tokens: Some(0.00015),
            supports_streaming: true,
            supports_tools: false,
            supports_multimodal: false,
            ..Default::default()
        },
        ModelInfo {
            id: "codellama/CodeLlama-34b-Instruct-hf".to_string(),
            name: "CodeLlama 34B Instruct".to_string(),
            provider: "anyscale".to_string(),
            max_context_length: 16384,
            max_output_length: Some(16384),
            input_cost_per_1k_tokens: Some(0.001),
            output_cost_per_1k_tokens: Some(0.001),
            supports_streaming: true,
            supports_tools: false,
            supports_multimodal: false,
            ..Default::default()
        },
    ]
}

pub fn is_model_supported(model_id: &str) -> bool {
    get_supported_models().iter().any(|m| m.id == model_id)
}
