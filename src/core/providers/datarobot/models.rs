//! DataRobot Model Information

use crate::core::types::model::ModelInfo;
use std::collections::HashMap;

pub struct DataRobotModelRegistry;

impl DataRobotModelRegistry {
    pub fn get_models() -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "datarobot-chat".to_string(),
            name: "DataRobot Chat Model".to_string(),
            provider: "datarobot".to_string(),
            max_context_length: 8192,
            max_output_length: Some(4096),
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

pub fn get_datarobot_registry() -> DataRobotModelRegistry {
    DataRobotModelRegistry
}
