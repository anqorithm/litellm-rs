//! Streaming Module for Lambda Labs AI
//!
//! Uses the unified SSE parser for consistent streaming across providers.
//! Lambda Labs uses OpenAI-compatible SSE format.

use super::error::LambdaAIError;
use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};
use crate::core::types::responses::{ChatChunk, ChatDelta, ChatResponse, ChatStreamChoice};
use crate::core::types::{message::MessageContent, message::MessageRole};
use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

/// Lambda Labs uses OpenAI-compatible SSE format
pub type LambdaAIStreamInner = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    OpenAICompatibleTransformer,
>;

/// Helper function to create Lambda AI stream
pub fn create_lambda_ai_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> LambdaAIStreamInner {
    let transformer = OpenAICompatibleTransformer::new("lambda_ai");
    UnifiedSSEStream::new(Box::pin(stream), transformer)
}

/// Wrapper stream that converts ProviderError to LambdaAIError for backward compatibility
pub struct LambdaAIStream {
    inner: LambdaAIStreamInner,
}

impl LambdaAIStream {
    pub fn new(stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static) -> Self {
        Self {
            inner: create_lambda_ai_stream(stream),
        }
    }
}

impl Stream for LambdaAIStream {
    type Item = Result<ChatChunk, LambdaAIError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::pin::Pin;
        use std::task::Poll;

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => Poll::Ready(Some(Ok(chunk))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(LambdaAIError::streaming_error(
                "lambda_ai",
                "chat",
                None,
                None,
                e.to_string(),
            )))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Create a fake stream from a complete response
pub async fn create_fake_stream(
    response: ChatResponse,
) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, LambdaAIError>> + Send>>, LambdaAIError> {
    // Convert response to chunks
    let chunks = response_to_chunks(response);
    let stream = futures::stream::iter(chunks.into_iter().map(Ok));
    Ok(Box::pin(stream))
}

/// Convert a complete ChatResponse to stream chunks
///
/// Pre-extracts shared fields (id, model, system_fingerprint) to avoid
/// cloning from the full response struct on every chunk. The last chunk
/// moves the values instead of cloning (zero-cost).
fn response_to_chunks(response: ChatResponse) -> Vec<ChatChunk> {
    // Extract shared fields once — avoids repeated clones from the full response
    let id = response.id;
    let created = response.created;
    let model = response.model;
    let system_fingerprint = response.system_fingerprint;
    let usage = response.usage;
    let object = "chat.completion.chunk".to_string();

    let mut chunks = Vec::new();

    // Create initial chunk with role
    chunks.push(ChatChunk {
        id: id.clone(),
        object: object.clone(),
        created,
        model: model.clone(),
        system_fingerprint: system_fingerprint.clone(),
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

    // Create content chunks + final chunk
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
                    id: id.clone(),
                    object: object.clone(),
                    created,
                    model: model.clone(),
                    system_fingerprint: system_fingerprint.clone(),
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

        // Final chunk moves shared fields instead of cloning
        chunks.push(ChatChunk {
            id,
            object,
            created,
            model,
            system_fingerprint,
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
            usage,
        });
    }

    chunks
}
