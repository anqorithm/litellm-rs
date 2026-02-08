//! Streaming Module for Hosted vLLM
//!
//! Uses the unified SSE parser for consistent streaming across providers.
//! vLLM uses OpenAI-compatible SSE format for streaming responses.

use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::{ChatChunk, ChatDelta, ChatResponse, ChatStreamChoice};
use crate::core::types::{message::MessageContent, message::MessageRole};
use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

/// Provider name for error messages
const PROVIDER_NAME: &str = "hosted_vllm";

/// Hosted vLLM uses OpenAI-compatible SSE format
pub type HostedVLLMStreamInner = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    OpenAICompatibleTransformer,
>;

/// Helper function to create hosted vLLM stream
pub fn create_hosted_vllm_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> HostedVLLMStreamInner {
    let transformer = OpenAICompatibleTransformer::new(PROVIDER_NAME);
    UnifiedSSEStream::new(Box::pin(stream), transformer)
}

/// Wrapper stream that provides ProviderError type
pub struct HostedVLLMStream {
    inner: HostedVLLMStreamInner,
}

impl HostedVLLMStream {
    pub fn new(stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static) -> Self {
        Self {
            inner: create_hosted_vllm_stream(stream),
        }
    }
}

impl Stream for HostedVLLMStream {
    type Item = Result<ChatChunk, ProviderError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll;

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => Poll::Ready(Some(Ok(chunk))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(ProviderError::streaming_error(
                PROVIDER_NAME,
                "chat",
                None,
                None,
                format!("Streaming error: {}", e),
            )))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Create a fake stream from a complete response (for batch processing)
pub async fn create_fake_stream(
    response: ChatResponse,
) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError> {
    let chunks = response_to_chunks(response);
    let stream = futures::stream::iter(chunks.into_iter().map(Ok));
    Ok(Box::pin(stream))
}

/// Convert a complete ChatResponse to stream chunks
fn response_to_chunks(response: ChatResponse) -> Vec<ChatChunk> {
    let mut chunks = Vec::new();

    // Create initial chunk with role
    chunks.push(ChatChunk {
        id: response.id.clone(),
        object: "chat.completion.chunk".to_string(),
        created: response.created,
        model: response.model.clone(),
        system_fingerprint: response.system_fingerprint.clone(),
        choices: vec![ChatStreamChoice {
            index: 0,
            delta: ChatDelta {
                role: Some(MessageRole::Assistant),
                content: None,
                thinking: None,
                tool_calls: None,
                function_call: None,
            },
            finish_reason: None,
            logprobs: None,
        }],
        usage: None,
    });

    // Create content chunks
    if let Some(choice) = response.choices.first() {
        if let Some(content) = &choice.message.content {
            let text = match content {
                MessageContent::Text(text) => text.clone(),
                MessageContent::Parts(_) => content.to_string(),
            };

            // Split content into smaller chunks for more natural streaming
            let words: Vec<&str> = text.split_whitespace().collect();
            let chunk_size = 5; // Words per chunk

            for word_chunk in words.chunks(chunk_size) {
                let chunk_text = word_chunk.join(" ") + " ";
                chunks.push(ChatChunk {
                    id: response.id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: response.created,
                    model: response.model.clone(),
                    system_fingerprint: response.system_fingerprint.clone(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: Some(chunk_text),
                            thinking: None,
                            tool_calls: None,
                            function_call: None,
                        },
                        finish_reason: None,
                        logprobs: None,
                    }],
                    usage: None,
                });
            }
        }

        // Add final chunk with finish_reason
        chunks.push(ChatChunk {
            id: response.id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: response.created,
            model: response.model.clone(),
            system_fingerprint: response.system_fingerprint.clone(),
            choices: vec![ChatStreamChoice {
                index: 0,
                delta: ChatDelta {
                    role: None,
                    content: None,
                    thinking: None,
                    tool_calls: None,
                    function_call: None,
                },
                finish_reason: choice.finish_reason.clone(),
                logprobs: None,
            }],
            usage: response.usage.clone(),
        });
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::responses::{ChatChoice, FinishReason, Usage};

    fn create_test_response() -> ChatResponse {
        use crate::core::types::ChatMessage;

        ChatResponse {
            id: "test-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "test-model".to_string(),
            system_fingerprint: Some("fp_test".to_string()),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: Some(MessageContent::Text(
                        "Hello world this is a test response".to_string(),
                    )),
                    name: None,
                    tool_calls: None,
                    function_call: None,
                    thinking: None,
                    ..Default::default()
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            usage: Some(Usage {
                prompt_tokens: 10,
                completion_tokens: 7,
                total_tokens: 17,
                ..Default::default()
            }),
        }
    }

    #[test]
    fn test_response_to_chunks() {
        let response = create_test_response();
        let chunks = response_to_chunks(response);

        // Should have at least 3 chunks: role, content, finish
        assert!(chunks.len() >= 3);

        // First chunk should have role
        assert_eq!(
            chunks[0].choices[0].delta.role,
            Some(MessageRole::Assistant)
        );
        assert!(chunks[0].choices[0].delta.content.is_none());

        // Last chunk should have finish_reason
        let last = chunks.last().unwrap();
        assert_eq!(last.choices[0].finish_reason, Some(FinishReason::Stop));
    }

    #[test]
    fn test_chunks_have_correct_metadata() {
        let response = create_test_response();
        let chunks = response_to_chunks(response);

        for chunk in &chunks {
            assert_eq!(chunk.id, "test-id");
            assert_eq!(chunk.model, "test-model");
            assert_eq!(chunk.object, "chat.completion.chunk");
            assert_eq!(chunk.created, 1234567890);
        }
    }

    #[test]
    fn test_empty_response_handling() {
        let response = ChatResponse {
            id: "empty-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "test-model".to_string(),
            system_fingerprint: None,
            choices: vec![],
            usage: None,
        };

        let chunks = response_to_chunks(response);
        // Should still have the initial role chunk
        assert!(!chunks.is_empty());
    }
}
