//! Docker Model Runner Model Information

use crate::core::types::model::ModelInfo;
use std::collections::HashMap;

pub struct DockerModelRunnerModelRegistry;

impl DockerModelRunnerModelRegistry {
    pub fn get_models() -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "docker-model".to_string(),
            name: "Docker Model Runner".to_string(),
            provider: "docker_model_runner".to_string(),
            max_context_length: 4096,
            max_output_length: Some(2048),
            supports_streaming: true,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![
                crate::core::types::model::ProviderCapability::ChatCompletion,
                crate::core::types::model::ProviderCapability::ChatCompletionStream,
            ],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        }]
    }
}

pub fn get_docker_model_runner_registry() -> DockerModelRunnerModelRegistry {
    DockerModelRunnerModelRegistry
}
