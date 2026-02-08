//! Chat completion functionality for Dashscope provider

pub mod transformation;

use serde_json::Value;
use tracing::{debug, info};

use crate::core::providers::dashscope::{DashscopeConfig, DashscopeError};
pub use transformation::DashscopeChatTransformation;

/// Dashscope chat handler
#[derive(Debug, Clone)]
pub struct DashscopeChatHandler {
    config: DashscopeConfig,
    transformation: DashscopeChatTransformation,
}

impl DashscopeChatHandler {
    /// Create a new chat handler
    pub fn new(config: DashscopeConfig) -> Result<Self, DashscopeError> {
        Ok(Self {
            config,
            transformation: DashscopeChatTransformation::new(),
        })
    }

    /// Get the config
    pub fn config(&self) -> &DashscopeConfig {
        &self.config
    }

    /// Transform a standard chat request to Dashscope format
    pub fn transform_request(
        &self,
        request: crate::core::types::ChatRequest,
    ) -> Result<Value, DashscopeError> {
        debug!("Transforming chat request for Dashscope");

        // Apply Dashscope-specific transformations
        let transformed = self.transformation.transform_request(request)?;

        Ok(transformed)
    }

    /// Transform a Dashscope response to standard format
    pub fn transform_response(
        &self,
        response: Value,
    ) -> Result<crate::core::types::responses::ChatResponse, DashscopeError> {
        debug!("Transforming Dashscope response");

        // Apply Dashscope-specific transformations
        let standard_response = self.transformation.transform_response(response)?;

        info!("Successfully transformed Dashscope response");
        Ok(standard_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{ChatMessage, message::MessageContent, message::MessageRole};

    fn create_test_config() -> DashscopeConfig {
        DashscopeConfig {
            api_key: "test_key".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_chat_handler_creation() {
        let config = create_test_config();
        let handler = DashscopeChatHandler::new(config);
        assert!(handler.is_ok());
    }

    #[test]
    fn test_chat_handler_config_access() {
        let config = create_test_config();
        let handler = DashscopeChatHandler::new(config.clone()).unwrap();
        assert_eq!(handler.config().api_key, config.api_key);
    }

    #[test]
    fn test_transform_request() {
        let config = create_test_config();
        let handler = DashscopeChatHandler::new(config).unwrap();

        let request = crate::core::types::ChatRequest {
            model: "qwen-turbo".to_string(),
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
