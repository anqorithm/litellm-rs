//! ExaAi Model Information

use crate::core::types::ModelInfo;
use std::collections::HashMap;

pub struct ExaAiModelRegistry;

impl ExaAiModelRegistry {
    pub fn get_models() -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "exa_ai-model".to_string(),
            name: "ExaAi Model".to_string(),
            provider: "exa_ai".to_string(),
            max_context_length: 8192,
            max_output_length: Some(4096),
            supports_streaming: true,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![
                crate::core::types::ProviderCapability::ChatCompletion,
                crate::core::types::ProviderCapability::ChatCompletionStream,
            ],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        }]
    }
}

pub fn get_exa_ai_registry() -> ExaAiModelRegistry {
    ExaAiModelRegistry
}
