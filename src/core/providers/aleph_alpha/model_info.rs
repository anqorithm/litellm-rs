//! Aleph Alpha Model Information

use crate::core::types::model::ModelInfo;

pub fn get_supported_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "luminous-supreme".to_string(),
            name: "Luminous Supreme".to_string(),
            provider: "aleph_alpha".to_string(),
            max_context_length: 2048,
            max_output_length: Some(1024),
            input_cost_per_1k_tokens: Some(0.03),
            output_cost_per_1k_tokens: Some(0.06),
            supports_streaming: true,
            supports_tools: false,
            supports_multimodal: true,
            ..Default::default()
        },
        ModelInfo {
            id: "luminous-extended".to_string(),
            name: "Luminous Extended".to_string(),
            provider: "aleph_alpha".to_string(),
            max_context_length: 2048,
            max_output_length: Some(1024),
            input_cost_per_1k_tokens: Some(0.015),
            output_cost_per_1k_tokens: Some(0.03),
            supports_streaming: true,
            supports_tools: false,
            supports_multimodal: true,
            ..Default::default()
        },
        ModelInfo {
            id: "luminous-base".to_string(),
            name: "Luminous Base".to_string(),
            provider: "aleph_alpha".to_string(),
            max_context_length: 2048,
            max_output_length: Some(1024),
            input_cost_per_1k_tokens: Some(0.006),
            output_cost_per_1k_tokens: Some(0.012),
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
