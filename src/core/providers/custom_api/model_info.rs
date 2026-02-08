//! Custom HTTPX Model Information

use crate::core::types::model::ModelInfo;

pub fn get_supported_models() -> Vec<ModelInfo> {
    vec![ModelInfo {
        id: "custom-model".to_string(),
        name: "Custom HTTP Model".to_string(),
        provider: "custom_httpx".to_string(),
        max_context_length: 4096,
        max_output_length: Some(2048),
        input_cost_per_1k_tokens: None,
        output_cost_per_1k_tokens: None,
        supports_streaming: false,
        supports_tools: false,
        supports_multimodal: false,
        ..Default::default()
    }]
}

pub fn is_model_supported(_model_id: &str) -> bool {
    true
}
