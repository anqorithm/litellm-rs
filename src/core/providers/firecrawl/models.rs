//! Firecrawl Model Information

use crate::core::types::ModelInfo;
use std::collections::HashMap;

pub struct FirecrawlModelRegistry;

impl FirecrawlModelRegistry {
    pub fn get_models() -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "firecrawl-model".to_string(),
            name: "Firecrawl Model".to_string(),
            provider: "firecrawl".to_string(),
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

pub fn get_firecrawl_registry() -> FirecrawlModelRegistry {
    FirecrawlModelRegistry
}
