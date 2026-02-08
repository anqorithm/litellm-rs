//! AIML API Model Information

use crate::core::types::model::ModelInfo;

pub fn get_supported_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            provider: "aiml".to_string(),
            max_context_length: 8192,
            max_output_length: Some(4096),
            input_cost_per_1k_tokens: Some(0.00003),
            output_cost_per_1k_tokens: Some(0.00006),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            ..Default::default()
        },
        ModelInfo {
            id: "gpt-3.5-turbo".to_string(),
            name: "GPT-3.5 Turbo".to_string(),
            provider: "aiml".to_string(),
            max_context_length: 4096,
            max_output_length: Some(4096),
            input_cost_per_1k_tokens: Some(0.0000015),
            output_cost_per_1k_tokens: Some(0.000002),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            ..Default::default()
        },
    ]
}

pub fn is_model_supported(model_id: &str) -> bool {
    get_supported_models().iter().any(|m| m.id == model_id)
}
