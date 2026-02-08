//! Comet API Model Information

use crate::core::types::model::ModelInfo;

pub fn get_supported_models() -> Vec<ModelInfo> {
    vec![ModelInfo {
        id: "comet-chat".to_string(),
        name: "Comet Chat Model".to_string(),
        provider: "cometapi".to_string(),
        max_context_length: 8192,
        max_output_length: Some(4096),
        input_cost_per_1k_tokens: Some(0.000002),
        output_cost_per_1k_tokens: Some(0.000004),
        supports_streaming: true,
        supports_tools: true,
        supports_multimodal: false,
        ..Default::default()
    }]
}

pub fn is_model_supported(model_id: &str) -> bool {
    get_supported_models().iter().any(|m| m.id == model_id)
}
