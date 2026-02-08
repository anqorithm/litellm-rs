//! Featherless Model Information

use crate::core::types::model::ModelInfo;
use std::collections::HashMap;

pub struct FeatherlessModelRegistry;

impl FeatherlessModelRegistry {
    pub fn get_models() -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "featherless-model".to_string(),
            name: "Featherless Model".to_string(),
            provider: "featherless".to_string(),
            max_context_length: 8192,
            max_output_length: Some(4096),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_1k_tokens: Some(0.0001),
            output_cost_per_1k_tokens: Some(0.0002),
            currency: "USD".to_string(),
            capabilities: vec![
                crate::core::types::model::ProviderCapability::ChatCompletion,
                crate::core::types::model::ProviderCapability::ChatCompletionStream,
                crate::core::types::model::ProviderCapability::ToolCalling,
            ],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        }]
    }
}

pub fn get_featherless_registry() -> FeatherlessModelRegistry {
    FeatherlessModelRegistry
}
