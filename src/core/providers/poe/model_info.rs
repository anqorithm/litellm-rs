//! Poe Model Information

use crate::core::types::{model::ModelInfo, model::ProviderCapability};
use std::collections::HashMap;

pub fn get_models() -> Vec<ModelInfo> {
    vec![ModelInfo {
        id: "poe-default".to_string(),
        name: "Poe Default Model".to_string(),
        provider: "poe".to_string(),
        max_context_length: 8192,
        max_output_length: Some(4096),
        supports_streaming: true,
        supports_tools: false,
        supports_multimodal: false,
        input_cost_per_1k_tokens: Some(0.0),
        output_cost_per_1k_tokens: Some(0.0),
        currency: "USD".to_string(),
        capabilities: vec![
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
        ],
        created_at: None,
        updated_at: None,
        metadata: HashMap::new(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_models() {
        let models = get_models();
        assert!(!models.is_empty());
        assert_eq!(models[0].provider, "poe");
    }
}
