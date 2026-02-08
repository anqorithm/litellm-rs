//! Bytez Model Information

use crate::core::types::model::ModelInfo;

pub fn get_supported_models() -> Vec<ModelInfo> {
    vec![ModelInfo {
        id: "bytez-chat".to_string(),
        name: "Bytez Chat Model".to_string(),
        provider: "bytez".to_string(),
        max_context_length: 4096,
        max_output_length: Some(2048),
        input_cost_per_1k_tokens: Some(0.000001),
        output_cost_per_1k_tokens: Some(0.000002),
        supports_streaming: true,
        supports_tools: false,
        supports_multimodal: false,
        ..Default::default()
    }]
}

pub fn is_model_supported(model_id: &str) -> bool {
    get_supported_models().iter().any(|m| m.id == model_id)
}
