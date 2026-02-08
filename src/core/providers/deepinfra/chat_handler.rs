//! DeepInfra Chat Handler
//!
//! Chat completion functionality for DeepInfra platform

use crate::core::providers::deepinfra::DeepInfraConfig;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{
    chat::ChatRequest,
    context::RequestContext,
    responses::{ChatChunk, ChatResponse},
};
// Removed unused async_trait import
use futures::Stream;
use std::pin::Pin;

/// DeepInfra chat handler
#[derive(Debug, Clone)]
pub struct DeepInfraChatHandler {
    config: DeepInfraConfig,
}

impl DeepInfraChatHandler {
    /// Create a new chat handler
    pub fn new(config: DeepInfraConfig) -> Self {
        Self { config }
    }

    /// Handle chat completion request
    pub async fn chat_completion(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, ProviderError> {
        let api_base = self.config.get_effective_api_base();
        Err(ProviderError::not_implemented(
            "deepinfra",
            format!("Chat completion not yet implemented for endpoint {}", api_base),
        ))
    }

    /// Handle streaming chat completion request
    pub async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError>
    {
        let api_base = self.config.get_effective_api_base();
        Err(ProviderError::not_implemented(
            "deepinfra",
            format!("Chat streaming not yet implemented for endpoint {}", api_base),
        ))
    }
}
