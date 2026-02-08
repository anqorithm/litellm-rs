//! Yi (01.AI) Model Information

use crate::core::types::model::ModelInfo;

pub fn get_supported_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "yi-large".to_string(),
            name: "Yi-Large".to_string(),
            provider: "yi".to_string(),
            max_context_length: 32768,
            max_output_length: Some(4096),
            input_cost_per_1k_tokens: Some(0.003),
            output_cost_per_1k_tokens: Some(0.012),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            ..Default::default()
        },
        ModelInfo {
            id: "yi-large-turbo".to_string(),
            name: "Yi-Large-Turbo".to_string(),
            provider: "yi".to_string(),
            max_context_length: 16384,
            max_output_length: Some(4096),
            input_cost_per_1k_tokens: Some(0.0012),
            output_cost_per_1k_tokens: Some(0.0012),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            ..Default::default()
        },
        ModelInfo {
            id: "yi-medium".to_string(),
            name: "Yi-Medium".to_string(),
            provider: "yi".to_string(),
            max_context_length: 16384,
            max_output_length: Some(4096),
            input_cost_per_1k_tokens: Some(0.00025),
            output_cost_per_1k_tokens: Some(0.00025),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            ..Default::default()
        },
        ModelInfo {
            id: "yi-spark".to_string(),
            name: "Yi-Spark".to_string(),
            provider: "yi".to_string(),
            max_context_length: 16384,
            max_output_length: Some(4096),
            input_cost_per_1k_tokens: Some(0.0001),
            output_cost_per_1k_tokens: Some(0.0001),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            ..Default::default()
        },
        ModelInfo {
            id: "yi-vision".to_string(),
            name: "Yi-Vision".to_string(),
            provider: "yi".to_string(),
            max_context_length: 16384,
            max_output_length: Some(4096),
            input_cost_per_1k_tokens: Some(0.0006),
            output_cost_per_1k_tokens: Some(0.0006),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: true,
            ..Default::default()
        },
    ]
}

pub fn is_model_supported(model_id: &str) -> bool {
    get_supported_models().iter().any(|m| m.id == model_id)
}
