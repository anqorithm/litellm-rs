//! SiliconFlow Model Information

use crate::core::types::model::ModelInfo;

pub fn get_supported_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "deepseek-ai/DeepSeek-V2.5".to_string(),
            name: "DeepSeek-V2.5".to_string(),
            provider: "siliconflow".to_string(),
            max_context_length: 32768,
            max_output_length: Some(8192),
            input_cost_per_1k_tokens: Some(0.00014),
            output_cost_per_1k_tokens: Some(0.00028),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            ..Default::default()
        },
        ModelInfo {
            id: "Qwen/Qwen2.5-72B-Instruct".to_string(),
            name: "Qwen2.5-72B-Instruct".to_string(),
            provider: "siliconflow".to_string(),
            max_context_length: 32768,
            max_output_length: Some(8192),
            input_cost_per_1k_tokens: Some(0.00056),
            output_cost_per_1k_tokens: Some(0.00056),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            ..Default::default()
        },
        ModelInfo {
            id: "Pro/Qwen/Qwen2.5-Coder-32B-Instruct".to_string(),
            name: "Qwen2.5-Coder-32B-Instruct (Pro)".to_string(),
            provider: "siliconflow".to_string(),
            max_context_length: 32768,
            max_output_length: Some(8192),
            input_cost_per_1k_tokens: Some(0.00042),
            output_cost_per_1k_tokens: Some(0.00042),
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
