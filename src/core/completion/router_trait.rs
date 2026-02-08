//! Router trait definition

use super::stream::CompletionStream;
use super::types::{CompletionOptions, CompletionResponse};
use crate::core::types::chat::ChatMessage;
use crate::utils::error::Result;
use async_trait::async_trait;

/// Unified message format (OpenAI compatible)
pub type Message = ChatMessage;

/// Router trait for handling completion requests
#[async_trait]
pub trait Router: Send + Sync + 'static {
    /// Complete a chat request
    async fn complete(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<CompletionResponse>;

    /// Complete a streaming chat request
    async fn complete_stream(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<CompletionStream>;
}
