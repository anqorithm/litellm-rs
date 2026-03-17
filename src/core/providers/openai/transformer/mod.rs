//! OpenAI Request and Response Transformers
//!
//! Unified transformation layer for converting between unified LiteLLM types and OpenAI-specific formats.
//!
//! This module is split into focused submodules:
//! - `request` — request body conversion (ChatRequest -> OpenAIChatRequest)
//! - `response` — response body and streaming conversion (OpenAIChatResponse -> ChatResponse)

mod request;
mod response;

pub use request::OpenAIRequestTransformer;
pub use response::OpenAIResponseTransformer;

use crate::core::providers::openai::error::OpenAIError;
use crate::core::providers::openai::models::OpenAIChatRequest;
use crate::core::traits::transformer::Transform;
use crate::core::types::chat::ChatRequest;
use crate::core::types::responses::ChatResponse;

use super::models::OpenAIChatResponse;

/// OpenAI Transformer (compatible with old interface)
pub struct OpenAITransformer;

impl Transform<ChatRequest, OpenAIChatRequest> for OpenAITransformer {
    type Error = OpenAIError;

    fn transform(input: ChatRequest) -> Result<OpenAIChatRequest, Self::Error> {
        OpenAIRequestTransformer::transform(input)
    }
}

impl Transform<OpenAIChatResponse, ChatResponse> for OpenAITransformer {
    type Error = OpenAIError;

    fn transform(input: OpenAIChatResponse) -> Result<ChatResponse, Self::Error> {
        OpenAIResponseTransformer::transform(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_trait_request() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            ..Default::default()
        };

        let result =
            <OpenAITransformer as Transform<ChatRequest, OpenAIChatRequest>>::transform(request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transform_trait_response() {
        let response = OpenAIChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "gpt-4".to_string(),
            choices: vec![],
            usage: None,
            system_fingerprint: None,
        };

        let result =
            <OpenAITransformer as Transform<OpenAIChatResponse, ChatResponse>>::transform(response);
        assert!(result.is_ok());
    }
}
