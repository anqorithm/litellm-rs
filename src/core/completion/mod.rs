//! Python LiteLLM compatible completion API
//!
//! This module provides a Python LiteLLM-style API for making completion requests.
//! It serves as the main entry point for the library, providing a unified interface
//! to call 100+ LLM APIs using OpenAI format.
//!
//! # Example
//! ```rust,no_run
//! # use litellm_rs::{completion, user_message, system_message};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let response = completion(
//!     "gpt-4",
//!     vec![
//!         system_message("You are helpful."),
//!         user_message("Hello!"),
//!     ],
//!     None,
//! ).await?;
//! # Ok(())
//! # }
//! ```

mod conversion;
mod default_router;
mod helpers;
mod router_trait;
mod stream;
mod types;

#[cfg(test)]
mod tests;

// Re-export main types
pub use conversion::{convert_from_chat_completion_response, convert_to_chat_completion_request};
pub use default_router::{DefaultRouter, ErrorRouter, acompletion, completion, completion_stream};
pub use helpers::{
    assistant_message, convert_messages_to_chat_messages, system_message, user_message,
};
pub use router_trait::{Message, Router};
pub use stream::{CompletionChunk, CompletionStream, StreamChoice, StreamDelta};
pub use types::{Choice, CompletionOptions, CompletionResponse, FunctionCall, ToolCall};

// Re-export types with proper paths
pub use crate::core::types::{content::ContentPart, message::MessageContent, message::MessageRole};

/// LiteLLM Error type alias
pub type LiteLLMError = crate::utils::error::GatewayError;

/// Usage statistics (re-export from core types)
pub type Usage = crate::core::types::responses::Usage;

/// Finish reason enumeration (re-export from core types)
pub type FinishReason = crate::core::types::responses::FinishReason;
