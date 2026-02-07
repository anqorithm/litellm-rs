//! Maritalk Model Information
//!
//! Maritalk specializes in Brazilian Portuguese language models.
//! Sabiá models are designed specifically for Portuguese language understanding and generation.

use crate::core::types::ModelInfo;

pub fn get_supported_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "sabia-2-medium".to_string(),
            name: "Sabiá-2 Medium".to_string(),
            provider: "maritalk".to_string(),
            max_context_length: 8192,
            max_output_length: Some(4096),
            input_cost_per_1k_tokens: Some(0.00002),
            output_cost_per_1k_tokens: Some(0.00004),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            ..Default::default()
        },
        ModelInfo {
            id: "sabia-2-small".to_string(),
            name: "Sabiá-2 Small".to_string(),
            provider: "maritalk".to_string(),
            max_context_length: 4096,
            max_output_length: Some(2048),
            input_cost_per_1k_tokens: Some(0.00001),
            output_cost_per_1k_tokens: Some(0.00002),
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
