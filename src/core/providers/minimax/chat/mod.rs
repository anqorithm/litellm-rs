//! Chat completion functionality for Minimax provider

pub mod transformation;

use serde_json::Value;
use tracing::{debug, info};

use crate::core::providers::minimax::{MinimaxConfig, MinimaxError};
pub use transformation::MinimaxChatTransformation;

/// Minimax chat handler
#[derive(Debug, Clone)]
pub struct MinimaxChatHandler {
    config: MinimaxConfig,
    transformation: MinimaxChatTransformation,
}

impl MinimaxChatHandler {
    /// Create a new chat handler
    pub fn new(config: MinimaxConfig) -> Result<Self, MinimaxError> {
        Ok(Self {
            config,
            transformation: MinimaxChatTransformation::new(),
        })
    }

    /// Get the config
    pub fn config(&self) -> &MinimaxConfig {
        &self.config
    }

    /// Transform a standard chat request to Minimax format
    pub fn transform_request(
        &self,
        request: crate::core::types::ChatRequest,
    ) -> Result<Value, MinimaxError> {
        debug!("Transforming chat request for Minimax");

        // Apply Minimax-specific transformations
        let transformed = self.transformation.transform_request(request)?;

        Ok(transformed)
    }

    /// Transform a Minimax response to standard format
    pub fn transform_response(
        &self,
        response: Value,
    ) -> Result<crate::core::types::responses::ChatResponse, MinimaxError> {
        debug!("Transforming Minimax response");

        // Apply Minimax-specific transformations
        let standard_response = self.transformation.transform_response(response)?;

        info!("Successfully transformed Minimax response");
        Ok(standard_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{ChatMessage, MessageContent, MessageRole};

    fn create_test_config() -> MinimaxConfig {
        MinimaxConfig {
            api_key: "test_key".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_chat_handler_creation() {
        let config = create_test_config();
        let handler = MinimaxChatHandler::new(config);
        assert!(handler.is_ok());
    }

    #[test]
    fn test_chat_handler_config_access() {
        let config = create_test_config();
        let handler = MinimaxChatHandler::new(config.clone()).unwrap();
        assert_eq!(handler.config().api_key, config.api_key);
    }

    #[test]
    fn test_transform_request() {
        let config = create_test_config();
        let handler = MinimaxChatHandler::new(config).unwrap();

        let request = crate::core::types::ChatRequest {
            model: "MiniMax-M2.1".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = handler.transform_request(request);
        assert!(result.is_ok());
    }
}
